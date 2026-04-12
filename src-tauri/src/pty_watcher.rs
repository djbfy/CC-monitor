// pty_watcher.rs — PTY spawning and ANSI stripping

use crate::session::SessionMonitor;
use crate::state_machine::matches_confirm;
use portable_pty::{CommandBuilder, PtySize, Child};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
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

/// Cached node.exe path — resolved once per app lifetime using LazyLock.
static NODE_PATH: LazyLock<PathBuf, fn() -> PathBuf> = LazyLock::new(|| {
    let path_var = std::env::var_os("PATH")
        .expect("PATH environment variable not set");
    for dir in std::env::split_paths(&path_var) {
        let node = dir.join("node.exe");
        let candidate = if node.exists() { node } else { dir.join("node") };
        if candidate.exists() {
            return candidate;
        }
    }
    panic!("node.exe not found in PATH. Please install Node.js and ensure it is in your PATH.");
});

/// Find node.exe path. Cached — only searches PATH once.
fn find_node() -> &'static PathBuf {
    &NODE_PATH
}

/// Find the Claude Code CLI script by asking node to resolve it.
/// Falls back to scanning npm global modules directory.
fn find_cc_cli() -> Result<PathBuf, String> {
    let output = Command::new(find_node())
        .args(["-e", "console.log(require.resolve('@anthropic-ai/claude-code/cli.js'))"])
        .output();

    if let Ok(out) = output {
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    // Fallback: find npm global root and check there
    let output = Command::new("cmd")
        .args(["/c", "npm", "root", "-g"])
        .output();

    if let Ok(out) = output {
        let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let cli = PathBuf::from(&root).join("@anthropic-ai/claude-code/cli.js");
        return Ok(cli);
    }

    Err(
        "Claude Code CLI not found. Install it with: npm install -g @anthropic-ai/claude-code".to_string(),
    )
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

    let node_exec = find_node();
    let cli_path = find_cc_cli()?;

    let mut cmd = if cfg!(windows) {
        let mut c = CommandBuilder::new(node_exec);
        c.arg(&cli_path);
        c
    } else {
        let c = CommandBuilder::new("npx");
        c
    };
    cmd.cwd(work_dir);
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());

    let child = pair.slave.spawn_command(cmd)
        .map_err(|e| format!("failed to spawn CC: {}. Ensure Claude Code CLI is installed.", e))?;

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
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    // Only strip ANSI escapes if the buffer contains ESC (0x1b)
                    let chunk = if buf[..n].contains(&0x1b) {
                        strip_ansi_escapes::strip(&buf[..n])
                    } else {
                        buf[..n].to_vec()
                    };
                    line_buf.extend_from_slice(&chunk);
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
                            if line.starts_with('\x1b') {
                                continue;
                            }

                            let mut m = monitor_clone.lock().unwrap();
                            if matches_confirm(&line) {
                                m.awaiting_confirm = true;
                            }
                            let is_user_keypress = line.starts_with('\x1b') || line == "\r";
                            if !is_user_keypress && !matches_confirm(&line) {
                                m.awaiting_confirm = false;
                            }
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
