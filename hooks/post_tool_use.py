#!/usr/bin/env python3
import json
import sys
import urllib.request
import os
from datetime import datetime

LOG_BASE = os.environ.get("CC_MONITOR_LOG", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "logs"))
LOG_DIR = os.path.abspath(LOG_BASE)
os.makedirs(LOG_DIR, exist_ok=True)

def log(msg):
    ts = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    log_file = os.path.join(LOG_DIR, "hook_calls.log")
    with open(log_file, "a", encoding="utf-8") as f:
        f.write(f"[{ts}] [PostToolUse] {msg}\n")

PORT = int(os.environ.get("CC_MONITOR_PORT", "4321"))

try:
    data = json.load(sys.stdin)
except Exception as e:
    log(f"Failed to parse stdin: {e}")
    sys.exit(0)

session_id = data.get("session_id", "")
cwd = os.getcwd()

payload = json.dumps({
    "session_id": session_id,
    "cwd": cwd,
    "action": "resolved",
}).encode()

log(f"PostToolUse cwd={cwd}")

req = urllib.request.Request(
    f"http://127.0.0.1:{PORT}/hook/confirm-resolved",
    data=payload,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    response = urllib.request.urlopen(req, timeout=5)
    log(f"confirm-resolved OK {response.status}")
except Exception as e:
    log(f"Error: {e}")

sys.exit(0)
