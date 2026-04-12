# CC Monitor

桌面端 Claude Code 会话监控工具（Windows）。

## 功能特性

- **会话发现**：自动扫描并监控系统中正在运行的 Claude Code 进程
- **会话启动**：通过目录启动新的 Claude Code 会话
- **实时状态**：监控 CPU、活跃时长、输出状态
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

| 操作 | 说明 |
|------|------|
| 双击会话名称 | 编辑名称 |
| Enter / Escape | 确认/取消重命名 |

## 技术栈

- **前端**：React + TypeScript + Vite
- **后端**：Rust + Tauri v2
- **PTY**：portable-pty (Windows ConPTY)
- **监控**：sysinfo  crate

## License

MIT
