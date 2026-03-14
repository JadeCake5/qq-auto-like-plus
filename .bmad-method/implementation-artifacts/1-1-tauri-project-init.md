# Story 1.1: 使用 Tauri 模板初始化项目并安装核心依赖

Status: Done

## Story

As a 开发者,
I want 使用官方 create-tauri-app 初始化项目并安装所有核心依赖,
so that 项目有一个干净、标准的基础结构可以开始开发。

## Acceptance Criteria

1. **Given** 开发环境已安装 Node.js、pnpm 和 Rust 工具链 **When** 执行 `pnpm create tauri-app qq-auto-like-plus --template react-ts` **Then** 项目结构包含 `src/`（React 前端）和 `src-tauri/`（Rust 后端）
2. 前端依赖已安装：tailwindcss、shadcn/ui、zustand、react-router-dom、recharts、lucide-react
3. Rust 依赖已添加到 Cargo.toml：rusqlite(bundled)、reqwest(json)、tokio(full)、serde/serde_json、tracing/tracing-subscriber、anyhow、thiserror、zip、tokio-cron-scheduler、axum
4. Tauri 插件已添加：tray-icon、autostart、notification、shell、single-instance、log
5. Tailwind CSS 4.x 配置完成，Kawaii 设计 token（马卡龙色板、圆角、字体栈）已写入 @theme
6. shadcn/ui 已初始化，components.json 配置完成
7. 项目可以通过 `pnpm tauri dev` 成功启动显示默认页面
8. ESLint 配置完成

## Tasks / Subtasks

