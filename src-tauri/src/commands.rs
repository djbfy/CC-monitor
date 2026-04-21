// commands.rs — Tauri commands

use crate::session::{SessionInfo, SessionMonitor, SessionState};
use crate::state_machine::{evaluate_state, matches_confirm, matches_idle, get_cpu_percent};
use crate::pty_watcher::spawn_pty;
use crate::AppState;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::System;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};
use crate::hooks_server::HookAction;
use tokio::sync::mpsc;

use uuid::Uuid;

#[cfg(windows)]
extern "system" {
    fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> *mut std::ffi::c_void;
    fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    fn GetCurrentProcessId() -> u32;
}

#[cfg(windows)]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

// === Constants ===

const OFFLINE_CLEANUP_SECS: u64 = 10 * 60;
const POLL_INTERVAL_MS: u64 = 1000; // 1 second between checks
const CPU_LOW_THRESHOLD: f32 = 0.5;
const STARTUP_GRACE_SECS: u64 = 30;

/// Shared monitoring loop for both discovered and launched sessions.
fn spawn_monitor_loop(
    monitor: Arc<Mutex<SessionMonitor>>,
    app_handle: tauri::AppHandle,
    poll_interval_ms: u64,
    session_id: String,
    cpu_threshold: f32,
    require_startup_grace: bool,
    monitors: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<SessionMonitor>>>>>,
) {
    let stopped = monitor.lock().unwrap().stopped.clone();
    let monitor_clone = monitor.clone();
    let app_clone = app_handle.clone();
    let monitors_clone = monitors.clone();

    tokio::spawn(async move {
        let mut sys = System::new_all();
        loop {
            if stopped.load(Ordering::Relaxed) {
                let _ = app_clone.emit("session_cleanup", &session_id);
                break;
            }

            tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;

            // Check stopped flag after sleep — no duplicate check needed before sleep
            if stopped.load(Ordering::Relaxed) {
                let _ = app_clone.emit("session_cleanup", &session_id);
                break;
            }

            let (new_state, old_state) = {
                let mut m = monitor_clone.lock().unwrap();
                let pid = m.pid.unwrap_or(0);

                // Refresh CPU data before reading — required for get_cpu_percent to work
                sys.refresh_processes();
                let cpu = get_cpu_percent(&mut sys, pid);
                m.cpu_percent = cpu;

                let started_at = m.started_at;
                let uptime_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                    .saturating_sub(started_at / 1000);

                let in_startup = require_startup_grace && uptime_secs < STARTUP_GRACE_SECS;

                if cpu < cpu_threshold && !in_startup {
                    m.cpu_low_consecutive += 1;
                } else if cpu >= cpu_threshold {
                    m.cpu_low_consecutive = 0;
                }

                let confirm = matches_confirm(&m.last_line);
                let idle = matches_idle(&m.last_line);
                let cpu_low = m.cpu_low_consecutive;
                let awaiting = m.awaiting_confirm;

                let old = m.state.clone();
                let new = evaluate_state(&m, &sys, confirm, idle, cpu_low, awaiting);
                m.state = new.clone();

                (new, old)
            };

            if old_state != new_state {
                let infos: Vec<SessionInfo> = monitors_clone.lock().unwrap().values()
                    .map(|m| m.lock().unwrap().to_info())
                    .collect();
                let _ = app_clone.emit("session_update", &infos);
            }

            if old_state != SessionState::Confirm && new_state == SessionState::Confirm {
                let name = monitor_clone.lock().unwrap().name.clone();
                let _ = app_clone.emit("session_confirm", &name);
            }

            if new_state == SessionState::Offline {
                let silent = monitor_clone.lock().unwrap().last_output.elapsed().as_secs();
                if silent >= OFFLINE_CLEANUP_SECS {
                    monitors_clone.lock().unwrap().remove(&session_id);
                    let _ = app_clone.emit("session_cleanup", &session_id);
                    break;
                }
            }
        }
    });
}

// === Tauri Commands ===

