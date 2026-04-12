# CC Monitor — Requirements Document

> Phase 0 产出物。确认后再进入 Phase 1。

---

## 1. 项目概述

**项目名称**: CC Monitor
**类型**: Tauri v2 桌面应用
**核心功能**: 监控多个 Claude Code 终端实例的运行状态，支持卡片视图与系统顶栏两种显示模式。
**目标用户**: 同时运行多个 Claude Code 实例的开发者

**技术栈**:
- 前端: Tauri v2 + React + TypeScript
- 后端: Rust（状态机 + PTY 监控 + 进程管理）
- 持久化: tauri-plugin-store

---

## 2. UI / UX 规范

### 2.1 双视图模式

| 视图 | 触发 | 可见内容 |
|------|------|---------|
| 卡片模式 (Card) | 默认 / 切换按钮 | 所有会话网格，含状态、进度条、终端预览、CPU/时长 |
| 顶栏模式 (Bar) | 切换按钮 | 仅 running + confirm 会话；idle/offline 折叠为徽章 |

### 2.2 视觉规范（参考 cc_monitor_dual_mode.html）

**字体**:
- 代码/终端预览: JetBrains Mono, monospace
- UI 文本: 系统默认 sans-serif

**颜色 Token (CSS Variables)**:
```css
--color-text-primary:       /* 主文字 */
--color-text-secondary:     /* 次要文字 */
--color-text-tertiary:      /* 弱化文字 */
--color-background-primary: /* 卡片/面板背景 */
--color-background-secondary: /* 顶栏/输入框背景 */
--color-border-tertiary:    /* 边框 */
--color-border-secondary:   /* hover 边框 */
```

**状态颜色**:
| 状态 | 圆点 | 进度条 | 文字 |
|------|------|--------|------|
| Running | #639922 (呼吸动画) | #639922 | #3B6D11 |
| Confirm | #EF9F27 (快速闪烁) | #EF9F27 | #854F0B |
| Idle | #378ADD (静止) | #378ADD | #185FA5 |
| Offline | #888480 (静止) | #888480 | #888480 |

**Confirm 高亮**: 最后一行 `<span class="w">` 警告色 + `<span class="p c">` 高亮闪烁游标 `█`

**深色模式**: 支持 `@media prefers-color-scheme: dark`，单独定义上述颜色 token 的 dark 版本。

**顶栏样式**:
- 背景: `var(--color-background-secondary)`
- 边框: `0.5px solid var(--color-border-tertiary)`, `border-radius: 8px`
- 透明: 否（顶栏无需透明，但需 alwaysOnTop）
- 拖拽: 使用 `[data-tauri-drag-region]` 属性

### 2.3 组件清单

| 组件 | 职责 |
|------|------|
| `StatusDot` | 状态圆点，running 呼吸 / confirm 快闪 / idle 静止蓝 / offline 灰 |
| `SilentBar` | 静默时长进度条，颜色跟随状态 |
| `TerminalPreview` | 终端最后 2 行预览，confirm 高亮最后一行 |
| `SessionCard` | 单个会话卡片，含头部+状态+进度+预览+元数据 |
| `CardGrid` | 卡片网格，「+ 添加终端」按钮触发目录选择器 |
| `TopBar` | 顶栏渲染 running/confirm，右侧折叠徽章 (idle/offline 数量) |
| `ViewToggle` | 卡片/顶栏切换按钮组 |
| `ErrorToast` | 全局错误提示 |

---

## 3. 功能需求

### 3.1 四种终端状态

| 状态 | 进程 | CPU | 静默时长 | 判断依据 |
|------|------|-----|---------|---------|
| Running | ✅ 存活 | > 5% 或有输出 | < 5s | 否 |
| Confirm | ✅ 存活 | 任意 | 任意 | 匹配确认关键词 |
| Idle | ✅ 存活 | < 2% | > 5s | 匹配空闲提示符 |
| Offline | ❌ 不存在 | — | — | — |

**优先级**: Offline > Confirm > Idle > Running（进程不存在直接判定 Offline，不再做后续判断）

**关键设计**: Confirm 状态由关键词匹配决定，**不依赖** CPU 和静默时长，避免 LLM 推理期间 Rust 进程 CPU 偏低导致的误判。

### 3.2 确认关键词（Claude Code 专用）

```
(?i)\[y/N\]        (?i)do you want to
(?i)\[Y/n\]        (?i)are you sure
(?i)continue\?     (?i)proceed\?
(?i)overwrite\?   (?i)press enter to
```

### 3.3 空闲提示符

