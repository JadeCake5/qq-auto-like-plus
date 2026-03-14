# Story 4.3: 设置面板

Status: Done

## Story

As a 用户,
I want 通过面板修改配置,
so that 不需要手动编辑文件。

## Acceptance Criteria

1. **设置分组**：设置项分为 4 个分组卡片（圆角 14px、深色背景 `--color-bg-card`）：点赞设置、回赞设置、系统设置、运行环境设置
2. **点赞设置组**：每日名额（滑块 1-200）、每人次数（滑块 1-20）、定时时间（时:分选择器）、批次间隔（滑块 1-60 秒）
3. **回赞设置组**：回赞开关（Switch）、回赞次数（滑块 1-20）、预留名额（滑块 0-100）、回赞延迟范围（双滑块 0-60 秒）
4. **系统设置组**：开机自启开关、最小化到托盘开关
5. **运行环境设置组**：NapCat 路径（显示当前路径 + 修改按钮）、API 地址、端口配置
6. **即时保存**：所有设置修改后即时保存（调用 `invoke("update_config")`），保存成功显示 toast 通知
7. **热更新**：设置修改通过 `config:updated` 事件实时生效，无需重启（已由 TauriEventProvider 处理）
8. **恢复默认**：每个分组卡片右上角有"恢复默认"按钮
9. **表单校验**：数值范围校验、必填项校验、超出范围自动修正到边界值
10. **术语隐藏**：对用户隐藏 NapCat 术语，展示为"运行环境"
11. **无障碍**：所有表单控件有 `aria-label`，键盘可导航，focus-visible 样式

## Tasks / Subtasks