/// Auto-discover running Claude Code processes and add them to monitoring.
pub async fn discover_sessions(
    app: AppHandle,
    monitors: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<SessionMonitor>>>>>,
) {
    let (tx, mut rx) = mpsc::channel::<(SessionMonitor, String)>(100);

    let _ = tokio::task::spawn_blocking(move || {
        let mut sys = System::new_all();
        sys.refresh_processes();

        let mut seen_pids: HashSet<u32> = HashSet::new();

        // Skip processes whose parent is cc-monitor itself (i.e., spawned by this monitor's sub-agents)
        #[cfg(windows)]
        let monitor_pid = unsafe { GetCurrentProcessId() };

        for (pid, proc) in sys.processes() {
            // Skip non-running processes (zombie/terminating)
            use sysinfo::ProcessStatus;
            if proc.status() != ProcessStatus::Run {
                continue;
            }

            // Double-check: only accept if memory usage > 0 (real alive process)
            if proc.memory() == 0 {
                continue;
            }

            let pid_u32 = pid.as_u32();

            #[cfg(windows)]
            {
                let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid_u32) };
                if handle.is_null() {
                    continue;
                }
                unsafe { CloseHandle(handle) };
            }

            let is_cc =
                proc.name().eq_ignore_ascii_case("claude.exe")
                || proc.cmd().iter().any(|s| s.contains("@anthropic-ai/claude-code"))
                || proc.cmd().iter().any(|s| s.contains("claude-code/cli.js"))
                || proc.exe().map(|p| p.to_string_lossy().to_lowercase().contains("claude")).unwrap_or(false);

            #[cfg(windows)]
            if proc.parent().map(|pp: sysinfo::Pid| pp.as_u32() == monitor_pid).unwrap_or(false) {
                continue;
            }
            if !is_cc {
                continue;
            }

            let work_dir = proc.cwd()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();

            if work_dir.as_os_str().is_empty() {
                continue;
            }

            if seen_pids.contains(&pid_u32) {
                continue;
            }
            seen_pids.insert(pid_u32);

            let id = Uuid::new_v4().to_string();
            let name = work_dir.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("CC-{}", pid_u32));

            let monitor = SessionMonitor::new(id.clone(), name.clone(), work_dir.clone(), Some(pid_u32));

            let _ = tx.blocking_send((monitor, id)).is_ok();
        }
    }).await;

    // Process discovered sessions
    while let Some((monitor, id)) = rx.recv().await {
        let pid_to_check = monitor.pid;
        {
            let existing = monitors.lock().unwrap();
            if existing.values().any(|m| m.lock().unwrap().pid == pid_to_check) {
                continue;
            }
        }

        let monitor = Arc::new(Mutex::new(monitor));

        spawn_monitor_loop(
            monitor.clone(),
            app.clone(),
            POLL_INTERVAL_MS,
            id.clone(),
            CPU_LOW_THRESHOLD,
            true,
            monitors.clone(),
        );

        {
            let mut monitors_guard = monitors.lock().unwrap();
            monitors_guard.insert(id.clone(), monitor.clone());
        }

        let infos: Vec<SessionInfo> = monitors.lock().unwrap().values()
            .map(|m| m.lock().unwrap().to_info())
            .collect();
        let _ = app.emit("session_update", &infos);
    }
}

#[tauri::command]
pub async fn launch_session(work_dir: String, app: AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();
    let work_path = PathBuf::from(&work_dir);
    if !work_path.exists() {
        return Err(format!("工作目录不存在: {}", work_dir));
    }

    let id = Uuid::new_v4().to_string();
    let name = work_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| work_dir.clone());

    {
        let monitors = state.monitors.lock().unwrap();
        if monitors.values().any(|m| m.lock().unwrap().work_dir == work_path) {
            return Err("该目录已在监控列表中".to_string());
        }
    }

    let monitor = Arc::new(Mutex::new(SessionMonitor::new(
        id.clone(),
        name,
        work_path.clone(),
        None,
    )));

    let shutdown_flag = monitor.lock().unwrap().stopped.clone();

    let mut pty_result = spawn_pty(&work_path, monitor.clone(), shutdown_flag)
        .map_err(|e| format!("启动 PTY 失败: {}", e))?;

    {
        let mut m = monitor.lock().unwrap();
        m.pid = Some(pty_result.pid);
        m.pty_write_tx = pty_result.write_tx.take();
        m.pty_child = Some(pty_result);
    }

    spawn_monitor_loop(
        monitor.clone(),
        state.app_handle.clone(),
        POLL_INTERVAL_MS,
        id.clone(),
        CPU_LOW_THRESHOLD,
        true,
        state.monitors.clone(),
    );

    {
        let mut monitors = state.monitors.lock().unwrap();
        monitors.insert(id.clone(), monitor.clone());
    }

    let infos: Vec<SessionInfo> = state.monitors.lock().unwrap().values()
        .map(|m| m.lock().unwrap().to_info())
        .collect();
    let _ = state.app_handle.emit("session_update", &infos);

    Ok(id)
}

#[tauri::command]
pub async fn rename_session(id: String, name: String, app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let infos: Vec<SessionInfo> = {
        let monitors = state.monitors.lock().unwrap();
        let monitor = monitors.get(&id).ok_or("会话不存在")?;
        monitor.lock().unwrap().name = name;
        monitors.values()
            .map(|m| m.lock().unwrap().to_info())
            .collect()
    };
    let _ = state.app_handle.emit("session_update", &infos);
    Ok(())
}

