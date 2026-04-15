#!/usr/bin/env python3
import json
import sys
import urllib.request
import os
from datetime import datetime

# Use environment variable or default to ./logs relative to project root
LOG_BASE = os.environ.get("CC_MONITOR_LOG", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "logs"))
LOG_DIR = os.path.abspath(LOG_BASE)
os.makedirs(LOG_DIR, exist_ok=True)

def log(msg):
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    log_file = os.path.join(LOG_DIR, "hook_calls.log")
    with open(log_file, "a", encoding="utf-8") as f:
        f.write(f"[{ts}] {msg}\n")

PORT = int(os.environ.get("CC_MONITOR_PORT", "4321"))

try:
    data = json.load(sys.stdin)
except Exception as e:
    log(f"Failed to parse stdin: {e}")
    sys.exit(0)

hook_event = data.get("hook_event_name", "")
session_id = data.get("session_id", "")
cwd = data.get("cwd", "")
message = data.get("message", "")

payload = json.dumps({
    "session_id": session_id,
    "cwd": cwd,
    "status": "confirm",
    "tool": hook_event,
    "message": message[:200] if message else "",
    "confirm_id": f"{hook_event}-{session_id}",
}).encode()

req = urllib.request.Request(
    f"http://127.0.0.1:{PORT}/hook/confirm",
    data=payload,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    response = urllib.request.urlopen(req, timeout=5)
    log(f"OK {response.status}")
except Exception as e:
    log(f"Error: {e}")

sys.exit(0)
