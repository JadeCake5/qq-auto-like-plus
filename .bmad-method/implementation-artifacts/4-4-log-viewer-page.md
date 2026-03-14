# Story 4.4: 运行日志页面

Status: Ready for Review

## Story

As a 用户,
I want 查看运行日志,
so that 出现问题时能排查原因。

## Acceptance Criteria

1. **实时日志列表**：显示实时运行日志列表，自动滚动到底部
2. **日志接收**：通过 `@tauri-apps/plugin-log` 的 `attachLogger` API 接收 Rust 后端 tracing 日志
3. **级别着色**：INFO（天空蓝 `--color-secondary` #a7c7f2）、WARN（蜜桃橙 `--color-peach` #f2cfa7）、ERROR（珊瑚红 `--color-coral` #f28b8b）
4. **工具栏**：顶部工具栏包含搜索框（关键字过滤）、级别下拉筛选（全部/INFO/WARN/ERROR）、"清空日志"按钮
5. **搜索过滤**：搜索框实时过滤当前显示的日志条目（不区分大小写）
6. **日志格式**：条目格式 `[HH:mm:ss] [LEVEL] 消息内容`，使用等宽字体 `--font-mono`
7. **日志持久化**：`tauri-plugin-log` 配置文件轮转（单文件 < 10 MB）
8. **清空行为**："清空日志"仅清空当前内存中的显示，不影响日志文件
9. **虚拟滚动**：日志条目 >500 时使用虚拟滚动优化性能（仅渲染可见区域 + 上下缓冲）
10. **滚动锁定**：用户手动上滚时暂停自动滚动，出现"回到底部"按钮；点击按钮恢复自动滚动
11. **无障碍**：日志列表 `role="log"` + `aria-live="polite"`，工具栏控件有 `aria-label`，支持键盘导航

## Tasks / Subtasks

