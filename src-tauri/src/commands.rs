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
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use uuid::Uuid;

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

        for (pid, proc) in sys.processes() {
            let cmd_str = proc.cmd().iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" ");

            if !cmd_str.contains("@anthropic-ai/claude-code") {
                continue;
            }

            let pid_u32 = pid.as_u32();
            if seen_pids.contains(&pid_u32) {
                continue;
            }
            seen_pids.insert(pid_u32);

            let work_dir = proc.cwd()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();

            if work_dir.as_os_str().is_empty() {
                continue;
            }

            let id = Uuid::new_v4().to_string();
            let name = work_dir.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("CC-{}", pid_u32));

            let monitor = SessionMonitor::new(id.clone(), name, work_dir, Some(pid_u32));

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
pub async fn launch_session(work_dir: String, state: State<'_, AppState>) -> Result<String, String> {
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

    let pty_result = spawn_pty(&work_path, monitor.clone(), shutdown_flag)
        .map_err(|e| format!("启动 PTY 失败: {}", e))?;

    {
        let mut m = monitor.lock().unwrap();
        m.pid = Some(pty_result.pid);
        m.pty_child = Some(pty_result);
    }

    spawn_monitor_loop(
        monitor.clone(),
        state.inner().app_handle.clone(),
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
    let _ = state.inner().app_handle.emit("session_update", &infos);

    Ok(id)
}

#[tauri::command]
pub async fn stop_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
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
pub async fn refresh_sessions(state: State<'_, AppState>) -> Result<(), String> {
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
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
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
            if let Some(w) = bar { let _ = w.show(); }
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
