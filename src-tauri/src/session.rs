// session.rs — Session data structures and serialization

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc::Sender};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Running,
    Confirm,
    Idle,
    Offline,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Running => "running",
            SessionState::Confirm => "confirm",
            SessionState::Idle => "idle",
            SessionState::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub work_dir: String,
    pub pid: Option<u32>,
    pub state: SessionState,
    pub silent_secs: f64,
    pub cpu_percent: f32,
    pub last_line: String,
    pub started_at: u64,
}

pub struct SessionMonitor {
    pub id: String,
    pub name: String,
    pub work_dir: PathBuf,
    pub pid: Option<u32>,
    pub last_output: Instant,
    pub last_line: String,
    pub cpu_percent: f32,
    pub state: SessionState,
    pub started_at: u64,
    /// PTY handle — must stay alive to keep PTY master side open.
    /// Never read from outside this module; only stored to prevent it from being dropped.
    pub(crate) pty_child: Option<crate::pty_watcher::PtyChild>,
    /// Sender for writing to PTY stdin. If Some, sending Vec<u8> through it writes to the PTY.
    pub pty_write_tx: Option<Sender<Vec<u8>>>,
    /// Whether this session has ever received PTY output.
    /// Only relevant for launched sessions (auto-discovered sessions always have PTY output = false).
    pub has_seen_output: bool,
    /// Stop flag — when true, monitoring loop exits
    pub stopped: Arc<AtomicBool>,
    /// Consecutive low-CPU readings (for idle detection without PTY)
    pub cpu_low_consecutive: u32,
    /// PTY reader saw a confirm prompt — stays true until user input is received
    pub awaiting_confirm: bool,
}

impl SessionMonitor {
    pub fn new(id: String, name: String, work_dir: PathBuf, pid: Option<u32>) -> Self {
        Self {
            id,
            name,
            work_dir,
            pid,
            last_output: Instant::now(),
            last_line: String::new(),
            cpu_percent: 0.0,
            state: SessionState::Running,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            pty_child: None,
            pty_write_tx: None,
            stopped: Arc::new(AtomicBool::new(false)),
            cpu_low_consecutive: 0,
            has_seen_output: false,
            awaiting_confirm: false,
        }
    }
}

impl SessionMonitor {
    pub fn to_info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            work_dir: self.work_dir.to_string_lossy().to_string(),
            pid: self.pid,
            state: self.state,
            // silent_secs only makes sense for sessions with PTY output
            silent_secs: if self.has_seen_output {
                self.last_output.elapsed().as_secs_f64()
            } else {
                0.0
            },
            cpu_percent: self.cpu_percent,
            last_line: self.last_line.clone(),
            started_at: self.started_at,
        }
    }
}