- [x] Task 1: 配置 Rust 端 tauri-plugin-log 文件轮转 (AC: #7)
  - [x] 1.1 修改 `src-tauri/src/lib.rs` 中 `tauri_plugin_log::Builder`：添加 `.max_file_size(10_000_000)`（10 MB）和 `.rotation_strategy(RotationStrategy::KeepAll)`
  - [x] 1.2 添加 `use tauri_plugin_log::RotationStrategy;` 导入
  - [x] 1.3 设置日志级别过滤 `.level(log::LevelFilter::Info)` 过滤 trace/debug

- [x] Task 2: 增强 useLogStore (AC: #1, #4, #5, #8, #9)
  - [x] 2.1 在 `src/stores/useLogStore.ts` 添加 `searchKeyword: string` 状态和 `setSearchKeyword` setter
  - [x] 2.2 `addEntry` 中添加缓冲上限逻辑：超过 `MAX_LOG_ENTRIES`（2000）时丢弃最早的条目
  - [x] 2.3 添加 `filteredEntries` 计算 getter 函数（根据 filter + searchKeyword 过滤 entries）
  - [x] 2.4 导出 `MAX_LOG_ENTRIES` 常量

- [x] Task 3: 在 TauriEventProvider 集成 attachLogger (AC: #2)
  - [x] 3.1 在 `src/components/TauriEventProvider.tsx` 导入 `attachLogger` from `@tauri-apps/plugin-log`
  - [x] 3.2 在 `useEffect` 中调用 `attachLogger(callback)`，将 Rust 日志转换为 `LogEntry` 并调用 `useLogStore.getState().addEntry()`
  - [x] 3.3 level 映射：`1,2→忽略(trace/debug)`、`3→"info"`、`4→"warn"`、`5→"error"`
  - [x] 3.4 timestamp 取前端 `new Date().toLocaleTimeString("zh-CN", { hour12: false })`
  - [x] 3.5 id 使用自增计数器（`let logId = 0; () => String(++logId)`），避免 crypto.randomUUID 开销
  - [x] 3.6 `useEffect` cleanup 中调用 detach 函数

- [x] Task 4: 创建 LogToolbar 组件 (AC: #4, #5)
  - [x] 4.1 创建 `src/components/logs/LogToolbar.tsx`
  - [x] 4.2 搜索框：使用 shadcn Input + lucide Search 图标前缀，`placeholder="搜索日志..."`，`onChange` 调用 `useLogStore.setSearchKeyword`
  - [x] 4.3 级别筛选：使用 shadcn Select，选项：全部/INFO/WARN/ERROR，`onChange` 调用 `useLogStore.setFilter`
  - [x] 4.4 清空按钮：Ghost 样式 Button + lucide Trash2 图标，`onClick` 调用 `useLogStore.clear`
  - [x] 4.5 右侧显示日志计数 badge（当前 filtered / 总数）

- [x] Task 5: 创建 VirtualLogList 组件 (AC: #1, #3, #6, #9, #10, #11)
  - [x] 5.1 创建 `src/components/logs/VirtualLogList.tsx`
  - [x] 5.2 固定行高 `LOG_ROW_HEIGHT = 28px`，容器高度占满可用空间
  - [x] 5.3 虚拟滚动：根据 `scrollTop` 计算可见范围（startIndex, endIndex），只渲染可见条目 + 上下各 10 条缓冲
  - [x] 5.4 使用一个 `div` 撑起总高度（`totalHeight = entries.length * ROW_HEIGHT`），可见区域绝对定位或 `paddingTop` 偏移
  - [x] 5.5 每行格式：`[HH:mm:ss]` 灰色 + `[INFO/WARN/ERROR]` 级别色 + 消息内容 白色，全部 `font-mono`
  - [x] 5.6 级别着色 class 映射：`info→text-secondary`、`warn→text-peach`、`error→text-coral`
  - [x] 5.7 自动滚动：新日志到达时，如果已在底部则自动滚动到底（判断条件：`scrollTop + clientHeight >= scrollHeight - ROW_HEIGHT * 2`）
  - [x] 5.8 用户上滚时暂停自动滚动，显示浮动"↓ 回到底部"按钮（绝对定位在容器右下角）
  - [x] 5.9 容器添加 `role="log"` + `aria-live="polite"` + `aria-label="运行日志"`

- [x] Task 6: 实现 Logs 页面 (AC: #1-#11)
  - [x] 6.1 替换 `src/pages/Logs.tsx` 占位内容
  - [x] 6.2 顶部：页面标题 "运行日志"（`--text-display` 24px） + LogToolbar 同行
  - [x] 6.3 主体：VirtualLogList 占满剩余高度（`flex-1 min-h-0`）
  - [x] 6.4 容器背景 `bg-bg-card` + 圆角 `rounded-[14px]` + 内边距
  - [x] 6.5 空状态：entries 为空时显示 Mascot emoji（📋）+ "还没有日志记录~运行后会出现在这里"
  - [x] 6.6 页面使用 `page-enter` class 渐入动画

## Dev Notes

### 已有基础设施（Story 4.1/4.2/4.3 产出，直接复用！）

**Zustand Store（已存在，需增强）：**
- `useLogStore` (`src/stores/useLogStore.ts`)
  - 已有：`entries: LogEntry[]`、`filter: LogFilter`、`addEntry`、`clear`、`setFilter`
  - 已有类型：`LogEntry { id, timestamp, level, message, source? }`、`LogFilter = "all" | "info" | "warn" | "error"`
  - 缺少：`searchKeyword`、`setSearchKeyword`、缓冲上限、`getFilteredEntries()` 辅助函数

**TauriEventProvider（已存在，需添加 log 集成）：**
- 已监听：`napcat:status-changed`、`engine:status-changed`、`like:progress`、`like:batch-complete`、`config:updated`
- 缺少：`attachLogger` 日志接收 → Task 3 添加

**已安装前端依赖：**
- `@tauri-apps/plugin-log: ^2.8.0` — 已在 package.json，提供 `attachLogger` API
- Tauri capability `log:default` — 已在 `capabilities/default.json`

**已安装 Rust 依赖：**
- `tauri-plugin-log = "2"` — 已在 Cargo.toml
- `tracing = "0.1"` — 全项目 Rust 代码已使用 `tracing::info!`/`warn!`/`error!`
- 当前配置（`lib.rs:30`）：`tauri_plugin_log::Builder::new().build()` — 无轮转/限制，需增强

**已有 UI 组件：**
- `input.tsx`、`select.tsx`、`button.tsx`、`card.tsx`、`sonner.tsx` — 全部可用
- lucide-react 图标：`Search`、`Trash2`、`ArrowDown`（回到底部按钮）

**已有 Tauri 封装 (`src/lib/tauri.ts`)：**
- 本 Story 不需要新增 Tauri command wrapper — 日志通过 `attachLogger` 接收，清空仅操作前端 store

**已有 Tauri Commands（Rust 后端）：**
- 本 Story 不需要新增 Tauri command — `tauri-plugin-log` 自带日志转发机制

### `@tauri-apps/plugin-log` attachLogger API

```typescript
import { attachLogger } from "@tauri-apps/plugin-log";

// callback 参数: { level: number, message: string }
// level 映射: 1=TRACE, 2=DEBUG, 3=INFO, 4=WARN, 5=ERROR
const detach = await attachLogger(({ level, message }) => {
  // 处理日志...
}, { level: 3 }); // level 可选过滤：只接收 >= INFO 的日志

// 清理时调用 detach()
```

### 虚拟滚动实现要点

不引入外部虚拟滚动库。用 ~60 行 React 代码实现：

```
容器 (overflow-y: auto, 固定高度)
  └─ 内部 div (height = entries.length * ROW_HEIGHT)
       └─ 可见区域 (position: absolute / paddingTop 偏移)
            └─ 仅渲染 visibleStart..visibleEnd 的条目
```

关键计算：
```typescript
const ROW_HEIGHT = 28;
const BUFFER = 10;
const scrollTop = containerRef.current.scrollTop;
const visibleCount = Math.ceil(containerHeight / ROW_HEIGHT);
const startIndex = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER);
const endIndex = Math.min(entries.length, startIndex + visibleCount + BUFFER * 2);
```

### 自动滚动逻辑

```typescript
// 判断是否在底部（容差 2 行）
const isAtBottom = scrollTop + clientHeight >= scrollHeight - ROW_HEIGHT * 2;

// 新条目到达 + 在底部 → 自动滚动
// 用户手动上滚 → isAtBottom 变 false → 显示"回到底部"按钮
// 点击"回到底部" → scrollTo bottom + 恢复自动滚动
```

用 `useRef<boolean>` 跟踪 `isUserScrolledUp` 状态，避免 re-render。

### 架构与编码规范

- **组件文件位置**：新增 Logs 子组件放在 `src/components/logs/` 目录下
- **页面文件**：修改现有 `src/pages/Logs.tsx`，不要创建新页面文件
- **import 别名**：使用 `@/` 路径别名
- **样式方案**：Tailwind CSS 4.x 工具类 + CSS 变量（token 在 `src/index.css` 的 `@theme` 块中）
- **组件库**：shadcn/ui（base-nova style, base-ui primitives）
- **状态管理**：Zustand v5 — 增强已有 `useLogStore`，不创建新 store
- **日志接收**：`attachLogger` from `@tauri-apps/plugin-log`，不通过自定义 Tauri event
- **命名规范**：TypeScript camelCase，React 组件 PascalCase
- **错误处理**：`attachLogger` 调用 try/catch，失败时 console.error（不 toast — 日志系统自身的错误不应弹用户通知）

### 视觉设计要求

- **日志容器**：`bg-bg-card`（#231f31），圆角 `rounded-[14px]`
- **工具栏**：与日志容器同卡片内，顶部分隔，下方 `border-b border-border`
- **日志文字**：`font-mono`（JetBrains Mono / Cascadia Code / Consolas）
- **行高**：28px（紧凑显示更多日志）
- **时间戳**：`text-text-muted`（#6b6578）
- **级别标签宽度**：固定 6 字符宽（`[INFO ]`、`[WARN ]`、`[ERROR]`）— 使用 `inline-block` + `w-[4.5ch]` 对齐
- **级别颜色**：
  - INFO → `text-secondary`（#a7c7f2 天空蓝）
  - WARN → `text-peach`（#f2cfa7 蜜桃橙）
  - ERROR → `text-coral`（#f28b8b 珊瑚红）
- **搜索匹配高亮**：匹配的关键字用 `bg-accent/20` 半透明薰衣草紫底色标记（可选优化，非必须）
- **"回到底部"按钮**：圆形 `w-8 h-8`，`bg-bg-elevated`，`border border-border`，lucide `ArrowDown` 图标，`position: absolute` 右下角
- **空状态**：居中 emoji + 柔和文案，与 Dashboard 空状态风格一致
- **页面标题**：`--text-display` (24px)，与 Settings 页风格一致
- **清空按钮**：Ghost 样式，`text-text-muted` hover `text-coral`

### 前几个 Story 的经验教训（必须遵守！）

1. **shadcn base-ui Tooltip**：不支持 `asChild`，需要使用 `render` prop（Story 4.2 教训）
2. **Slider `onValueChange` 类型**：值可能是 `number | readonly number[]`，需 `Array.isArray` guard（Story 4.3 教训）
3. **不要在 useEffect 中 setState 然后立即用**：使用 uncontrolled 组件 + `key` prop remount 或 ref 模式（Story 4.3 教训）
4. **sonner.tsx 的 "use client" 指令**：Vite 应用无意义但无害，不要动它（Story 4.3 P3-F2）
5. **Toast 不要用于批量操作反馈**：避免 toast 轰炸（Story 4.3 P2-F1）
6. **Tauri invoke 一致性**：新增的 Tauri 调用统一放 `src/lib/tauri.ts`（Story 4.2 P3-F1），但本 Story 不需要新增 invoke wrapper

### Rust 端改动范围（最小化）

仅修改 `src-tauri/src/lib.rs` 中 1 处：

```rust
// 当前:
.plugin(tauri_plugin_log::Builder::new().build())

// 改为:
.plugin(
    tauri_plugin_log::Builder::new()
        .max_file_size(10_000_000) // 10 MB
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .build()
)
```

添加依赖 `log` crate（可能已通过 tauri-plugin-log 间接引入，如未引入需在 Cargo.toml 添加 `log = "0.4"`）— 检查后确认是否需要。如果 `RotationStrategy` 不需要额外 import，则无需 `log` crate。

### 不要做的事情

- 不要创建新的 Zustand store — 增强已有 `useLogStore`
- 不要修改 `Layout.tsx`、`SidebarNav.tsx`、`StatusBar.tsx`
- 不要修改 `App.tsx`（Toaster 已在 Story 4.3 添加）
- 不要添加额外的路由（`/logs` 路由已在 App.tsx 注册）
- 不要引入外部虚拟滚动库 — 自己实现轻量版
- 不要修改 `src/index.css`（主题 token 已完整，`--font-mono` 已定义）
- 不要用 `console.log` 做调试
- 不要为日志文件清理实现后端 command — MVP 不需要
- 不要修改已有的 TauriEventProvider 事件监听 — 只添加 attachLogger 调用
- 不要添加 `scroll-area` shadcn 组件 — 用原生 `overflow-y-auto` + 自定义细滚动条

### Project Structure Notes

新增文件：
```
src/
├── components/
│   └── logs/
│       ├── LogToolbar.tsx         # NEW — 搜索 + 级别筛选 + 清空按钮
│       └── VirtualLogList.tsx     # NEW — 虚拟滚动日志列表
└── pages/
    └── Logs.tsx                   # MODIFY — 替换占位内容为完整日志页
```

修改文件：
```
src/stores/useLogStore.ts          # MODIFY — 添加 searchKeyword、缓冲上限、filteredEntries
src/components/TauriEventProvider.tsx  # MODIFY — 添加 attachLogger 日志接收
src-tauri/src/lib.rs               # MODIFY — 配置 tauri-plugin-log 文件轮转（1 处，3 行）
```

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 4.4: 运行日志页面]
- [Source: .bmad-method/planning-artifacts/architecture.md#前端架构 — Zustand + React Router]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — TypeScript camelCase]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri 事件命名 — log:entry]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单]
- [Source: .bmad-method/planning-artifacts/architecture.md#基础设施与部署 — tracing + tauri-plugin-log]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#字体系统 — font-mono]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#色彩系统 — 级别着色]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#无障碍策略 — WCAG AA]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#状态展示模式 — 空状态]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#动画与过渡模式]
- [Source: src/stores/useLogStore.ts — 现有 LogEntry 类型和基础 store]
- [Source: src/components/TauriEventProvider.tsx — 事件监听模式参考]
- [Source: src-tauri/src/lib.rs:30 — tauri-plugin-log 当前配置]
- [Source: package.json — @tauri-apps/plugin-log ^2.8.0 已安装]
- [Source: src-tauri/capabilities/default.json — log:default 权限已配置]
- [Source: .bmad-method/implementation-artifacts/4-3-settings-panel.md — 前置 Story 经验教训]
- [Source: .bmad-method/implementation-artifacts/4-2-dashboard-page.md — 前置 Story 经验教训]

## Dev Agent Record

### Agent Model Used
Claude Opus 4.6

### Debug Log References
- attachLogger API 签名仅接受 1 个参数（callback），不支持 options 第二参数。Story Dev Notes 中示例有误，实际 `@tauri-apps/plugin-log@^2.8.0` 的 `attachLogger` 签名为 `(fn: LoggerFn) => Promise<UnlistenFn>`。已在 callback 内部过滤 level <= 2 的 trace/debug 日志。
- `log = "0.4"` 需要作为显式依赖添加到 Cargo.toml，因为 `log::LevelFilter::Info` 需要直接引用 log crate。

### Completion Notes List
- Task 1.2 中未使用独立 `use` 导入 RotationStrategy，改为内联全路径 `tauri_plugin_log::RotationStrategy::KeepAll`，减少 import 行
- useLogStore 的 `getFilteredEntries` 实现为 `get()` 方法而非 Zustand `computed`，调用方使用 `useLogStore(s => s.getFilteredEntries)()` 模式
- 虚拟滚动使用 ResizeObserver 监听容器尺寸变化，确保动态布局下正确计算可见区域
- 所有 AC 验收标准均已覆盖：实时日志、attachLogger 集成、级别着色、工具栏、搜索过滤、日志格式、文件轮转、清空行为、虚拟滚动、滚动锁定、无障碍

### File List
- `src-tauri/Cargo.toml` — MODIFIED — 添加 `log = "0.4"` 依赖
- `src-tauri/src/lib.rs` — MODIFIED — 配置 tauri-plugin-log 文件轮转 + 日志级别过滤
- `src/stores/useLogStore.ts` — MODIFIED — 添加 searchKeyword、缓冲上限、getFilteredEntries
- `src/components/TauriEventProvider.tsx` — MODIFIED — 集成 attachLogger 接收 Rust 日志
- `src/components/logs/LogToolbar.tsx` — NEW — 搜索框 + 级别筛选 + 清空按钮 + 计数
- `src/components/logs/VirtualLogList.tsx` — NEW — 虚拟滚动日志列表
- `src/pages/Logs.tsx` — MODIFIED — 完整日志页面实现

### Change Log
- 2026-03-14: Story 4.4 全部 6 个 Task 完成，TypeScript + Rust 编译通过，ESLint 零错误

## QA Results

### Gate Decision: PASS
- **Reviewer**: Quinn (QA Agent) | Claude Opus 4.6
- **Date**: 2026-03-14
- **Gate File**: `.bmad-method/test-artifacts/gates/4.4-log-viewer-page.yml`

### AC Coverage: 11/11 PASS
All acceptance criteria verified and satisfied.

### Findings Summary
| ID | Severity | Type | Description |
|---|---|---|---|
| P2-F1 | P2 | UX | 过滤结果为空时无"未找到匹配"反馈，仅显示空白区域（工具栏计数 badge 部分缓解） |
| P3-F1 | P3 | Performance | getFilteredEntries 无 memoization，每次渲染重算 O(n) |
| P3-F2 | P3 | Performance | addEntry spread 复制整个数组，高频场景产生 GC 压力 |
| P3-F3 | P3 | UX | 清空日志后 searchKeyword 未重置，新日志仍被旧关键字过滤 |
| P4-F1 | P4 | Style | scrollbar-thin CSS 类未定义，静默回退到浏览器默认滚动条 |
| P4-F2 | P4 | UX | 日志消息 truncate 无法查看完整内容（无 tooltip/展开机制） |

### Risk Assessment: LOW
- 0 P1, 1 P2 (UX only), 3 P3, 2 P4
- Rust 改动极小（3 行 plugin 配置），爆炸半径低
- 虚拟滚动实现正确，性能有保障
- 架构规范全部遵守
