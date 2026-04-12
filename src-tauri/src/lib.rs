// lib.rs — CC Monitor Tauri application

pub mod commands;
pub mod session;
pub mod state_machine;
pub mod pty_watcher;
pub mod state;

use state::AppState;
use tauri::{Manager, WindowEvent};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = AppState::new(handle.clone());
            app.manage(state.clone());

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

            if let Some(bar) = app.get_webview_window("bar") {
                let _ = bar.hide();
            }

            // When main window close is requested, prevent it.
            // Frontend will show confirmation and call exit_app if confirmed.
            if let Some(main) = app.get_webview_window("main") {
                main.on_window_event(|event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent close — frontend shows confirmation dialog,
                        // then calls exit_app to actually close and kill sessions
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
