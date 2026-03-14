# Story 4.1: 应用布局框架与路由

Status: Done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 用户,
I want 面板有清晰的导航和一致的布局,
so that 我能快速找到需要的功能。

## Acceptance Criteria

1. **Given** 用户打开管理面板 **When** 面板窗口显示 **Then** 左侧显示 56px 宽的图标导航栏（深色背景 `#16121f`）
2. **Given** 导航栏已渲染 **When** 用户查看导航 **Then** 导航栏包含 5 个图标按钮：仪表盘、好友管理、数据统计、运行日志、设置
3. **Given** 用户点击导航项 **When** 路由切换 **Then** 当前选中的导航项有高亮样式（渐变背景 + 发光效果）
4. **Given** 用户鼠标悬停图标 **When** 停留 500ms **Then** 显示 tooltip 文字（中文页面名称）
5. **Given** 路由匹配 **When** 导航切换 **Then** 右侧主内容区根据路由渲染对应页面
6. **Given** 路由系统 **When** 配置完成 **Then** 使用 React Router v7 管理路由（`/dashboard`、`/friends`、`/statistics`、`/logs`、`/settings`）
7. **Given** 用户首次打开面板 **When** 无特定路由 **Then** 默认路由为 `/dashboard`
8. **Given** 面板已渲染 **When** 用户查看顶部 **Then** 顶部 24px 状态条显示 NapCat 连接状态指示器 + 登录 QQ 信息
9. **Given** 全局样式 **When** 面板显示 **Then** 使用 Kawaii 暗色主题（深紫底色 `#1a1625`、马卡龙色点缀）
10. **Given** 全局样式 **When** 文字渲染 **Then** 所有文字使用 HarmonyOS Sans SC 字体栈
11. **Given** 无障碍 **When** 用户使用键盘 **Then** 支持键盘导航（Tab 切换导航项，Enter 选中）
12. **Given** 状态管理 **When** 应用初始化 **Then** 创建 Zustand stores：`useLikeStore`、`useNapCatStore`、`useSettingsStore`、`useLogStore`
13. **Given** 事件系统 **When** 应用初始化 **Then** 创建 `useTauriEvent` hook 封装 Tauri 事件监听

## Tasks / Subtasks

