// state_machine.rs — State evaluation logic

use crate::session::{SessionMonitor, SessionState};
use regex::Regex;
use once_cell::sync::Lazy;
use sysinfo::System;

// Idle detection threshold — must match CPU_LOW_THRESHOLD in commands.rs
const CPU_LOW_THRESHOLD: f32 = 0.5;

// Confirm patterns — Claude Code专用确认提示
static CONFIRM_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"(?i)\[y/N\]",
        r"(?i)\[Y/n\]",
        r"(?i)do you want to",
        r"(?i)are you sure",
        r"(?i)continue\?",
        r"(?i)proceed\?",
        r"(?i)overwrite\?",
        r"(?i)press enter to",
    ]
    .iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
});

// Idle patterns — 含 Windows cmd/PowerShell 提示符
static IDLE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"(?i)what would you like",
        r"(?i)how can i help",
        // Unix shell
        r"^\s*>\s*$",
        // Windows cmd
        r"^[A-Z]:\\.*>$",
        // Windows PowerShell (simplified)
        r"^PS [A-Z]:\\.*>$",
        // PowerShell with prefix (e.g. "[some stuff] PS C:\>")
        r"^\[.*\]\s+PS\s+[A-Z]:\\.*>$",
    ]
    .iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
});

/// Evaluate the state of a session.
/// Priority: Offline > Confirm > Idle > Running
pub fn evaluate_state(
    monitor: &SessionMonitor,
    sys: &System,
    confirm_match: bool,
    idle_match: bool,
    cpu_low_consecutive: u32,
    awaiting_confirm: bool,
) -> SessionState {
    // Priority 1: Offline — process does not exist
    if monitor.pid.is_none() {
        return SessionState::Offline;
    }

    let pid = sysinfo::Pid::from_u32(monitor.pid.unwrap());
    if !sys.process(pid).is_some() {
        return SessionState::Offline;
    }

    // Priority 2: Confirm — pattern match OR awaiting_confirm from PTY reader
    // awaiting_confirm handles the case where user input overwrites the prompt in PTY tail mode
    if confirm_match || awaiting_confirm {
        return SessionState::Confirm;
    }

    // Priority 3: Idle
    let has_pty_output = !monitor.last_line.is_empty();
    if has_pty_output {
        // With PTY output: silent > 5s && idle pattern && CPU low
        let silent_secs = monitor.last_output.elapsed().as_secs_f64();
        if monitor.cpu_percent < CPU_LOW_THRESHOLD && silent_secs > 5.0 && idle_match {
            return SessionState::Idle;
        }
    } else {
        // Without PTY output (auto-discovered): CPU < 0.5% for 5+ consecutive polls
        if cpu_low_consecutive >= 5 {
            return SessionState::Idle;
        }
    }

    // Priority 4: Running — everything else
    SessionState::Running
}

/// Check if last line matches a confirm pattern
pub fn matches_confirm(last_line: &str) -> bool {
    CONFIRM_PATTERNS.iter().any(|p| p.is_match(last_line))
}

/// Check if last line matches an idle pattern
pub fn matches_idle(last_line: &str) -> bool {
    IDLE_PATTERNS.iter().any(|p| p.is_match(last_line))
}

/// Get CPU percent for a process (does NOT refresh — caller decides when to refresh)
pub fn get_cpu_percent(sys: &mut System, pid: u32) -> f32 {
    let spid = sysinfo::Pid::from_u32(pid);
    sys.process(spid)
        .map(|p| p.cpu_usage())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_patterns() {
        assert!(matches_confirm("Continue? [y/N]"));
        assert!(matches_confirm("Do you want to proceed?"));
        assert!(matches_confirm("OVERWRITE? [y/N]"));
        assert!(!matches_confirm("Running tsc --noEmit"));
    }

    #[test]
    fn test_idle_patterns_unix() {
        assert!(matches_idle("what would you like me to do"));
        assert!(matches_idle("How can I help?"));
        assert!(matches_idle("   >   "));
    }

    #[test]
    fn test_idle_patterns_windows() {
        assert!(matches_idle("D:\\Projects>"));
        assert!(matches_idle("PS D:\\Projects>"));
        assert!(matches_idle("[some stuff] PS C:\\>"));
    }

    #[test]
    fn test_offline_priority() {
        let sys = System::new_all();
        let monitor = SessionMonitor {
            id: "test".into(),
            name: "test".into(),
            work_dir: std::path::PathBuf::from("D:\\test"),
            pid: None,
            last_output: std::time::Instant::now(),
            last_line: String::new(),
            cpu_percent: 0.0,
            state: SessionState::Offline,
            started_at: 0,
            pty_child: None,
            has_seen_output: false,
            stopped: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            cpu_low_consecutive: 0,
        };
        let state = evaluate_state(&monitor, &sys, false, false, 0);
        assert_eq!(state, SessionState::Offline);
    }
}