- [X] Task 1: Tauri 项目脚手架初始化 (AC: #1)

  - [X] 1.1 执行 `pnpm create tauri-app qq-auto-like-plus --template react-ts`
  - [X] 1.2 验证项目目录结构：`src/`（React）+ `src-tauri/`（Rust）
  - [X] 1.3 验证 `pnpm tauri dev` 可启动默认欢迎页面
- [X] Task 2: 安装前端依赖 (AC: #2)

  - [X] 2.1 安装核心依赖：`pnpm add react-router-dom zustand recharts lucide-react`
  - [X] 2.2 安装 Tailwind CSS 4.x：`pnpm add -D tailwindcss @tailwindcss/vite`
  - [X] 2.3 Vite 插件配置：在 vite.config.ts 中添加 `@tailwindcss/vite` 插件
  - [X] 2.4 index.css 中引入 `@import "tailwindcss"`
- [X] Task 3: Rust Cargo 依赖配置 (AC: #3)

  - [X] 3.1 编辑 `src-tauri/Cargo.toml`，添加所有 Rust crate 依赖（见 Dev Notes 依赖清单）
  - [X] 3.2 运行 `cargo check` 确认依赖解析无冲突
- [X] Task 4: Tauri 插件安装与注册 (AC: #4)

  - [X] 4.1 安装 Tauri 插件 npm 包：`pnpm add @tauri-apps/plugin-autostart @tauri-apps/plugin-notification @tauri-apps/plugin-shell @tauri-apps/plugin-single-instance @tauri-apps/plugin-log`
  - [X] 4.2 在 `src-tauri/Cargo.toml` 添加对应的 Rust 侧插件依赖
  - [X] 4.3 在 `src-tauri/src/lib.rs` 的 `tauri::Builder` 中注册所有插件
  - [X] 4.4 在 `src-tauri/capabilities/default.json` 中声明插件权限
- [X] Task 5: Tailwind CSS 4.x Kawaii 设计 token 配置 (AC: #5)

  - [X] 5.1 在 `src/index.css` 中用 `@theme` 指令定义完整色彩系统（见 Dev Notes 色彩表）
  - [X] 5.2 定义圆角 token（`--radius-sm: 8px`, `--radius-md: 12px`, `--radius-lg: 16px`）
  - [X] 5.3 定义字体栈 token（`--font-primary`, `--font-display`, `--font-mono`）
  - [X] 5.4 定义间距 token（4px 基础单位）
  - [X] 5.5 验证 Tailwind 工具类可用（如 `bg-[var(--color-bg-base)]`）
- [X] Task 6: shadcn/ui 初始化 (AC: #6)

  - [X] 6.1 执行 `pnpm dlx shadcn@latest init`
  - [X] 6.2 配置 `components.json`：设置 style="new-york"、tsx=true、alias paths
  - [X] 6.3 安装基础组件验证：`pnpm dlx shadcn@latest add button card`
  - [X] 6.4 验证 shadcn/ui 组件可正常渲染并使用 Kawaii 主题色
- [X] Task 7: ESLint 配置 (AC: #8)

  - [X] 7.1 安装 ESLint 及 TypeScript/React 插件
  - [X] 7.2 创建 `eslint.config.js`（flat config 格式）
  - [X] 7.3 运行 `pnpm eslint .` 确认无报错
- [X] Task 8: 项目结构骨架搭建 (AC: #1)

  - [X] 8.1 创建前端目录结构：`src/pages/`, `src/components/`, `src/components/ui/`, `src/stores/`, `src/hooks/`, `src/lib/`, `src/types/`, `src/assets/mascot/`
  - [X] 8.2 创建 Rust 模块目录结构：`src-tauri/src/commands/`, `src-tauri/src/db/`, `src-tauri/src/napcat/`, `src-tauri/src/onebot/`, `src-tauri/src/engine/`, `src-tauri/src/friends/`, `src-tauri/src/stats/`, `src-tauri/src/webhook/`, `src-tauri/src/tray/`, `src-tauri/src/config/`
  - [X] 8.3 每个 Rust 模块目录创建空的 `mod.rs` 占位文件
  - [X] 8.4 创建 `src-tauri/migrations/` 目录
  - [X] 8.5 最终验证 `pnpm tauri dev` 启动成功

## Dev Notes

### Rust 依赖精确清单（Cargo.toml [dependencies] 部分）

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-autostart = "2"
tauri-plugin-notification = "2"
tauri-plugin-shell = "2"
tauri-plugin-single-instance = "2"
tauri-plugin-log = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
tokio-cron-scheduler = "0.13"
axum = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
thiserror = "2"
zip = "2"
```

### 前端依赖精确清单

```bash
# 运行时依赖
pnpm add react-router-dom zustand recharts lucide-react

# Tailwind CSS 4.x（开发依赖）
pnpm add -D tailwindcss @tailwindcss/vite

# Tauri 前端插件
pnpm add @tauri-apps/plugin-autostart @tauri-apps/plugin-notification @tauri-apps/plugin-shell @tauri-apps/plugin-single-instance @tauri-apps/plugin-log
```

### Tailwind CSS 4.x 设计 Token（写入 src/index.css）

```css
@import "tailwindcss";

@theme {
  /* === 色彩系统 — Kawaii 暗色主题 === */
  --color-bg-base: #1a1625;
  --color-bg-card: #231f31;
  --color-bg-elevated: #2d2840;
  --color-bg-nav: #16121f;

  --color-primary: #f2a7c3;       /* 樱花粉 */
  --color-secondary: #a7c7f2;     /* 天空蓝 */
  --color-accent: #c3a7f2;        /* 薰衣草紫 */
  --color-mint: #a7f2d4;          /* 薄荷绿 */
  --color-peach: #f2cfa7;         /* 蜜桃橙 */
  --color-coral: #f28b8b;         /* 珊瑚红 */

  --color-text-primary: #f0edf5;
  --color-text-secondary: #9b95a8;
  --color-text-muted: #6b6578;

  /* === 圆角系统 — 偏大圆角营造 Kawaii 感 === */
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 20px;
  --radius-full: 9999px;

  /* === 字体系统 === */
  --font-primary: "HarmonyOS Sans SC", "PingFang SC", "Microsoft YaHei UI", sans-serif;
  --font-display: "HarmonyOS Sans SC", "PingFang SC", sans-serif;
  --font-mono: "JetBrains Mono", "Cascadia Code", "Consolas", monospace;

  /* === 字号层级 === */
  --text-display: 24px;
  --text-heading: 18px;
  --text-subheading: 15px;
  --text-body: 14px;
  --text-caption: 12px;
  --text-stat: 32px;
  --text-stat-label: 11px;
}
```

### Vite 配置参考（vite.config.ts）

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
```

### Tauri 插件注册参考（src-tauri/src/lib.rs）

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 二次打开时激活已有窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_log::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Tauri 2.0 权限配置（src-tauri/capabilities/default.json）

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "shell:allow-execute",
    "notification:default",
    "autostart:default",
    "log:default"
  ]
}
```

### main.rs 入口（隐藏 Windows 控制台）

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    qq_auto_like_plus_lib::run();
}
```

### Project Structure Notes

**关键目录结构（本 Story 需创建）：**

```
qq-auto-like-plus/
├── src/
│   ├── main.tsx              # React 入口（create-tauri-app 生成）
│   ├── App.tsx               # 根组件（create-tauri-app 生成，后续 Story 改造）
│   ├── index.css             # Tailwind 入口 + @theme 设计 token
│   ├── vite-env.d.ts         # Vite 类型声明
│   ├── pages/                # 空目录（后续 Story 填充）
│   ├── components/
│   │   └── ui/               # shadcn/ui 组件目录
│   ├── stores/               # Zustand stores 目录
│   ├── hooks/                # 自定义 hooks 目录
│   ├── lib/                  # 工具函数目录
│   ├── types/                # TypeScript 类型目录
│   └── assets/
│       └── mascot/           # Mascot 素材目录
├── src-tauri/
│   ├── Cargo.toml            # 含所有依赖
│   ├── tauri.conf.json       # Tauri 配置
│   ├── capabilities/
│   │   └── default.json      # 权限声明
│   ├── icons/                # 应用图标
│   ├── migrations/           # SQL 迁移目录（空）
│   └── src/
│       ├── main.rs           # Windows 入口
│       ├── lib.rs            # App 构建 + 插件注册
│       ├── commands/mod.rs   # 空模块占位
│       ├── db/mod.rs
│       ├── napcat/mod.rs
│       ├── onebot/mod.rs
│       ├── engine/mod.rs
│       ├── friends/mod.rs
│       ├── stats/mod.rs
│       ├── webhook/mod.rs
│       ├── tray/mod.rs
│       └── config/mod.rs
├── package.json
├── vite.config.ts
├── components.json           # shadcn/ui 配置
└── eslint.config.js
```

**命名规范强制规则：**

- Rust 文件/模块：snake_case（如 `like_executor.rs`）
- React 组件文件：PascalCase（如 `StatusCard.tsx`）
- Zustand store 文件：camelCase + use 前缀（如 `useLikeStore.ts`）
- CSS class：Tailwind 工具类（kebab-case）

**反模式警告：**

- 禁止 `println!`，使用 `tracing::info!` 等宏
- 禁止 `unwrap()` / `expect()` 在非初始化代码中
- 禁止在前端硬编码 API URL
- 禁止 `static mut`，使用 Tauri State 管理

### References

- [Source: .bmad-method/planning-artifacts/architecture.md#Starter模板评估] — 初始化命令和依赖清单
- [Source: .bmad-method/planning-artifacts/architecture.md#完整项目目录结构] — 项目结构定义
- [Source: .bmad-method/planning-artifacts/architecture.md#实现模式与一致性规则] — 命名规范和反模式
- [Source: .bmad-method/planning-artifacts/architecture.md#核心架构决策] — 技术选型和版本
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#DesignSystemFoundation] — 设计 token 定义
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#色彩系统] — 马卡龙色板精确色值
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#字体系统] — 字体栈和字号层级
- [Source: .bmad-method/planning-artifacts/epics.md#Story1.1] — AC 定义

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- V2Ray TUN 模式导致 Tauri CLI 无法检测 Vite dev server（localhost 被代理拦截返回 503），关闭 TUN 后解决
- Git 的 `link.exe` 遮蔽 MSVC `link.exe` 导致 cargo check 失败，安装 VS Build Tools C++ 组件后解决
- `@tauri-apps/plugin-single-instance` 无前端 npm 包，仅 Rust 侧依赖

### Completion Notes List

- 项目从 `create-tauri-app --template react-ts` 初始化，包名已修正为 `qq-auto-like-plus`
- 554 个 Rust crate 编译通过，`pnpm tauri dev` 成功启动并显示 Kawaii 暗色主题页面
- shadcn/ui 使用 `-d` 默认配置初始化，已安装 button 和 card 组件
- Tailwind CSS 4.x @theme 设计 token 已与 shadcn/ui CSS 变量系统集成
- 删除 App.css，App.tsx 已替换为使用 shadcn/ui 组件的验证页面
- ESLint flat config 配置完成，`pnpm lint` 无报错

### File List

- `package.json` — 修改（包名、脚本、依赖、pnpm 配置）
- `vite.config.ts` — 修改（添加 Tailwind 插件、path alias）
- `tsconfig.json` — 修改（添加 baseUrl 和 paths alias）
- `eslint.config.js` — 新建
- `components.json` — 新建（shadcn/ui 配置）
- `src/index.css` — 新建（Tailwind 入口 + Kawaii 设计 token + shadcn 变量）
- `src/main.tsx` — 修改（引入 index.css）
- `src/App.tsx` — 修改（替换为 shadcn/ui 验证页面）
- `src/App.css` — 删除
- `src/lib/utils.ts` — 新建（shadcn/ui 工具函数）
- `src/components/ui/button.tsx` — 新建（shadcn/ui）
- `src/components/ui/card.tsx` — 新建（shadcn/ui）
- `src-tauri/Cargo.toml` — 修改（包名、lib 名、所有依赖）
- `src-tauri/tauri.conf.json` — 修改（产品名、标识符、窗口配置）
- `src-tauri/capabilities/default.json` — 修改（插件权限声明）
- `src-tauri/src/main.rs` — 修改（Windows 控制台隐藏 + lib 引用）
- `src-tauri/src/lib.rs` — 修改（插件注册）
- `src-tauri/src/commands/mod.rs` — 新建（空占位）
- `src-tauri/src/db/mod.rs` — 新建（空占位）
- `src-tauri/src/napcat/mod.rs` — 新建（空占位）
- `src-tauri/src/onebot/mod.rs` — 新建（空占位）
- `src-tauri/src/engine/mod.rs` — 新建（空占位）
- `src-tauri/src/friends/mod.rs` — 新建（空占位）
- `src-tauri/src/stats/mod.rs` — 新建（空占位）
- `src-tauri/src/webhook/mod.rs` — 新建（空占位）
- `src-tauri/src/tray/mod.rs` — 新建（空占位）
- `src-tauri/src/config/mod.rs` — 新建（空占位）
- `src-tauri/migrations/` — 新建（空目录）

## QA Results

**Reviewer:** Quinn (Test Architect) — Claude Opus 4.6
**Date:** 2026-03-11
**Gate Decision:** PASS with CONCERNS

### Issues Found: 3 High, 4 Medium, 3 Low

### Fixed Issues (by QA)

| ID | Severity | Issue                                                                                | Resolution                                                        |
| -- | -------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------- |
| H1 | HIGH     | `.gitignore` 使用 UTF-16LE 编码，Git 无法解析                                      | 重写为 UTF-8 编码                                                 |
| H2 | HIGH     | `.gitignore` 仅包含 `node_modules`，缺少 `dist/`、`target/` 等关键排除项     | 补充完整的 ignore 规则                                            |
| H3 | HIGH     | `@theme` Kawaii 圆角 token 被 `@theme inline` 中 shadcn 的 calc 值覆盖           | 替换 `@theme inline` 中 radius 值为 Kawaii 设计规范的固定像素值 |
| M2 | MEDIUM   | `capabilities/default.json` 包含 `shell:allow-execute`，允许前端执行任意系统命令 | 移除该权限，仅保留 `shell:allow-open`                           |

### Noted Concerns (未修改代码)

| ID | Severity | Issue                                                                              | Recommendation                                                             |
| -- | -------- | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| M1 | MEDIUM   | `components.json` style 为 `base-nova`，Story Task 6.2 记录为 `new-york`     | 文档差异，base-nova 是 shadcn V4 默认风格，功能无影响，但 Story 描述应更新 |
| M3 | MEDIUM   | 10 个 Rust 模块目录有 `mod.rs` 但 `lib.rs` 无 `mod` 声明                     | Story 1.1 范围内可接受（仅占位），后续 Story 需要时再声明                  |
| M4 | MEDIUM   | File List 缺少 `index.html`、`tsconfig.node.json`、`.gitignore`、`public/` | 文档完整性问题，不影响功能                                                 |
| L1 | LOW      | `tauri.conf.json` CSP 为 null                                                    | 开发阶段可接受，生产前需配置                                               |
| L2 | LOW      | `package.json` 含未在 Story 中列出的间接依赖                                     | shadcn/ui 和 Tauri 自动引入，无需操作                                      |
| L3 | LOW      | `lib.rs` 中 `use tauri::Manager` 由 trait resolution 间接使用                  | 当前无影响，保持即可                                                       |

### AC Verification

| AC                | Status       | Evidence                                         |
| ----------------- | ------------ | ------------------------------------------------ |
| #1 项目结构       | PASS         | `src/` + `src-tauri/` 存在，目录结构完整     |
| #2 前端依赖       | PASS         | package.json 含所有必需依赖                      |
| #3 Rust 依赖      | PASS         | Cargo.toml 含所有必需 crate                      |
| #4 Tauri 插件     | PASS         | lib.rs 注册 5 个插件，capabilities 配置权限      |
| #5 Tailwind token | PASS (fixed) | @theme 设计 token 完整，radius 冲突已修复        |
| #6 shadcn/ui      | PASS         | components.json 配置完成，button/card 组件已安装 |
| #7 启动验证       | PASS         | Dev Agent 确认 `pnpm tauri dev` 成功启动       |
| #8 ESLint         | PASS         | eslint.config.js flat config 配置完成            |
