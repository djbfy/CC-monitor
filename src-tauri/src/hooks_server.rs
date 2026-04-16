// hooks_server.rs — Built-in HTTP hook server for bidirectional CC confirm control
//
// HTTP server on localhost:4321.
// External tools register webhooks via POST /hook/register
// and receive confirm events as HTTP POST callbacks.
// Responses come back via POST /hook/respond.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::io::Write as IoWrite;
use tiny_http::{Response, Server};

// === File Logging ===

fn log_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("logs"))
        .unwrap_or_else(|| PathBuf::from("logs"))
}

fn hook_log(msg: &str) {
    let dir = log_dir();
    let _ = fs::create_dir_all(&dir);
    let log_file = dir.join("hooks.log");
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{}] {}\n", ts, msg);
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    eprintln!("[hook] {}", msg);
}

fn hook_log_json(path: &str, body: &[u8]) {
    hook_log(&format!("POST {} | body: {}", path, String::from_utf8_lossy(body)));
}

/// Hook event sent to registered external webhooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub session_id: String,
    pub prompt: Option<String>,
    pub action: Option<String>,
}

/// Payload from external tool registering a webhook.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookRegistration {
    pub url: String,
    pub events: Vec<String>,
}

/// Manages registered webhook URLs and sends confirm events to them.
#[derive(Clone)]
pub struct HookManager {
    registrations: Arc<Mutex<Vec<HookRegistration>>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&self, reg: HookRegistration) {
        let mut regs = self.registrations.lock().unwrap();
        regs.retain(|r| r.url != reg.url);
        regs.push(reg);
    }

    pub fn unregister(&self, url: &str) {
        let mut regs = self.registrations.lock().unwrap();
        regs.retain(|r| r.url != url);
    }

    pub fn list(&self) -> Vec<HookRegistration> {
        self.registrations.lock().unwrap().clone()
    }

    fn notify_webhooks(&self, event_type: &str, body_json: &str) {
        let regs = self.registrations.lock().unwrap();
        for reg in regs.iter() {
            if reg.events.iter().any(|e| e == event_type) {
                let url = reg.url.clone();
                let body = body_json.to_string();
                thread::spawn(move || {
                    let _ = WEBHOOK_CLIENT
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .body(body)
                        .send();
                });
            }
        }
    }

    pub fn notify_confirm_start(&self, session_id: &str, prompt: &str) {
        let body = serde_json::to_string(&HookEvent {
            event_type: "confirm_start".to_string(),
            session_id: session_id.to_string(),
            prompt: Some(prompt.to_string()),
            action: None,
        }).unwrap_or_default();
        self.notify_webhooks("confirm_start", &body);
    }

    pub fn notify_confirm_resolved(&self, session_id: &str, action: &str) {
        let body = serde_json::to_string(&HookEvent {
            event_type: "confirm_resolved".to_string(),
            session_id: session_id.to_string(),
            prompt: None,
            action: Some(action.to_string()),
        }).unwrap_or_default();
        self.notify_webhooks("confirm_resolved", &body);
    }
}

/// Map confirm action string to PTY input bytes.
/// Returns None for unknown actions.
pub fn pty_line_for_action(action: &str) -> Option<&'static str> {
    match action {
        "approve" | "allow" => Some("y\r\n"),
        "deny" | "reject" => Some("\r\n"),
        _ => None,
    }
}

/// Shared HTTP client for webhook calls — created once with connection pooling.
static WEBHOOK_CLIENT: std::sync::LazyLock<reqwest::blocking::Client, fn() -> reqwest::blocking::Client> =
    std::sync::LazyLock::new(|| {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("failed to build webhook client")
    });

/// Messages sent FROM the HTTP server TO the PTY writer loop.
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Write raw bytes to PTY stdin for a session (approve/deny).
    PtyWrite {
        session_id: String,
        data: Vec<u8>,
    },
    /// Register a new webhook URL.
    Register {
        registration: HookRegistration,
    },
    /// Unregister a webhook URL.
    Unregister {
        url: String,
    },
    /// Notify that a confirm prompt was detected (broadcast to webhooks).
    ConfirmStart {
        session_id: String,
        prompt: String,
    },
    /// CC itself calls this to notify the monitor that it is about to prompt user for confirmation.
    ConfirmNotify {
        session_id: String,
        prompt: String,
    },
    /// CC hook notifies by cwd (work_dir) instead of session_id.
    ConfirmNotifyByDir {
        cwd: String,
        prompt: String,
    },
    /// CC confirms resolved — clear awaiting_confirm state.
    ConfirmResolvedByDir {
        cwd: String,
    },
}

/// Global hook manager — set once at startup by lib.rs.
pub static HOOK_MANAGER: std::sync::LazyLock<HookManager, fn() -> HookManager> =
    std::sync::LazyLock::new(|| HookManager::new());

/// Initialize the global hook manager. Call once from lib.rs setup.
pub fn init_hook_manager() -> HookManager {
    HookManager::new()
}