```
(?i)what would you like
(?i)how can i help
^\s*>\s*$          （Unix shell 提示符）
```

> ⚠️ **Windows 补充**: 还需匹配 Windows cmd (`>`) 和 PowerShell (`PS ...>`) 提示符。
> 当前 pattern 仅支持 Unix，**需要扩展**。

### 3.4 核心功能

- [ ] 通过工作目录路径启动 CC 实例（Rust spawn + PTY 接管，不允许手动输入 PID）
- [ ] 双视图模式（卡片网格 / 透明顶栏），视图模式持久化（重启恢复）
- [ ] 待确认状态触发系统原生通知
- [ ] 状态变化通过 `app.emit` 推送，前端订阅无需轮询
- [ ] 进程异常退出时推送 `session_error` 事件

### 3.5 双窗口架构

| 窗口 | 标签 | 尺寸 | decorations | alwaysOnTop | transparent |
|------|------|------|------------|------------|-------------|
| main | main | 900×600 | true | false | false |
| bar | bar | 600×36 | false | true | true |

切换视图时 `hide/show`，**不销毁重建**。

---

## 4. 技术约束

### 4.1 Rust 依赖

| Crate | 版本 | 用途 |
|-------|------|------|
| portable-pty | 0.8 | PTY 接管，监控终端输出流 |
| sysinfo | 0.30 | CPU 占用率、进程存活检测 |
| serde / serde_json | 1 | 数据序列化 |
| tokio | 1 | async runtime（仅 rt-multi-thread, io-util, sync, macros） |
| regex | 1 | pattern 匹配 |
| strip-ansi-escapes | 0.2 | 剥离 PTY ANSI 转义 |
| tauri-plugin-store | 2 | 偏好持久化 |
| tauri-plugin-notification | 2 | 系统通知 |

### 4.2 前端依赖

```
@tauri-apps/api
@tauri-apps/plugin-store
@tauri-apps/plugin-notification
```

### 4.3 Tauri Commands

```rust
launch_session(work_dir: String) -> Result<String, String>   // 启动新会话
stop_session(id: String) -> Result<(), String>               // 停止会话
get_sessions() -> Result<Vec<SessionInfo>, String>           // 获取所有会话
set_view_mode(mode: String, app: AppHandle) -> Result<(), String>
get_view_mode(app: AppHandle) -> Result<String, String>
```

### 4.4 事件推送

| 事件 | 触发条件 |
|------|---------|
| `session_update` | 任意状态变化 |
| `session_error` | 进程异常退出 (exit_code != 0) |
| 系统通知 | 状态变为 Confirm（包含会话名称） |

---

## 5. 前端类型定义

```typescript
export type SessionState = 'running' | 'confirm' | 'idle' | 'offline'

export interface Session {
  id: string
  name: string
  workDir: string          // 工作目录路径，作为会话标识
  pid: number | null      // null 表示离线
  state: SessionState
  silentSecs: number
  cpuPercent: number
  lastLine: string         // ANSI 已剥离
  startedAt: number       // Unix timestamp (ms)
}

export type CcResult<T> = { ok: true; data: T } | { ok: false; error: string }
```

---

## 6. Windows 兼容性已知风险

| 风险项 | 说明 | 处理方式 |
|--------|------|---------|
| ConPTY 稳定性 | `portable-pty` 在 Windows ConPTY 下对某些程序支持不稳定 | 需测试，备选方案降级为 stdout 管道 |
| 路径格式 | Git Bash `/d/project` vs Windows `D:\project` | 启动时标准化路径 |
| 空闲提示符 | `IDLE_PATTERNS` 中 `^\s*>\s*$` 不匹配 Windows cmd/PowerShell | 补充 Windows 提示符 pattern |
| 透明窗口 | Windows DWM 对透明无边框窗口有特殊要求 | 顶栏 `transparent: false` 更稳定 |
| 通知权限 | `tauri-plugin-notification` 需 Windows 通知中心授权 | 首次触发时引导用户开启 |

---

## 7. 待确认事项

- [ ] 是否需要支持 macOS？（影响 path 处理和 PTY 实现）
- [ ] 顶栏模式透明窗口是否改为不透明？（Windows 稳定性优先）
- [ ] 是否需要支持 Git Bash / MSYS2 下启动 CC？（影响路径标准化逻辑）
- [ ] Confirm 状态是否需要用户手动点击卡片内的按钮触发 "y"，还是仅通知？
- [ ] 离线会话保留多久后自动清理？

---

*Phase 0 · 待确认后进入 Phase 1*
