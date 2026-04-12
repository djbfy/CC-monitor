// state.rs — Application state

use std::sync::{Arc, Mutex};
use crate::session::SessionMonitor;
use tauri::AppHandle;

#[derive(Clone)]
pub struct AppState {
    pub app_handle: AppHandle,
    pub monitors: Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<SessionMonitor>>>>>,
    pub discovering: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            monitors: Arc::new(Mutex::new(std::collections::HashMap::new())),
            discovering: Arc::new(Mutex::new(false)),
        }
    }
}
