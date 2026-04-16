# CC Monitor

桌面端 Claude Code 会话监控工具（Windows）。



卡片模式

![407345ac-dfee-4f70-b014-9600e2e96de5](file:///C:/Users/12520/Pictures/Typedown/407345ac-dfee-4f70-b014-9600e2e96de5.png)

顶栏模式

![4c612e20-eaa9-4b6c-ae12-073f9b42c20d](file:///C:/Users/12520/Pictures/Typedown/4c612e20-eaa9-4b6c-ae12-073f9b42c20d.png)

## 功能特性

- **会话发现**：自动扫描并监控系统中正在运行的 Claude Code 进程
- **会话启动**：通过目录启动新的 Claude Code 会话
- **实时状态**：监控 CPU、活跃时长、输出状态
- **双向 Hook**：CC 确认时实时通知 Monitor，选 Yes/No 均能正确恢复状态
- **卡片视图**：网格展示所有会话，支持双击重命名
- **顶栏模式**：紧凑的 Always-On-Top 顶栏，适合沉浸式工作流
- **关闭确认**：退出时确认，避免误操作
- **跨设备便携**：无需安装，复制即用，动态发现 Node.js 和 Claude Code CLI

## 系统要求

- Windows 10/11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（Windows 11 已内置）
- Node.js 18+（Claude Code CLI 需要）

## 下载使用

下载 `cc-monitor-portable.zip`，解压后双击 `cc-monitor.exe` 即可运行。

## 配置 Hook（让 CC 自动通知 Monitor）

CC Monitor 内置 HTTP 钩子服务器（默认端口 4321），需要配置 Claude Code 的 hooks 来触发通知。

### 1. 复制 Hook 脚本

将 `hooks/` 目录下的脚本复制到 CC 的 hooks 目录：

```
CC-Monitor/hooks/  →  ~/.claude/hooks/
```

### 2. 合并 settings.json

将以下配置合并到 `~/.claude/settings.json` 的 `hooks` 字段中：

```json
{
  "hooks": {
    "Notification": [{ "hooks": [{ "type": "command", "command": "python <CC-MONITOR-PATH>/hooks/notify_app.py" }] }],
    "PermissionRequest": [{ "hooks": [{ "type": "command", "command": "python <CC-MONITOR-PATH>/hooks/notify_app.py" }] }],
    "PostToolUse": [{ "hooks": [{ "type": "command", "command": "python <CC-MONITOR-PATH>/hooks/post_tool_use.py" }] }],
    "Stop": [{ "hooks": [{ "type": "command", "command": "python <CC-MONITOR-PATH>/hooks/stop_hook.py" }] }]
  }
}
```

将 `<CC-MONITOR-PATH>` 替换为实际的 CC Monitor 路径，例如：

- Windows: `C:/cc-monitor/hooks`
- Linux/Mac: `/home/user/cc-monitor/hooks`

### 3. 重启 CC

修改 `settings.json` 后需要完全退出 CC（`/exit`），然后重新启动 CC。

### 4. 验证

启动 CC Monitor，然后让 CC 触发一个确认请求（如执行需要权限的操作）。观察 CC Monitor 界面：

- CC 请求确认时 → Monitor 显示"待确认"状态
- 选 Yes 后 → Monitor 立即恢复"运行中"
- 选 No 后 → Monitor 立即恢复"运行中"

### 环境变量

Hook 脚本支持以下环境变量自定义：

| 变量                | 默认值      | 说明                |
| ----------------- | -------- | ----------------- |
| `CC_MONITOR_PORT` | `4321`   | Monitor HTTP 服务端口 |
| `CC_MONITOR_LOG`  | `./logs` | 日志目录（相对于项目根目录）    |

示例：

```bash
# 启动 CC Monitor 时指定端口
CC_MONITOR_PORT=5000 ./cc-monitor.exe

# Hook 日志单独存放
CC_MONITOR_LOG=/var/log/cc-monitor ./cc-monitor.exe
```

### 工作原理

```
CC 请求确认
    ↓
CC 调用 notify_app.py
    ↓
POST /hook/confirm → Monitor
    ↓
Monitor 状态变为"待确认"

用户选 Yes → PostToolUse → POST /hook/confirm-resolved → 状态恢复"运行中"
用户选 No  → Stop hook   → POST /hook/confirm-resolved → 状态恢复"运行中"
```

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

## 快捷键

| 操作             | 说明       |
| -------------- | -------- |
| 双击会话名称         | 编辑名称     |
| Enter / Escape | 确认/取消重命名 |

## 技术栈

- **前端**：React + TypeScript + Vite
- **后端**：Rust + Tauri v2
- **PTY**：portable-pty (Windows ConPTY)
- **监控**：sysinfo  crate
- **HTTP Server**：tiny_http

## License

MIT
