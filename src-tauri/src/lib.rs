// lib.rs — CC Monitor Tauri application

pub mod commands;
pub mod session;
pub mod state_machine;
pub mod pty_watcher;
pub mod state;
pub mod hooks_server;

use state::AppState;
use hooks_server::{HookAction, start_hook_server};
use tauri::{Manager, WindowEvent};

const HOOK_PORT: u16 = 4321;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = AppState::new(handle.clone());
            app.manage(state.clone());

            // Start the hook HTTP server in a background thread.
            // The server broadcasts confirm events to registered webhooks
            // and routes responses back via hook_tx -> hook_rx.
            let (hook_tx, hook_rx) = std::sync::mpsc::channel();
            {
                let mut tx = state.hook_tx.lock().unwrap();
                *tx = Some(hook_tx.clone());
            }
            let hook_manager = hooks_server::init_hook_manager();
            std::thread::spawn(move || {
                start_hook_server(hook_manager.clone(), hook_tx, HOOK_PORT);
            });

            // PTY write loop: receives actions from hook_rx and dispatches to PTY stdin.
            // PTY write loop: process HookAction from hook server.
            // Writes PTY responses (approve/deny) and broadcasts confirm events.
            let monitors = state.monitors.clone();
            let hook_manager = hooks_server::init_hook_manager();
            std::thread::spawn(move || {
                for action in hook_rx.iter() {
                    match action {
                        HookAction::PtyWrite { session_id, data } => {
                            let action_str = if data.first() == Some(&b'y') { "approve" } else { "deny" };
                            let write_tx = {
                                let monitors = monitors.lock().unwrap();
                                monitors.get(&session_id)
                                    .and_then(|m| m.lock().unwrap().pty_write_tx.clone())
                            };
                            if let Some(tx) = write_tx {
                                if tx.send(data).is_err() {
                                    eprintln!("[hooks] failed to send to PTY writer for session {}", session_id);
                                }
                            }
                            // Notify registered hooks of resolution
                            hook_manager.notify_confirm_resolved(&session_id, action_str);
                        }
                        HookAction::ConfirmStart { session_id, prompt } => {
                            hook_manager.notify_confirm_start(&session_id, &prompt);
                        }
                        HookAction::ConfirmNotify { session_id, prompt } => {
                            // CC is about to prompt user for confirm — update session state
                            if let Some(monitor) = monitors.lock().unwrap().get(&session_id) {
                                let mut m = monitor.lock().unwrap();
                                m.awaiting_confirm = true;
                                m.last_line = prompt.clone();
                            }
                            // Broadcast confirm_start to registered webhooks
                            hook_manager.notify_confirm_start(&session_id, &prompt);
                        }
                        HookAction::ConfirmNotifyByDir { cwd, prompt } => {
                            let cwd_normalized = cwd.trim_end_matches('\\');
                            let found = {
                                let guards = monitors.lock().unwrap();
                                guards.values().find(|m| {
                                    m.lock().unwrap().work_dir.to_string_lossy().trim_end_matches('\\') == cwd_normalized
                                }).cloned()
                            };
                            if let Some(monitor) = found {
                                let mut m = monitor.lock().unwrap();
                                m.awaiting_confirm = true;
                                m.last_line = prompt.clone();
                                let sid = m.id.clone();
                                drop(m);
                                hook_manager.notify_confirm_start(&sid, &prompt);
                            }
                        }
                        HookAction::ConfirmResolvedByDir { cwd } => {
                            let cwd_normalized = cwd.trim_end_matches('\\');
                            let found = {
                                let guards = monitors.lock().unwrap();
                                guards.values().find(|m| {
                                    m.lock().unwrap().work_dir.to_string_lossy().trim_end_matches('\\') == cwd_normalized
                                }).cloned()
                            };
                            if let Some(monitor) = found {
                                let mut m = monitor.lock().unwrap();
                                m.awaiting_confirm = false;
                            }
                        }
                        HookAction::Register { registration } => {
                            hook_manager.register(registration);
                        }
                        HookAction::Unregister { url } => {
                            hook_manager.unregister(&url);
                        }
                    }
                }
            });

            // Auto-discover running Claude Code processes (run async in background)
            let app_handle = handle.clone();
            let monitors = state.monitors.clone();
            {
                *state.discovering.lock().unwrap() = true;
            }
            let discovering = state.discovering.clone();
            tauri::async_runtime::spawn(async move {
                commands::discover_sessions(app_handle, monitors).await;
                *discovering.lock().unwrap() = false;
            });

            // Periodic re-discovery every 10 seconds to catch CC processes started after Monitor
            let app_handle2 = handle.clone();
            let monitors2 = state.monitors.clone();
            let discovering2 = state.discovering.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    let should_discover = {
                        let mut guard = discovering2.lock().unwrap();
                        if *guard {
                            false
                        } else {
                            *guard = true;
                            true
                        }
                    };
                    if !should_discover {
                        continue;
                    }
                    commands::discover_sessions(app_handle2.clone(), monitors2.clone()).await;
                    *discovering2.lock().unwrap() = false;
                }
            });

            if let Some(bar) = app.get_webview_window("bar") {
                let _ = bar.hide();
            }

            // When main window close is requested, prevent it.
            // Frontend will show confirmation and call exit_app if confirmed.
            if let Some(main) = app.get_webview_window("main") {
                main.on_window_event(|event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::launch_session,
            commands::stop_session,
            commands::rename_session,
            commands::get_sessions,
            commands::refresh_sessions,
            commands::set_view_mode,
            commands::get_view_mode,
            commands::exit_app,
            commands::respond_confirm,
            commands::register_hook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