/// Start the HTTP hook server in a background thread.
/// - `manager`: HookManager shared with HTTP handlers (read-only for broadcasting)
/// - `hook_tx`: channel to send actions (PtyWrite, Register, Unregister) back to the main loop
pub fn start_hook_server(
    manager: HookManager,
    hook_tx: std::sync::mpsc::Sender<HookAction>,
    port: u16,
) {
    let addr = format!("0.0.0.0:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            hook_log(&format!("FAILED to bind {}: {}", addr, e));
            return;
        }
    };
    hook_log(&format!("Hook server listening on http://{}", addr));

    loop {
        // Use recv_timeout for non-blocking request handling
        let request = server.recv_timeout(std::time::Duration::from_millis(50));
        if let Ok(Some(mut request)) = request {
            let path = request.url().to_string();
            let method = request.method().to_string();
            let manager = manager.clone();
            let hook_tx = hook_tx.clone();

            let mut body = Vec::new();
            let _ = request.as_reader().read_to_end(&mut body);
            let body_str = String::from_utf8_lossy(&body);
            hook_log_json(&path, &body);

            let response = match (method.as_str(), path.as_str()) {
                ("POST", "/hook/register") => {
                    if let Ok(reg) = serde_json::from_str::<HookRegistration>(&body_str) {
                        manager.register(reg.clone());
                        let _ = hook_tx.send(HookAction::Register { registration: reg });
                    }
                    Response::from_string("ok")
                }
                ("POST", "/hook/unregister") => {
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&body_str) {
                        if let Some(url) = payload.get("url").and_then(|v| v.as_str()) {
                            manager.unregister(url);
                            let _ = hook_tx.send(HookAction::Unregister { url: url.to_string() });
                        }
                    }
                    Response::from_string("ok")
                }
                ("POST", "/hook/respond") => {
                    // External tool responds to a confirm
                    #[derive(Deserialize)]
                    struct Resp { session_id: String, action: String }
                    if let Ok(resp) = serde_json::from_str::<Resp>(&body_str) {
                        let line = pty_line_for_action(resp.action.as_str()).unwrap_or("\r\n");
                        let _ = hook_tx.send(HookAction::PtyWrite {
                            session_id: resp.session_id,
                            data: line.as_bytes().to_vec(),
                        });
                    }
                    Response::from_string("ok")
                }
                ("POST", "/hook/notify") => {
                    // CC calls this when it is about to prompt the user for confirmation.
                    // This allows the monitor to mark a session as awaiting user confirm.
                    #[derive(Deserialize)]
                    struct NotifyPayload { session_id: String, prompt: Option<String> }
                    if let Ok(payload) = serde_json::from_str::<NotifyPayload>(&body_str) {
                        let _ = hook_tx.send(HookAction::ConfirmNotify {
                            session_id: payload.session_id,
                            prompt: payload.prompt.unwrap_or_default(),
                        });
                    }
                    Response::from_string("ok")
                }
                ("POST", "/hook/confirm") => {
                    // CC's pre_tool_use hook calls this to wait for user confirm.
                    // Blocks until monitor responds with allow/deny.
                    hook_log(&format!("=== CONFIRM RECEIVED ==="));
                    #[derive(Deserialize)]
                    struct ConfirmPayload {
                        cwd: Option<String>,
                        tool: Option<String>,
                        message: Option<String>,
                    }
                    #[derive(Serialize)]
                    struct ConfirmResp { action: String }
                    if let Ok(payload) = serde_json::from_str::<ConfirmPayload>(&body_str) {
                        hook_log(&format!("  cwd={:?} tool={:?} msg={:?}", payload.cwd, payload.tool, payload.message));
                        let tool = payload.tool.clone().unwrap_or_default();
                        let msg = payload.message.clone().unwrap_or_default();
                        let prompt = format!("[{}] {}", tool, msg);
                        // Use cwd to find session in monitors (work_dir match)
                        let _ = hook_tx.send(HookAction::ConfirmNotifyByDir {
                            cwd: payload.cwd.unwrap_or_default(),
                            prompt,
                        });
                        let resp = serde_json::to_string(&ConfirmResp { action: "allow".to_string() }).unwrap_or_default();
                        Response::from_string(resp)
                            .with_header(
                                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                            )
                    } else {
                        hook_log(&format!("  FAILED to parse confirm payload"));
                        Response::from_string("{\"action\":\"allow\"}")
                            .with_header(
                                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                            )
                    }
                }
                ("POST", "/hook/confirm-resolved") => {
                    hook_log(&format!("=== CONFIRM_RESOLVED RECEIVED ==="));
                    #[derive(Deserialize)]
                    struct ResolvedPayload { session_id: String, cwd: Option<String> }
                    if let Ok(payload) = serde_json::from_str::<ResolvedPayload>(&body_str) {
                        hook_log(&format!("  session_id={} cwd={:?}", payload.session_id, payload.cwd));
                        let _ = hook_tx.send(HookAction::ConfirmResolvedByDir {
                            cwd: payload.cwd.unwrap_or_default(),
                        });
                    }
                    Response::from_string("ok")
                }
                ("GET", "/hooks") => {
                    let list = manager.list();
                    Response::from_string(serde_json::to_string(&list).unwrap_or_default())
                        .with_header(
                            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()
                        )
                }
                ("GET", "/health") | ("GET", "/") => Response::from_string("ok"),
                _ => Response::from_string("not found").with_status_code(404),
            };

            let _ = request.respond(response);
        }

        thread::sleep(std::time::Duration::from_millis(50));
    }
}
