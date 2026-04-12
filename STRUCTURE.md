# CC Monitor — 项目结构

```
cc-monitor/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
├── .gitignore
├── public/
│   └── vite.svg
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── App.css
│   ├── types.ts
│   ├── vite-env.d.ts
│   ├── styles/
│   │   └── tokens.css
│   ├── components/
│   │   ├── StatusDot.tsx + StatusDot.css
│   │   ├── SilentBar.tsx + SilentBar.css
│   │   ├── TerminalPreview.tsx + TerminalPreview.css
│   │   ├── SessionCard.tsx + SessionCard.css
│   │   ├── CardGrid.tsx + CardGrid.css
│   │   ├── TopBar.tsx + TopBar.css
│   │   ├── ViewToggle.tsx + ViewToggle.css
│   │   └── ErrorToast.tsx + ErrorToast.css
│   └── hooks/
│       ├── useSessions.ts   (Phase 4 stub)
│       └── useLaunchSession.ts  (Phase 4 stub)
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── commands.rs    (Phase 3 implementation)
        ├── session.rs     (Phase 3 implementation)
        ├── state_machine.rs  (Phase 3 implementation)
        └── pty_watcher.rs    (Phase 3 implementation)
```

## 前端组件说明

| 组件 | 职责 | 状态 |
|------|------|------|
| StatusDot | 状态圆点 + 动画 | ✅ 完成 |
| SilentBar | 静默时长进度条 | ✅ 完成 |
| TerminalPreview | 终端最后一行预览 + 游标 | ✅ 完成 |
| SessionCard | 完整会话卡片 | ✅ 完成 |
| CardGrid | 卡片网格 + 添加按钮 | ✅ 完成 |
| TopBar | 顶栏 running/confirm + 折叠徽章 | ✅ 完成 |
| ViewToggle | 卡片/顶栏切换按钮组 | ✅ 完成 |
| ErrorToast | 全局错误提示 | ✅ 完成 |
| useSessions | 会话数据 hook | 🔒 Phase 4 |
| useLaunchSession | 启动会话 hook | 🔒 Phase 4 |

## 后端模块说明

| 模块 | 职责 | 状态 |
|------|------|------|
| main.rs | 入口点 | ✅ 完成 |
| lib.rs | Tauri builder + 插件初始化 | ✅ 完成 |
| commands.rs | Tauri commands (5个命令) | 🔒 Phase 3 |
| session.rs | 会话数据结构 | 🔒 Phase 3 |
| state_machine.rs | 状态判断逻辑 | 🔒 Phase 3 |
| pty_watcher.rs | PTY 监控线程 | 🔒 Phase 3 |

## 待安装依赖

前端:
```bash
npm install
```

后端（Tauri CLI 自动处理）:
```bash
cargo build
```