- [x] Task 1: 安装缺失的 shadcn/ui 组件 (AC: #1-#5)
  - [x] 1.1 安装 `slider` 组件：`pnpm dlx shadcn@latest add slider`
  - [x] 1.2 安装 `label` 组件：`pnpm dlx shadcn@latest add label`
  - [x] 1.3 安装 `input` 组件：`pnpm dlx shadcn@latest add input`
  - [x] 1.4 安装 `select` 组件：`pnpm dlx shadcn@latest add select`（用于时间选择器）
  - [x] 1.5 安装 `sonner`（toast 组件）：`pnpm dlx shadcn@latest add sonner`

- [x] Task 2: 扩展 AppConfig 类型和 tauri.ts 封装 (AC: #4, #5, #6)
  - [x] 2.1 在 `src/types/config.ts` 的 `AppConfig` 中添加缺失字段：`minimizeToTray: boolean`、`onebotApiUrl: string`、`webhookPort: number`
  - [x] 2.2 在 `CONFIG_DEFAULTS` 中添加新字段默认值：`minimizeToTray: true`、`onebotApiUrl: "http://127.0.0.1:3000"`、`webhookPort: 8080`
  - [x] 2.3 在 `parseConfigEntries` 中添加新字段解析
  - [x] 2.4 在 `src/lib/tauri.ts` 添加 `enableAutostart()`、`disableAutostart()`、`isAutostartEnabled()` 封装

- [x] Task 3: 创建 SettingCard 容器组件 (AC: #1, #8)
  - [x] 3.1 创建 `src/components/settings/SettingCard.tsx`
  - [x] 3.2 Props：`title: string`、`icon: ReactNode`、`onResetDefaults: () => void`、`children: ReactNode`
  - [x] 3.3 样式：`bg-bg-card` 背景、圆角 14px、内边距 20px
  - [x] 3.4 右上角渲染"恢复默认"按钮（Ghost 样式，点击调用 `onResetDefaults`）

- [x] Task 4: 创建 SliderField 复合组件 (AC: #2, #3, #9)
  - [x] 4.1 创建 `src/components/settings/SliderField.tsx`
  - [x] 4.2 Props：`label`、`value`、`min`、`max`、`step`、`unit`（可选，如"秒"/"人"）、`onChange`
  - [x] 4.3 布局：左侧标签 + 右侧当前值显示，下方 Slider
  - [x] 4.4 值变化时 clamp 到 [min, max] 范围

- [x] Task 5: 创建 TimePickerField 组件 (AC: #2)
  - [x] 5.1 创建 `src/components/settings/TimePickerField.tsx`
  - [x] 5.2 使用两个 Select（小时 0-23、分钟 0-59）组合
  - [x] 5.3 变化时同时更新 `schedule_hour` 和 `schedule_minute`

- [x] Task 6: 实现设置页面主体 (AC: #1-#11)
  - [x] 6.1 替换 `src/pages/Settings.tsx` 占位内容
  - [x] 6.2 页面挂载时调用 `useSettingsStore.fetchConfig()` 加载配置
  - [x] 6.3 渲染 4 个 SettingCard 分组，每组内包含对应表单控件
  - [x] 6.4 页面使用可滚动布局（`overflow-y: auto`）容纳全部设置项
  - [x] 6.5 开机自启开关调用 `enableAutostart()` / `disableAutostart()` 而非 `updateConfig`

- [x] Task 7: 实现即时保存与 toast 反馈 (AC: #6)
  - [x] 7.1 在 `src/App.tsx` 添加 `<Toaster />` 组件（来自 sonner）
  - [x] 7.2 每次 setting 变更调用 `useSettingsStore.updateConfig(key, value)`
  - [x] 7.3 成功后调用 `toast.success("设置已保存~")`
  - [x] 7.4 失败时调用 `toast.error("保存失败，请重试")`

- [x] Task 8: 实现恢复默认功能 (AC: #8)
  - [x] 8.1 每个 SettingCard 的"恢复默认"按钮批量调用 `updateConfig` 重置该组所有配置项
  - [x] 8.2 使用 `CONFIG_DEFAULTS` 中的默认值

- [x] Task 9: 修复 Story 4.2 遗留 QA 问题 (前置修复)
  - [x] 9.1 修复 P2-F1：`src/pages/Dashboard.tsx` 进度条 `batchProgress.total=0` 除零 — 已在 Story 4.2 中修复（`Math.max(total, 1)` 已存在）
  - [x] 9.2 修复 P3-F1：Dashboard 中直接 `invoke()` 改为使用 `src/lib/tauri.ts` 封装函数 — 已在 Story 4.2 中修复（Dashboard 已使用封装函数）

## Dev Notes

### 已有基础设施（Story 4.1 + 4.2 产出，直接复用，不要重新创建！）

**Zustand Store（已存在）：**
- `useSettingsStore` (`src/stores/useSettingsStore.ts`)
  - `config: AppConfig | null` — 当前配置
  - `fetchConfig()` — 从后端拉取全部配置
  - `updateConfig(key, value)` — 更新单个配置并自动刷新 store

**Types（已存在）：**
- `AppConfig` in `src/types/config.ts` — 当前字段：dailyLimit, timesPerFriend, scheduleHour, scheduleMinute, autoStart, replyLikeEnabled, napcatPath, qqNumber, qqNickname, reservedForReply, batchInterval, replyTimes, replyDelayMin, replyDelayMax
- `CONFIG_DEFAULTS` in `src/types/config.ts` — 所有字段默认值
- `parseConfigEntries` in `src/types/config.ts` — ConfigEntry[] → AppConfig 转换

**事件监听（已由 TauriEventProvider 全局处理）：**
- `config:updated` → 自动调用 `useSettingsStore.fetchConfig()` 重新拉取配置

**已有 UI 组件：**
- `button.tsx`、`card.tsx`、`tooltip.tsx`、`switch.tsx` — 已安装

**已有 Tauri 封装 (`src/lib/tauri.ts`)：**
- `getConfig()`、`updateConfig(key, value)` — 已有
- 缺少：`enableAutostart()`、`disableAutostart()`、`isAutostartEnabled()` → Task 2 添加

**已有 Tauri Commands（Rust 后端已实现，无需修改）：**

| Command | 说明 |
|---------|------|
| `get_config` | 读取所有 ConfigEntry |
| `update_config` | 更新单个配置，自动 emit config:updated 事件，定时配置变更时自动 reschedule |
| `enable_autostart` | 启用开机自启（调用 tauri-plugin-autostart + 写入 config） |
| `disable_autostart` | 禁用开机自启 |
| `is_autostart_enabled` | 查询自启状态 |

### 配置参数完整映射表（config 表 key → AppConfig 字段 → UI 控件）

**点赞设置组：**

| config key | AppConfig 字段 | 类型 | 默认 | 范围 | UI 控件 |
|-----------|---------------|------|------|------|---------|
| `daily_limit` | `dailyLimit` | int | 50 | 1-200 | Slider + 数值显示 |
| `times_per_friend` | `timesPerFriend` | int | 10 | 1-20 | Slider + 数值显示 |
| `schedule_hour` | `scheduleHour` | int | 0 | 0-23 | Select 下拉 |
| `schedule_minute` | `scheduleMinute` | int | 5 | 0-59 | Select 下拉 |
| `batch_interval` | `batchInterval` | int | 3 | 1-60 | Slider（单位：秒） |

**回赞设置组：**

| config key | AppConfig 字段 | 类型 | 默认 | 范围 | UI 控件 |
|-----------|---------------|------|------|------|---------|
| `reply_like_enabled` | `replyLikeEnabled` | bool | false | — | Switch |
| `reply_times` | `replyTimes` | int | 10 | 1-20 | Slider |
| `reserved_for_reply` | `reservedForReply` | int | 10 | 0-100 | Slider |
| `reply_delay_min` | `replyDelayMin` | int | 0 | 0-60 | Slider（单位：秒） |
| `reply_delay_max` | `replyDelayMax` | int | 0 | 0-60 | Slider（单位：秒） |

**系统设置组：**

| config key | AppConfig 字段 | 类型 | 默认 | 范围 | UI 控件 |
|-----------|---------------|------|------|------|---------|
| `auto_start` | `autoStart` | bool | false | — | Switch（调用 enable/disable_autostart） |
| `minimize_to_tray` | `minimizeToTray` | bool | true | — | Switch（NEW — 需添加到 AppConfig） |

**运行环境设置组：**

| config key | AppConfig 字段 | 类型 | 默认 | 范围 | UI 控件 |
|-----------|---------------|------|------|------|---------|
| `napcat_path` | `napcatPath` | string | auto | — | 只读文本 + "修改"按钮 |
| `onebot_api_url` | `onebotApiUrl` | string | `http://127.0.0.1:3000` | — | Input（NEW — 需添加到 AppConfig） |
| `webhook_port` | `webhookPort` | int | 8080 | 1024-65535 | Input（NEW — 需添加到 AppConfig） |

### 架构与编码规范

- **组件文件位置**：新增 Settings 子组件放在 `src/components/settings/` 目录下
- **页面文件**：修改现有 `src/pages/Settings.tsx`，不要创建新页面文件
- **import 别名**：使用 `@/` 路径别名
- **样式方案**：Tailwind CSS 4.x 工具类 + CSS 变量（token 在 `src/index.css` 的 `@theme` 块中）
- **组件库**：shadcn/ui（base-nova style, base-ui primitives）
- **状态管理**：Zustand v5 — 复用已有 `useSettingsStore`，不创建新 store
- **Tauri 调用**：封装在 `src/lib/tauri.ts`，页面通过 store 或 tauri.ts 调用
- **命名规范**：TypeScript camelCase，React 组件 PascalCase，config key snake_case
- **错误处理**：invoke 调用 try/catch，失败时 toast.error 通知用户

### 视觉设计要求

- **卡片背景**：`bg-bg-card`（`#231f31`），圆角 `rounded-[14px]`
- **卡片标题**：`--text-subheading` (15px)，`font-medium`，搭配 Lucide 图标
- **分组间距**：卡片间 16px gap
- **Slider 样式**：shadcn Slider 默认样式（`--color-primary` 轨道色）
- **Switch 样式**：已有 `switch.tsx`，选中时 `--color-primary`
- **Select 样式**：shadcn Select，弹出层 `bg-bg-elevated`
- **Input 样式**：shadcn Input，`bg-transparent` + border
- **Toast 样式**：使用 sonner，配置 Kawaii 暗色主题（`theme="dark"`，`richColors`）
- **恢复默认按钮**：Ghost 样式，`text-text-muted`，hover 时 `text-text-secondary`
- **页面滚动**：设置页内容超出视口时使用 `overflow-y-auto` + 自定义细滚动条
- **页面标题**：顶部 "设置" 标题使用 `--text-display` (24px)

### 关键实现细节

1. **即时保存模式**：每个控件的 onChange 直接调用 `useSettingsStore.updateConfig(key, String(value))`。`updateConfig` 内部已经：调用 `invoke("update_config")` → 后端写入 DB + emit `config:updated` → TauriEventProvider 收到事件 → 调用 `fetchConfig()` 刷新 store → UI 自动更新。所以前端只需要调 updateConfig 即可。

2. **开机自启开关特殊处理**：不走 `updateConfig`，而是调用 `invoke("enable_autostart")` / `invoke("disable_autostart")`。这两个命令内部已经同时操作注册表 + 写入 config 表。

3. **定时时间选择器**：`schedule_hour` 和 `schedule_minute` 是两个独立 config key。后端 `update_config` 在检测到这两个 key 变更时，自动调用 `scheduler.reschedule()` 热更新定时任务。

4. **回赞延迟双滑块**：`reply_delay_min` 和 `reply_delay_max` 是两个独立 Slider。校验规则：`min <= max`，如果用户拖动 min > max，自动将 max 设为 min 的值（反之亦然）。

5. **NapCat 路径修改**：只读显示当前路径。"修改"按钮暂不实现文件选择器（V1 MVP 不支持运行时切换 NapCat 路径），仅 disabled 状态 + tooltip "暂不支持修改"。

6. **Toast 集成**：sonner 的 `<Toaster />` 放在 `App.tsx` 的 `<Routes>` 同级。配置：`position="bottom-right"`, `theme="dark"`, `richColors`, `toastOptions={{ className: "!bg-bg-elevated !border-border !text-text-primary" }}`。

7. **恢复默认**：对每个分组，批量调用 updateConfig 重置该组所有配置项到 `CONFIG_DEFAULTS` 中的值。注意 autoStart 需要调用 `disableAutostart()` 而非 `updateConfig`。

8. **shadcn/ui 安装命令**：本项目使用 `base-nova` style + `base-ui` primitives（非 Radix）。安装命令统一用 `pnpm dlx shadcn@latest add <component>`。注意 Slider 在 base-ui 下的 API 可能与 Radix 版略有不同——安装后检查实际导出的 props。

### 不要做的事情

- 不要创建新的 Zustand store
- 不要修改 `TauriEventProvider.tsx`
- 不要修改 `Layout.tsx`、`SidebarNav.tsx`、`StatusBar.tsx`
- 不要修改任何 Rust 后端代码
- 不要添加额外的路由
- 不要引入新的状态管理库
- 不要修改 `src/index.css`（主题 token 已完整）
- 不要用 `console.log` 做调试
- 不要为 NapCat 路径实现文件选择对话框（V1 scope 外）

### Project Structure Notes

新增文件：
```
src/
├── components/
│   ├── settings/
│   │   ├── SettingCard.tsx       # NEW — 设置分组卡片容器
│   │   ├── SliderField.tsx       # NEW — 滑块 + 标签 + 数值显示
│   │   └── TimePickerField.tsx   # NEW — 时:分选择器
│   └── ui/
│       ├── slider.tsx            # NEW — shadcn Slider（CLI 安装）
│       ├── label.tsx             # NEW — shadcn Label（CLI 安装）
│       ├── input.tsx             # NEW — shadcn Input（CLI 安装）
│       ├── select.tsx            # NEW — shadcn Select（CLI 安装）
│       └── sonner.tsx            # NEW — shadcn Sonner/Toast（CLI 安装）
├── pages/
│   └── Settings.tsx              # MODIFY — 替换占位内容为完整设置页
└── lib/
    └── tauri.ts                  # MODIFY — 添加 autostart 封装函数
```

修改文件：
```
src/types/config.ts               # MODIFY — 添加 minimizeToTray, onebotApiUrl, webhookPort
src/App.tsx                        # MODIFY — 添加 <Toaster /> 组件
src/pages/Dashboard.tsx            # MODIFY — 修复 P2-F1 除零 bug + P3-F1 invoke 一致性
```

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 4.3: 设置面板]
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式 — Tauri IPC + Zustand Store]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — TypeScript camelCase, config key snake_case]
- [Source: .bmad-method/planning-artifacts/architecture.md#错误处理模式 — invoke try/catch + toast]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#表单模式 — 即时保存, 分组展示]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#反馈模式 — Toast 通知]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#色彩系统 — 暗色主题 token]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#无障碍策略 — WCAG AA]
- [Source: docs/prd/4-功能规格详述.md#4.1 配置参数表 — 全部配置项范围和默认值]
- [Source: src-tauri/src/commands/settings.rs — get_config, update_config, enable_autostart, disable_autostart, is_autostart_enabled]
- [Source: src/stores/useSettingsStore.ts — config, fetchConfig, updateConfig]
- [Source: src/types/config.ts — AppConfig, CONFIG_DEFAULTS, parseConfigEntries]
- [Source: src/lib/tauri.ts — 已有 Tauri command 封装]
- [Source: .bmad-method/implementation-artifacts/4-1-app-layout-and-routing.md — 基础设施清单]
- [Source: .bmad-method/implementation-artifacts/4-2-dashboard-page.md — QA findings P2-F1, P3-F1 待修复]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- Fixed sonner.tsx: Removed `next-themes` dependency (not installed, Next.js only) — hardcoded `theme="dark"`
- SliderField: Fixed `onValueChange`/`onValueCommitted` type mismatch — value is `number | readonly number[]`, added Array.isArray guard
- Settings.tsx: Removed useEffect+setState sync pattern (react-hooks/set-state-in-effect lint error) — switched to uncontrolled inputs with `key` prop for remount on config change
- Settings.tsx: Removed useRef during render pattern (react-hooks/refs lint error) — eliminated ref-based config tracking
- Task 9 (P2-F1, P3-F1): Both issues already resolved in Story 4.2 implementation, no changes needed

### Completion Notes List

- All 9 tasks completed, TypeScript and ESLint pass cleanly
- Settings page uses `updateConfig` from `@/lib/tauri.ts` directly (not store method) for proper error handling with toast feedback. Store auto-refreshes via TauriEventProvider `config:updated` event.
- Slider saves on `onValueCommitted` (mouse up) for smooth drag UX; visual feedback via local state during drag
- Text inputs (API URL, port) use uncontrolled pattern with `key`-based remount for config sync
- Port validation clamps to 1024-65535 on blur
- Delay sliders enforce min <= max constraint (auto-adjusts the other value)
- NapCat path "修改" button is disabled with tooltip as per V1 MVP scope

### File List

- `src/components/ui/slider.tsx` — NEW (shadcn CLI)
- `src/components/ui/label.tsx` — NEW (shadcn CLI)
- `src/components/ui/input.tsx` — NEW (shadcn CLI)
- `src/components/ui/select.tsx` — NEW (shadcn CLI)
- `src/components/ui/sonner.tsx` — NEW (shadcn CLI, fixed: removed next-themes)
- `src/components/settings/SettingCard.tsx` — NEW
- `src/components/settings/SliderField.tsx` — NEW
- `src/components/settings/TimePickerField.tsx` — NEW
- `src/pages/Settings.tsx` — MODIFIED (replaced placeholder with full settings page)
- `src/App.tsx` — MODIFIED (added Toaster component)
- `src/types/config.ts` — MODIFIED (added minimizeToTray, onebotApiUrl, webhookPort)
- `src/lib/tauri.ts` — MODIFIED (added enableAutostart, disableAutostart, isAutostartEnabled)
- `package.json` — MODIFIED (sonner dependency added by shadcn CLI)
- `pnpm-lock.yaml` — MODIFIED (lock file updated)

### Change Log

- 2026-03-14: Story 4.3 implementation complete — all 9 tasks done, tsc + eslint pass
- 2026-03-14: QA review complete — PASS (Quinn, Claude Opus 4.6)

## QA Results

**Gate Decision: PASS** | Reviewer: Quinn | Date: 2026-03-14
Gate File: `.bmad-method/test-artifacts/gates/4.3-settings-panel.yml`

**AC Results: 11/11 PASS**

| AC | Result |
|----|--------|
| #1 设置分组 (4 卡片, 14px 圆角) | PASS |
| #2 点赞设置组 | PASS |
| #3 回赞设置组 | PASS |
| #4 系统设置组 | PASS |
| #5 运行环境设置组 | PASS |
| #6 即时保存 + toast | PASS |
| #7 热更新 (config:updated) | PASS |
| #8 恢复默认 | PASS |
| #9 表单校验 | PASS |
| #10 术语隐藏 (NapCat→运行环境) | PASS |
| #11 无障碍 (aria-label + keyboard) | PASS |

**Findings: 0 P1, 1 P2, 2 P3, 2 P4**

- **P2-F1** (UX): 恢复默认 toast 轰炸 — 批量 saveConfig 每次独立触发 toast，单次点击产生 2-5 条相同 toast
- **P3-F1** (consistency): useSettingsStore 仍直接 invoke() 而非 tauri.ts 封装（延续 4.1 P3-F1）
- **P3-F2** (hygiene): label.tsx 含 "use client" Next.js 指令，Vite 应用无意义
- **P4-F1** (UX): 恢复默认按钮无 loading 状态
- **P4-F2** (robustness): uncontrolled Input key prop 重载边界情况（极低概率）

**Story 4.2 遗留修复验证: 3/3 ✅**
- P2-F1 进度条除零 → `Math.max(total, 1)` ✅
- P3-F1 invoke 一致性 → Dashboard 已用 tauri.ts 封装 ✅
- P3-F2 NapCat starting 状态 → 已映射为"重连中" ✅

**Risk: LOW** | 纯前端, tsc+eslint clean, 零 Rust 修改
