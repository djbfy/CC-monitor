// state.rs — Application state

use std::sync::{Arc, Mutex, mpsc};
use crate::session::SessionMonitor;
use crate::hooks_server::HookAction;
use tauri::AppHandle;

#[derive(Clone)]
pub struct AppState {
    pub app_handle: AppHandle,
    pub monitors: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<SessionMonitor>>>>>,
    pub discovering: Arc<Mutex<bool>>,
    /// Channel sender for routing HookActions (PtyWrite, Register, Unregister).
    pub hook_tx: Arc<Mutex<Option<mpsc::Sender<HookAction>>>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            monitors: Arc::new(Mutex::new(std::collections::HashMap::new())),
            discovering: Arc::new(Mutex::new(false)),
            hook_tx: Arc::new(Mutex::new(None)),
        }
    }
}
