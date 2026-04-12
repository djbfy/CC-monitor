// pty_watcher.rs — PTY spawning and ANSI stripping

use crate::session::SessionMonitor;
use portable_pty::{CommandBuilder, PtySize, Child};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::mem::ManuallyDrop;

/// Result of spawning a PTY.
/// Both `child` and `master` must be kept alive — dropping `master` closes the PTY
/// and causes the child process to exit on Windows ConPTY.
pub struct PtyChild {
    /// Child process handle — used to get PID and wait for exit
    pub child: Box<dyn Child + Send + Sync>,
    /// PID of the spawned process
    pub pid: u32,
    /// PTY master side — must be kept alive to keep the PTY open.
    /// Stored here to prevent it from being dropped; dropping the master closes ConPTY.
    #[allow(dead_code)]
    master: ManuallyDrop<Box<dyn portable_pty::MasterPty + Send>>,
}

/// Spawn a PTY for a Claude Code session.
/// The monitor is used to record PTY output so the monitoring loop can detect
/// user activity via CC's terminal output.
pub fn spawn_pty(
    work_dir: &std::path::Path,
    monitor: Arc<Mutex<SessionMonitor>>,
    shutdown_flag: Arc<AtomicBool>,
) -> Result<PtyChild, String> {
    let pair = portable_pty::native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("failed to open PTY: {}", e))?;

    let node_exec = std::env::var("CLAUDE_CODE_NODE_PATH")
        .unwrap_or_else(|_| "D:\\CODE\\NodeJs\\node.exe".to_string());
    let cli_path = std::env::var("CLAUDE_CODE_CLI_PATH")
        .unwrap_or_else(|_| "D:\\CODE\\NodeJs\\node_global\\node_modules\\@anthropic-ai\\claude-code\\cli.js".to_string());

    let mut cmd = if cfg!(windows) {
        let mut c = CommandBuilder::new(&node_exec);
        c.arg(&cli_path);
        c
    } else {
        let c = CommandBuilder::new("npx");
        c
    };
    cmd.cwd(work_dir);
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    if cfg!(windows) {
        let bash_path = std::env::var("CLAUDE_CODE_GIT_BASH_PATH")
            .unwrap_or_else(|_| "C:\\Program Files\\Git\\bin\\bash.exe".to_string());
        cmd.env("CLAUDE_CODE_GIT_BASH_PATH", &bash_path);
    }

    let child = pair.slave.spawn_command(cmd)
        .map_err(|e| format!("failed to spawn CC: {}", e))?;

    let pid = child.process_id().unwrap_or(0);

    // Reader thread: read PTY output and update monitor so monitoring loop can
    // detect user activity (CC produces terminal output when user types).
    // Respects shutdown_flag to exit cleanly when the session is stopped.
    let mut reader = pair.master.try_clone_reader()
        .map_err(|e| format!("failed to clone PTY reader: {}", e))?;
    let monitor_clone = monitor.clone();
    let shutdown = shutdown_flag.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut line_buf = Vec::new();
        const MAX_LINE_BUF: usize = 65536;
        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            // Use timeout read to allow checking shutdown_flag periodically
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let stripped = strip_ansi_escapes::strip(&buf[..n]);
                    line_buf.extend_from_slice(&stripped);
                    // Force-flush if line_buf grows too large (no newlines seen)
                    if line_buf.len() > MAX_LINE_BUF {
                        if let Some(last) = line_buf.len().checked_sub(MAX_LINE_BUF / 2) {
                            let overflow = line_buf[last..].to_vec();
                            line_buf = overflow;
                        }
                    }
                    while let Some(pos) = line_buf.iter().position(|&b| b == b'\n') {
                        let line = String::from_utf8_lossy(&line_buf[..=pos]).trim().to_string();
                        line_buf.drain(..=pos);
                        if !line.is_empty() {
                            let mut m = monitor_clone.lock().unwrap();
                            m.last_line = line.clone();
                            m.last_output = Instant::now();
                            m.has_seen_output = true;
                        }
                    }
                }
            }
        }
    });

    Ok(PtyChild {
        child,
        pid,
        master: ManuallyDrop::new(pair.master),
    })
}

/// Strip ANSI escape sequences from a string
pub fn strip_ansi(input: &str) -> String {
    let bytes = strip_ansi_escapes::strip(input.as_bytes());
    String::from_utf8_lossy(&bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("\x1b[2J\x1b[H"), "");
        assert_eq!(strip_ansi("\x1b[1;32mRunning\x1b[0m  tsc"), "Running  tsc");
    }
}