#[tauri::command]
pub async fn stop_session(id: String, app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let monitor = {
        let mut monitors = state.monitors.lock().unwrap();
        match monitors.remove(&id) {
            Some(m) => m,
            None => return Err("会话不存在".to_string()),
        }
    };
    // Signal shutdown so PTY reader thread exits
    monitor.lock().unwrap().stopped.store(true, Ordering::Relaxed);
    let _ = state.app_handle.emit("session_cleanup", &id);
    Ok(())
}

#[tauri::command]
pub async fn refresh_sessions(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    // Use mutex to prevent concurrent discovery
    let mut discovering = state.discovering.lock().unwrap();
    if *discovering {
        return Ok(());
    }
    *discovering = true;
    drop(discovering);

    let app_handle = state.app_handle.clone();
    let monitors = state.monitors.clone();
    let discovering = state.discovering.clone();
    tauri::async_runtime::spawn(async move {
        discover_sessions(app_handle, monitors).await;
        *discovering.lock().unwrap() = false;
    });
    Ok(())
}

#[tauri::command]
pub async fn get_sessions(app: AppHandle) -> Result<Vec<SessionInfo>, String> {
    let state = app.state::<AppState>();
    let monitors = state.monitors.lock().unwrap();
    let infos: Vec<SessionInfo> = monitors.values()
        .map(|m| m.lock().unwrap().to_info())
        .collect();
    Ok(infos)
}

#[tauri::command]
pub fn set_view_mode(mode: String, app: AppHandle) -> Result<(), String> {
    if mode != "card" && mode != "bar" {
        return Err("mode must be 'card' or 'bar'".to_string());
    }

    let main = app.get_webview_window("main");
    let bar = app.get_webview_window("bar");

    match mode.as_str() {
        "bar" => {
            if let Some(w) = main { let _ = w.hide(); }
            if let Some(w) = bar {
                // Position bar at top-center, below system taskbar
                if let Ok(Some(monitor)) = app.primary_monitor() {
                    let monitor_size = monitor.size();
                    let bar_size = w.outer_size().unwrap_or(PhysicalSize::new(800, 44));
                    let x = ((monitor_size.width as i32) - (bar_size.width as i32)) / 2;
                    let _ = w.set_position(PhysicalPosition::new(x.max(0), 40));
                }
                let _ = w.show();
            }
        }
        "card" => {
            if let Some(w) = bar { let _ = w.hide(); }
            if let Some(w) = main { let _ = w.show(); }
        }
        _ => {}
    }

    let _ = app.emit("view_mode_changed", &mode);
    Ok(())
}

#[tauri::command]
pub fn get_view_mode(app: AppHandle) -> Result<String, String> {
    let bar = app.get_webview_window("bar");
    if bar.map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false) {
        Ok("bar".to_string())
    } else {
        Ok("card".to_string())
    }
}

#[tauri::command]
pub fn respond_confirm(id: String, action: String, app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let monitors = state.monitors.lock().unwrap();
    let monitor = monitors.get(&id).ok_or("会话不存在")?;
    let write_tx = {
        let m = monitor.lock().unwrap();
        m.pty_write_tx.clone()
    };
    drop(monitors);

    let tx = write_tx.ok_or("该会话不支持写入 PTY")?;
    let line = crate::hooks_server::pty_line_for_action(action.as_str())
        .ok_or_else(|| "action 必须是 approve/allow 或 deny/reject".to_string())?;
    tx.send(line.as_bytes().to_vec())
        .map_err(|e| format!("写入 PTY 失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn register_hook(url: String, events: Vec<String>, app: AppHandle) -> Result<(), String> {
    use crate::hooks_server::HookRegistration;
    let state = app.state::<AppState>();
    let hook_tx = state.hook_tx.lock().unwrap();
    let tx = hook_tx.as_ref().ok_or("Hook server 未启动")?;
    tx.send(HookAction::Register {
        registration: HookRegistration { url, events },
    }).map_err(|e| format!("注册 hook 失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    let state = app.state::<AppState>();
    // Stop all PTY sessions — removing from HashMap drops PtyChild -> kills child process
    let monitors: Vec<_> = {
        let mut monitors = state.monitors.lock().unwrap();
        monitors.drain().collect()
    };
    for (_id, monitor) in monitors {
        monitor.lock().unwrap().stopped.store(true, Ordering::Relaxed);
        // Dropping monitor drops PtyChild -> PTY master closed -> child process killed
    }
    if let Some(bar) = app.get_webview_window("bar") {
        let _ = bar.hide();
    }
    app.exit(0);
}