- [x] Task 1: 安装 React Router 并配置路由系统 (AC: #5, #6, #7)
  - [x] 1.1 修改 `src/main.tsx`，用 `BrowserRouter` 包裹 `<App />`
  - [x] 1.2 修改 `src/App.tsx`，引入 `Routes`/`Route`，设置 5 条路由 + 默认重定向到 `/dashboard`
  - [x] 1.3 创建 5 个页面占位组件：`src/pages/Dashboard.tsx`、`Friends.tsx`、`Statistics.tsx`、`Logs.tsx`、`Settings.tsx`（每个页面仅显示标题占位文字）

- [x] Task 2: 创建应用布局组件 Layout (AC: #1, #9)
  - [x] 2.1 创建 `src/components/layout/Layout.tsx`，实现 56px 侧边栏 + 主内容区 Flex 布局
  - [x] 2.2 布局使用 `bg-[#1a1625]` 全局底色，侧边栏使用 `bg-[#16121f]`
  - [x] 2.3 主内容区使用 `<Outlet />` 渲染子路由页面
  - [x] 2.4 Layout 嵌套在 `App.tsx` 的 `Route` 结构中作为 layout route

- [x] Task 3: 创建侧边栏导航组件 SidebarNav (AC: #1, #2, #3, #4, #11)
  - [x] 3.1 创建 `src/components/layout/SidebarNav.tsx`
  - [x] 3.2 使用 Lucide 图标：`LayoutDashboard`（仪表盘）、`Users`（好友管理）、`BarChart3`（数据统计）、`FileText`（运行日志）、`Settings`（设置）
  - [x] 3.3 导航项使用 `NavLink` (react-router-dom) 实现路由联动，通过 `isActive` 判断选中态
  - [x] 3.4 选中态样式：圆角背景 `bg-gradient-to-br from-primary/20 to-accent/20` + `shadow-[0_0_12px_rgba(242,167,195,0.3)]` 发光效果
  - [x] 3.5 未选中态：图标 `text-text-muted`，hover 时 `text-text-secondary` + 轻微背景
  - [x] 3.6 使用 shadcn/ui 的 `Tooltip` 组件，hover 延迟 500ms 显示中文页面名
  - [x] 3.7 设置图标放在底部（用 `mt-auto` 分隔），其余 4 个图标在顶部
  - [x] 3.8 键盘无障碍：`<nav>` 语义标签，每个图标 `aria-label`，选中项 `aria-current="page"`（NavLink 自动设置），Tab 可聚焦，Enter 激活

- [x] Task 4: 创建顶部状态条组件 StatusBar (AC: #8)
  - [x] 4.1 创建 `src/components/layout/StatusBar.tsx`
  - [x] 4.2 高度 36px（UX spec 修正），全宽，`bg-bg-card` 背景 + 底部 1px 分隔线
  - [x] 4.3 左侧：NapCat 连接状态指示灯（8px 圆形，绿/黄/红）+ 状态文字（"已连接"/"断开"/"重连中"）
  - [x] 4.4 右侧：QQ 昵称 + QQ 号显示（从 `useNapCatStore` 读取）
  - [x] 4.5 状态数据通过 `useNapCatStore` 获取，初始显示 "未连接"

- [x] Task 5: 创建 `useTauriEvent` hook (AC: #13)
  - [x] 5.1 创建 `src/hooks/useTauriEvent.ts`
  - [x] 5.2 封装 `@tauri-apps/api/event` 的 `listen` 函数
  - [x] 5.3 在 `useEffect` 中注册监听，返回 `unlisten` 清理函数
  - [x] 5.4 泛型支持：`useTauriEvent<T>(eventName: string, handler: (payload: T) => void)`

- [x] Task 6: 创建 Zustand stores (AC: #12)
  - [x] 6.1 创建 `src/stores/useNapCatStore.ts` — 管理 NapCat 状态（status、loginInfo、qrCode）
  - [x] 6.2 创建 `src/stores/useLikeStore.ts` — 管理点赞引擎状态（dailyStats、isRunning、batchProgress）
  - [x] 6.3 创建 `src/stores/useSettingsStore.ts` — 管理配置（config、fetchConfig、updateConfig）
  - [x] 6.4 创建 `src/stores/useLogStore.ts` — 管理日志（entries、addEntry、clear、filter）
  - [x] 6.5 每个 store 使用 `invoke` 调用对应的 Tauri commands 获取初始数据

- [x] Task 7: 创建全局事件初始化组件 (AC: #12, #13)
  - [x] 7.1 创建 `src/components/TauriEventProvider.tsx`
  - [x] 7.2 在组件挂载时使用 `useTauriEvent` 监听所有核心事件并写入对应 store：
    - `napcat:status-changed` → `useNapCatStore.setStatus`
    - `engine:status-changed` → `useLikeStore.setEngineStatus`
    - `like:progress` → `useLikeStore.setBatchProgress`
    - `like:batch-complete` → `useLikeStore.onBatchComplete`
    - `config:updated` → `useSettingsStore.fetchConfig`
  - [x] 7.3 在 `App.tsx` 中渲染 `<TauriEventProvider />` 确保全局事件监听

- [x] Task 8: 页面切换动画与样式收尾 (AC: #9, #10)
  - [x] 8.1 页面切换添加 CSS 淡入过渡（`opacity 0→1, 150ms ease-out`）
  - [x] 8.2 确认 `index.css` 中 `--font-primary` 字体栈全局生效
  - [x] 8.3 给 `body` / `#root` 添加 `overflow: hidden` 防止滚动条（固定窗口）

- [x] Task 9: 构建验证 (AC: #1-#13)
  - [x] 9.1 `pnpm build` 编译无 TypeScript 错误
  - [x] 9.2 `pnpm lint` 无 ESLint 错误
  - [x] 9.3 手动验证：侧边栏 5 个图标可点击切换页面
  - [x] 9.4 手动验证：选中态高亮 + tooltip 显示
  - [x] 9.5 手动验证：键盘 Tab 导航 + Enter 激活
  - [x] 9.6 手动验证：顶部状态条显示（即使数据为空也有占位）

## Dev Notes

### 核心挑战：搭建前端骨架，为后续 4 个前端 Story 建立基础

本 Story 是 Epic 4（管理面板）的第一个 Story，目标是建立**完整的前端应用骨架**。包括路由系统、布局组件、状态管理、事件系统。后续 Story 4.2（仪表盘）、4.3（设置）、4.4（日志）将在这个骨架上填充各页面的具体内容。

**本 Story 仅创建前端代码，不修改任何 Rust 代码。**

### 已存在的基础设施（不要重复！）

**前端已有文件：**
- `src/App.tsx` — 当前是简单占位卡片，**需要完全重写**为路由布局
- `src/main.tsx` — React 入口，需要包裹 `BrowserRouter`
- `src/index.css` — **完整的 Kawaii 暗色主题已配置**（所有颜色 token、圆角、字体栈），不要修改
- `src/lib/utils.ts` — `cn()` 工具函数已存在
- `src/lib/tauri.ts` — Tauri command 调用封装已存在（getConfig、updateConfig、getNapCatStatus 等）
- `src/components/ui/button.tsx` — shadcn/ui Button 组件已安装
- `src/components/ui/card.tsx` — shadcn/ui Card 组件已安装
- `src/types/config.ts` — ConfigEntry、AppConfig、parseConfigEntries 已定义
- `src/types/engine.ts` — EngineStatus 接口已定义
- `src/types/napcat.ts` — NapCatStatus、DownloadProgress、ExtractProgress、LoginInfo 已定义
- `src/types/like.ts` — BatchLikeProgress、BatchLikeResult 已定义
- `src/types/stats.ts` — QuotaStatus 已定义
- `src/types/onebot.ts` — FriendInfo、OneBotLoginInfo 已定义

**package.json 已安装的依赖（直接 import 使用，不要 `pnpm add`）：**
- `react-router-dom@7.13.1` ✅
- `zustand@5.0.11` ✅
- `lucide-react@0.577.0` ✅
- `recharts@3.8.0` ✅
- `@tauri-apps/api@2` ✅（包含 `@tauri-apps/api/event` 和 `@tauri-apps/api/core`）

**Tailwind CSS 4.x + shadcn/ui 已配置：**
- `components.json` 配置完成（style: base-nova, aliases 映射 `@/`）
- `@` 路径别名已在 `vite.config.ts` 和 `tsconfig.json` 中配置

**Rust 后端已提供的 Tauri commands（无需修改，直接 invoke 调用）：**

| Command | 返回类型 | 用途 |
|---------|---------|------|
| `get_config` | `ConfigEntry[]` | 读取所有配置 |
| `update_config` | `void` | 更新单个配置 |
| `get_napcat_status` | `NapCatStatus` | 获取 NapCat 状态 |
| `get_login_info_cmd` | `LoginInfo` | 获取登录 QQ 信息 |
| `start_napcat` | `void` | 启动 NapCat |
| `stop_napcat` | `void` | 停止 NapCat |
| `get_daily_stats` | `QuotaStatus` | 获取今日名额状态 |
| `start_batch_like` | `void` | 手动触发批量点赞 |
| `pause_engine` | `void` | 暂停引擎 |
| `resume_engine` | `void` | 恢复引擎 |
| `get_engine_status` | `EngineStatus` | 获取引擎状态 |
| `get_next_run_time` | `string \| null` | 获取下次执行时间 |
| `enable_autostart` | `void` | 启用开机自启 |
| `disable_autostart` | `void` | 禁用开机自启 |
| `is_autostart_enabled` | `bool` | 查询自启状态 |

**Rust 后端已提供的 Tauri 事件（前端通过 `listen` 接收）：**

| 事件名 | Payload 类型 | 说明 |
|--------|-------------|------|
| `napcat:status-changed` | `NapCatStatus` | NapCat 状态变化 |
| `napcat:download-progress` | `DownloadProgress` | 下载进度 |
| `napcat:extract-progress` | `ExtractProgress` | 解压进度 |
| `napcat:qr-code` | `string` (base64) | 二维码数据 |
| `napcat:login-required` | — | 需要重新扫码 |
| `engine:status-changed` | `EngineStatus` | 引擎状态变化 |
| `like:progress` | `BatchLikeProgress` | 批量点赞进度 |
| `like:batch-complete` | `BatchLikeResult` | 批量点赞完成 |
| `config:updated` | — | 配置已更新 |

### React Router v7 用法（react-router-dom@7.13.1）

```tsx
// src/main.tsx
import { BrowserRouter } from "react-router-dom";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>,
);

// src/App.tsx
import { Routes, Route, Navigate } from "react-router-dom";
import { Layout } from "@/components/layout/Layout";

function App() {
  return (
    <>
      <TauriEventProvider />
      <Routes>
        <Route element={<Layout />}>
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/friends" element={<Friends />} />
          <Route path="/statistics" element={<Statistics />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Route>
      </Routes>
    </>
  );
}
```

### Zustand v5 Store 模式（zustand@5.0.11）

```typescript
// Zustand v5 用法 — 与 v4 基本一致
import { create } from "zustand";

interface NapCatStore {
  status: NapCatStatus;
  loginInfo: LoginInfo | null;
  setStatus: (status: NapCatStatus) => void;
  setLoginInfo: (info: LoginInfo) => void;
  fetchLoginInfo: () => Promise<void>;
}

export const useNapCatStore = create<NapCatStore>((set) => ({
  status: "notInstalled" as NapCatStatus,
  loginInfo: null,
  setStatus: (status) => set({ status }),
  setLoginInfo: (info) => set({ loginInfo: info }),
  fetchLoginInfo: async () => {
    try {
      const info = await invoke<LoginInfo>("get_login_info_cmd");
      set({ loginInfo: info });
    } catch {
      // 未登录时静默忽略
    }
  },
}));
```

### useTauriEvent Hook 模式

```typescript
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void,
) {
  useEffect(() => {
    const unlisten = listen<T>(eventName, (event) => {
      handler(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [eventName, handler]);
}
```

**注意**：`handler` 应使用 `useCallback` 包裹或直接传 store 的 setter（Zustand setter 引用稳定），避免 useEffect 无限重新注册。

### 侧边栏布局精确规格

```
┌──────────────────────────────────────────────┐
│ 56px 侧边栏  │     主内容区 (844px)           │
│  bg-bg-nav   │                               │
│              │  StatusBar (36px)               │
│  [仪表盘]    │ ┌───────────────────────────┐  │
│  [好友]      │ │                           │  │
│  [统计]      │ │   页面内容                 │  │
│  [日志]      │ │   (padding: 20px)          │  │
│              │ │                           │  │
│  ─────       │ │                           │  │
│  [设置]      │ └───────────────────────────┘  │
└──────────────────────────────────────────────┘
```

- 总窗口：900×600px（tauri.conf.json 已配置，不可调整大小）
- 侧边栏：56px × 600px，`bg-[var(--color-bg-nav)]`（`#16121f`）
- 导航图标：每个 40×40px 可点击区域，图标 20px，水平居中
- 图标间距：垂直 8px gap
- 前 4 个图标顶部区域 + 设置图标底部区域（用 `flex-col` + `mt-auto`）
- 主内容区：844×600px，`bg-[var(--color-bg-base)]`（`#1a1625`）

### shadcn/ui Tooltip 安装

当前项目只有 `button.tsx` 和 `card.tsx`。**需要安装 `tooltip` 组件**：

```bash
pnpm dlx shadcn@latest add tooltip
```

这会在 `src/components/ui/` 下生成 `tooltip.tsx`。使用方式：

```tsx
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

<TooltipProvider delayDuration={500}>
  <Tooltip>
    <TooltipTrigger asChild>
      <button>...</button>
    </TooltipTrigger>
    <TooltipContent side="right">
      <p>仪表盘</p>
    </TooltipContent>
  </Tooltip>
</TooltipProvider>
```

### 导航项选中态样式参考

```tsx
// 选中态（active）
className={cn(
  "relative flex items-center justify-center w-10 h-10 rounded-xl transition-all duration-200",
  isActive
    ? "bg-gradient-to-br from-primary/20 to-accent/20 text-primary shadow-[0_0_12px_rgba(242,167,195,0.3)]"
    : "text-text-muted hover:text-text-secondary hover:bg-white/5"
)}
```

### 页面切换动画

使用 CSS 类实现简单淡入：

```css
/* 在 index.css @layer base 中添加 */
.page-enter {
  animation: fadeIn 150ms ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
```

每个页面组件的根 div 添加 `className="page-enter"`。

### NapCat 连接状态指示灯颜色

| 状态 | 颜色 | CSS |
|------|------|-----|
| running | 薄荷绿 | `bg-mint` |
| ready | 薄荷绿 | `bg-mint` |
| starting / waitingForLogin / downloading / extracting | 蜜桃橙 | `bg-peach` |
| notInstalled / error | 珊瑚红 | `bg-coral` |

指示灯 8px 圆形 + 呼吸动画（`animate-pulse`）当 starting/downloading 等进行中状态。

### 不要做的事

- **不要** 修改任何 Rust 代码（`src-tauri/` 下的所有文件）
- **不要** 修改 `src/index.css` — Kawaii 主题已完整配置
- **不要** 修改 `package.json` — 所有依赖已安装
- **不要** 修改 `vite.config.ts`、`tsconfig.json`、`components.json`
- **不要** 在 store 中使用 `persist` 中间件 — 所有持久化由 Rust 后端 SQLite 负责
- **不要** 在页面组件中放任何实际内容 — 页面内容是 Story 4.2/4.3/4.4 的范围
- **不要** 创建 `useFriendsStore` 或 `useStatsStore` — 这些属于 Epic 6/7
- **不要** 使用 `println` 或 `console.log` 做调试 — 生产代码中不保留
- **不要** 删除 `src/assets/react.svg` — 不影响构建，留着不碍事
- **不要** 安装 `@radix-ui/react-tooltip` — shadcn/ui tooltip 会自动处理依赖

### 需要新安装的 shadcn/ui 组件

```bash
pnpm dlx shadcn@latest add tooltip
```

仅此一个。Button 和 Card 已存在。

### 文件创建/修改清单

**修改文件（2 个）：**
```
src/main.tsx          ← 添加 BrowserRouter 包裹
src/App.tsx           ← 完全重写为路由布局
```

**新建文件（13 个）：**
```
src/components/layout/
├── Layout.tsx        ← 应用主布局（侧边栏 + StatusBar + Outlet）
├── SidebarNav.tsx    ← 左侧图标导航栏
└── StatusBar.tsx     ← 顶部状态条

src/pages/
├── Dashboard.tsx     ← 占位页面
├── Friends.tsx       ← 占位页面
├── Statistics.tsx    ← 占位页面
├── Logs.tsx          ← 占位页面
└── Settings.tsx      ← 占位页面

src/hooks/
└── useTauriEvent.ts  ← Tauri 事件监听 hook

src/stores/
├── useNapCatStore.ts ← NapCat 状态
├── useLikeStore.ts   ← 点赞引擎状态
├── useSettingsStore.ts ← 配置状态
└── useLogStore.ts    ← 日志状态

src/components/
└── TauriEventProvider.tsx ← 全局事件监听初始化
```

**自动生成（1 个）：**
```
src/components/ui/tooltip.tsx ← shadcn add tooltip 自动生成
```

### 与其他 Story 的边界

- **Story 4.2**（仪表盘页面）：将填充 `pages/Dashboard.tsx` 的实际内容（Hero Banner、StatCard、趋势图等），使用本 Story 创建的 `useLikeStore` 和 `useNapCatStore`
- **Story 4.3**（设置面板）：将填充 `pages/Settings.tsx`，使用本 Story 创建的 `useSettingsStore`
- **Story 4.4**（日志页面）：将填充 `pages/Logs.tsx`，使用本 Story 创建的 `useLogStore`
- **Story 3.4**（已完成）：提供了 autostart commands，设置页面（4.3）将调用 `enable_autostart`/`disable_autostart`
- **Story 3.4 Dev Notes 提到**：开机自启后 NapCat 启动由前端触发——本 Story 的 `TauriEventProvider` 或 `useNapCatStore` 应在初始化时检查 NapCat 状态并按需触发 `start_napcat`（但实际的 NapCat 启动流程由 Story 4.2 Dashboard 负责，本 Story 仅提供 store 基础设施）

### Project Structure Notes

- 组件目录结构遵循 UX spec 的 `src/components/layout/` 分类
- 页面组件直接放在 `src/pages/` 下，PascalCase 命名
- Zustand stores 放在 `src/stores/` 下，`use` 前缀 camelCase 命名
- Hooks 放在 `src/hooks/` 下
- 所有路径使用 `@/` 别名导入

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story4.1] — AC 定义：应用布局框架与路由
- [Source: .bmad-method/planning-artifacts/architecture.md#前端架构] — shadcn/ui + Tailwind CSS 4.x + Zustand + React Router v7 + Recharts + Lucide
- [Source: .bmad-method/planning-artifacts/architecture.md#结构模式] — React 组件组织：pages/{PageName}.tsx, components/{ComponentName}.tsx, components/ui/
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式] — Zustand Store 模式 + Tauri events emit/listen
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范] — TypeScript/React 命名规范
- [Source: .bmad-method/planning-artifacts/architecture.md#完整项目目录结构] — src/pages/, src/components/, src/stores/, src/hooks/
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#窗口布局] — 56px 侧边导航栏 + 主内容区布局
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#色彩系统] — 全部颜色 token 定义
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#字体系统] — HarmonyOS Sans SC 字体栈
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#SidebarNav组件] — 导航栏组件规格
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#StatusBar组件] — 状态条组件规格
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#无障碍策略] — WCAG 2.1 AA, 键盘导航, ARIA 标注
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#导航模式] — 侧边栏选中态样式规格
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#动画与过渡模式] — 页面切换淡入 150ms
- [Source: src/index.css] — Kawaii 暗色主题完整 token（不修改）
- [Source: src/lib/tauri.ts] — 已有 Tauri command 封装函数
- [Source: src/types/] — 已有全部 TypeScript 类型定义
- [Source: src-tauri/src/lib.rs:199-218] — 全部已注册 Tauri commands
- [Source: src-tauri/src/lib.rs:95-179] — 已有 Tauri 事件监听（napcat:status-changed, engine:status-changed）
- [Source: .bmad-method/implementation-artifacts/3-4-autostart-and-single-instance.md#NapCat自动启动边界说明] — NapCat 启动由前端触发
- [Source: package.json] — 全部依赖已安装，无需 pnpm add

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- 无调试问题

### Completion Notes List

- shadcn/ui tooltip 使用 base-ui（非 Radix），API 为 `delay` 而非 `delayDuration`
- TooltipTrigger 使用 `render` prop 渲染 NavLink 以兼容 base-ui 的 trigger API
- NavLink 自动设置 `aria-current="page"`，无需手动传递
- useLogStore 定义了 LogEntry 接口（id、timestamp、level、message、source），类型不在 src/types/ 中因为日志是纯前端状态
- 所有 Zustand store setter 引用稳定，可直接传给 useTauriEvent 无需 useCallback 包裹
- index.css 仅追加了 page-enter 动画和 overflow:hidden，未修改主题 token

### Change Log

- 修改 `src/main.tsx` — 添加 BrowserRouter 包裹
- 重写 `src/App.tsx` — 路由系统 + Layout + TauriEventProvider + TooltipProvider
- 追加 `src/index.css` — page-enter 动画 + overflow:hidden（仅新增规则）

### File List

**修改文件：**
- `src/main.tsx` — 添加 BrowserRouter
- `src/App.tsx` — 完全重写为路由布局
- `src/index.css` — 追加 page-enter 动画 + overflow:hidden

**新建文件：**
- `src/components/layout/Layout.tsx` — 应用主布局
- `src/components/layout/SidebarNav.tsx` — 左侧图标导航栏
- `src/components/layout/StatusBar.tsx` — 顶部状态条
- `src/components/TauriEventProvider.tsx` — 全局事件监听初始化
- `src/pages/Dashboard.tsx` — 仪表盘占位页
- `src/pages/Friends.tsx` — 好友管理占位页
- `src/pages/Statistics.tsx` — 数据统计占位页
- `src/pages/Logs.tsx` — 运行日志占位页
- `src/pages/Settings.tsx` — 设置占位页
- `src/hooks/useTauriEvent.ts` — Tauri 事件监听 hook
- `src/stores/useNapCatStore.ts` — NapCat 状态 store
- `src/stores/useLikeStore.ts` — 点赞引擎状态 store
- `src/stores/useSettingsStore.ts` — 配置状态 store
- `src/stores/useLogStore.ts` — 日志状态 store

**自动生成：**
- `src/components/ui/tooltip.tsx` — shadcn add tooltip 生成

## QA Results

**Gate Decision: PASS** | Reviewer: Quinn | Date: 2026-03-14 | Model: Claude Opus 4.6

**AC 验证: 13/13 PASS** — 全部验收标准通过

**构建验证:**
- `tsc --noEmit` 零 TypeScript 错误

**Findings: 0 P1, 0 P2, 1 P3, 2 P4**

- **P3-F1 (consistency)**: Stores 直接调用 `invoke()` 而非复用 `src/lib/tauri.ts` 封装函数（如 `getNapCatStatus()`、`getConfig()`）。违反 DRY，建议后续 Story 实现时一并重构。
- **P4-F1 (scalability)**: `useLogStore.addEntry` 无上限追加，长时间运行可能积累大量条目。建议 Story 4.4 实现时加 maxEntries 限制。
- **P4-F2 (ux)**: StatusBar 错误状态仅显示"错误"，未展示 `{ error: string }` 的具体信息。建议后续展示错误详情。

**风险评估: LOW** — 纯前端改动，零 Rust 修改，模式清晰，基础设施到位。

**Gate 文件:** `.bmad-method/test-artifacts/gates/4.1-app-layout-and-routing.yml`
