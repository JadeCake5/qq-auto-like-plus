---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-03-10'
inputDocuments:
  - docs/project-brief.md
  - docs/prd/index.md
  - docs/prd/1-概述.md
  - docs/prd/2-用户画像.md
  - docs/prd/3-用户故事.md
  - docs/prd/4-功能规格详述.md
  - docs/prd/5-技术架构.md
  - docs/prd/6-非功能需求.md
  - docs/prd/7-用户体验流程.md
  - docs/prd/8-风险与缓解.md
  - docs/prd/9-成功指标.md
  - docs/prd/10-发布与分发.md
  - docs/prd/11-开放问题.md
  - docs/prd/12-参考文档.md
  - PROJECT_SUMMARY.md
workflowType: 'architecture'
project_name: 'qq-auto-like-plus'
user_name: 'Shira'
date: '2026-03-10'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## 项目上下文分析

### 需求概览

**功能需求：**
- 12 个用户故事（P0×9 核心功能 + P1×3 增强功能）
- 5 个功能域：环境引导、点赞引擎、系统集成、管理面板、数据管理
- 核心业务逻辑：定时全量点赞 + 事件驱动回赞 + 名额管理算法

**非功能需求：**
- 性能：安装包 <10MB、运行内存 <50MB、冷启动 <3s
- 可靠性：NapCat 异常自动恢复（3次重试）、SQLite WAL 防数据丢失
- 兼容性：Windows 10 x64+、WebView2 自动安装
- 安全性：不存储 QQ 密码/Token，由 NapCat 管理 session

**规模与复杂度：**
- 主要域：全栈桌面应用（Rust 后端 + React 前端 + 外部进程管理）
- 复杂度：中等
- 架构组件预估：~10 个核心模块

### 技术约束与依赖

| 约束 | 来源 | 影响 |
|------|------|------|
| Tauri 2.0 | 产品决策 | 决定了桌面框架、IPC 模型、插件体系 |
| Rust stable | 产品决策 | 后端语言、异步运行时选择（Tokio） |
| React + TypeScript | 产品决策 | 前端框架、构建工具（Vite） |
| NapCat Shell OneKey | 产品决策 | 外部进程管理复杂度、OneBot 11 API |
| SQLite | 产品决策 | 嵌入式存储、无需额外服务 |
| Windows Only (V1) | 范围约束 | 可使用 Windows 特有 API（注册表、系统托盘） |
| OneBot 11 HTTP | 协议约束 | HTTP 请求/响应模式 + Webhook 事件推送 |

### 跨切面关注点

1. **NapCat 生命周期管理** — 贯穿所有功能：下载→启动→监控→重启→停止
2. **Rust ↔ React 状态同步** — Tauri events 实时推送引擎状态到前端
3. **错误恢复与韧性** — 进程崩溃自动重启、网络中断重连、30天重登提醒
4. **配置热更新** — 前端修改设置后 Rust 后端即时生效，无需重启
5. **统一日志** — Rust 应用日志 + NapCat 进程日志归集到同一查看界面
6. **数据持久化** — SQLite 事务保证状态一致性，WAL 模式保证异常安全

## Starter 模板评估

### 主要技术域

桌面全栈应用：Tauri 2.0 (Rust) + React (TypeScript) + Vite

### 选定方案：官方 `create-tauri-app` (v4.7.0)

**选择理由：**
- 项目有高度特定的需求（NapCat 进程管理、Webhook 服务器），没有模板能覆盖
- 干净的起点避免与模板预设冲突
- 官方维护，版本更新有保障

**初始化命令：**

```bash
pnpm create tauri-app qq-auto-like-plus --template react-ts
```

**Starter 提供的基础决策：**
- 语言：Rust stable + TypeScript 5.x
- 构建：Vite 5.x（HMR 开发服务器）
- 结构：`src/` (React 前端) + `src-tauri/` (Rust 后端)

### 需要手动添加的依赖

**Tauri 官方插件：**

| 插件 | 用途 |
|------|------|
| tray-icon (内置) | 系统托盘图标与菜单 |
| autostart | 开机自启 |
| notification | 系统通知（掉线提醒） |
| shell | 启动 NapCat 子进程 |
| single-instance | 防止多开 |
| log | 统一日志 |

**Rust Crates：**

| Crate | 用途 |
|-------|------|
| rusqlite (bundled) | SQLite 数据库 |
| reqwest (json) | OneBot HTTP API 客户端 |
| tokio (full) | 异步运行时 |
| tokio-cron-scheduler | 定时任务 |
| serde / serde_json | 序列化 |
| zip | 解压 NapCat OneKey 包 |
| tracing / tracing-subscriber | 结构化日志 |

**前端依赖：**

| 包 | 用途 |
|----|------|
| tailwindcss | 样式方案 |
| shadcn/ui | UI 组件库 |
| zustand | 状态管理 |
| recharts | 数据统计图表 |
| react-router-dom | 页面路由 |
| lucide-react | 图标库 |

## 核心架构决策

### 决策优先级分析

**关键决策（阻塞实现）：**
- 数据访问层：rusqlite 原生
- 前后端通信：Tauri IPC (invoke + events)
- Webhook 服务器：axum

**重要决策（塑造架构）：**
- 前端：shadcn/ui + Zustand + React Router v7 + Recharts
- 日志：tracing + tauri-plugin-log
- 错误处理：anyhow + thiserror

**延迟决策（V1 后）：**
- 自动更新机制、多账号架构、插件系统

### 数据架构

| 决策 | 选型 | 理由 |
|------|------|------|
| 数据库 | SQLite 3.x (bundled) | 嵌入式、零配置、桌面应用首选 |
| 访问层 | rusqlite 原生 | 查询简单，无需 ORM 抽象开销 |
| 写入模式 | WAL | 并发读写安全、异常退出数据不丢失 |
| 迁移策略 | 嵌入式版本化迁移 | 启动时自动检查并执行，代码内管理 |

### 安全策略

| 决策 | 选型 | 理由 |
|------|------|------|
| QQ 凭据 | 不存储 | 由 NapCat 管理 session |
| Webhook 安全 | 仅监听 localhost | NapCat 和应用在同一机器 |
| 进程隔离 | NapCat 独立进程 | 崩溃不影响主应用 |

### API 与通信

| 决策 | 选型 | 理由 |
|------|------|------|
| 前后端通信 | Tauri IPC (invoke + events) | 标准 Tauri 模式，类型安全 |
| OneBot 调用 | reqwest HTTP POST | OneBot 11 仅需 HTTP |
| Webhook 服务器 | axum | 基于 Tokio，与 Tauri 异步运行时统一 |
| 实时状态推送 | Tauri events (emit) | 点赞进度、NapCat 状态→前端实时更新 |

### 前端架构

| 决策 | 选型 | 理由 |
|------|------|------|
| UI 组件库 | shadcn/ui | Tailwind 原生、可定制、体积极小 |
| 样式方案 | Tailwind CSS 4.x | 原子化 CSS、与 shadcn/ui 配合 |
| 状态管理 | Zustand | 轻量 (~1KB)、API 简洁 |
| 路由 | React Router v7 | 成熟稳定 |
| 图表 | Recharts | React 原生、声明式 |
| 图标 | Lucide React | shadcn/ui 默认图标集 |

### 基础设施与部署

| 决策 | 选型 | 理由 |
|------|------|------|
| CI/CD | GitHub Actions | Tauri 官方 Action 支持 |
| 安装包 | NSIS (.exe) + Portable (.zip) | 覆盖两种用户场景 |
| 日志 | tracing + tauri-plugin-log | 结构化、高性能、前后端统一 |
| 错误处理 | anyhow + thiserror | Rust 生态标准方案 |
| 单实例 | tauri-plugin-single-instance | 防止多开端口冲突 |

### 实现顺序

1. 项目初始化（create-tauri-app + 依赖安装）
2. SQLite 数据库层 + 迁移
3. 应用配置管理
4. NapCat 进程管理器
5. OneBot API 客户端
6. Webhook 服务器
7. 点赞引擎 + 定时调度
8. 系统托盘
9. React 前端面板
10. 好友标签系统
11. 数据统计面板
12. CI/CD + 打包发布

## 实现模式与一致性规则

### 冲突点识别

**已识别 6 类潜在 AI Agent 冲突领域：**
命名冲突、结构冲突、格式冲突、通信冲突、流程冲突、错误处理冲突

### 命名规范

**Rust 代码：**
| 元素 | 规范 | 示例 |
|------|------|------|
| 函数/方法 | snake_case | `get_friend_list()` |
| 结构体 | PascalCase | `LikeExecutor` |
| 模块/文件 | snake_case | `like_executor.rs` |
| 常量 | SCREAMING_SNAKE | `MAX_RETRY_COUNT` |
| Tauri command | snake_case | `#[tauri::command] fn get_daily_stats()` |

**TypeScript/React 代码：**
| 元素 | 规范 | 示例 |
|------|------|------|
| 组件 | PascalCase | `StatusCard.tsx` / `<StatusCard />` |
| 函数/变量 | camelCase | `getDailyStats()` |
| 常量 | SCREAMING_SNAKE | `API_BASE_URL` |
| Hook | camelCase + use 前缀 | `useLikeEngine()` |
| Store | camelCase + use 前缀 | `useLikeStore` |
| CSS class | kebab-case (Tailwind) | `class="text-sm font-bold"` |

**数据库命名：**
| 元素 | 规范 | 示例 |
|------|------|------|
| 表名 | snake_case 复数 | `friends`, `like_history`, `daily_state` |
| 列名 | snake_case | `user_id`, `created_at`, `like_type` |
| 外键 | {关联表单数}_id | `friend_id`, `tag_id` |
| 索引 | idx_{表}_{列} | `idx_like_history_user_id` |

**Tauri 事件命名：**
| 事件 | 格式 | 说明 |
|------|------|------|
| `like:progress` | namespace:action (kebab) | 单个好友点赞进度 |
| `like:batch-complete` | | 批量点赞完成 |
| `napcat:status-changed` | | NapCat 状态变化 |
| `napcat:login-required` | | 需要扫码登录 |
| `config:updated` | | 配置已热更新 |
| `log:entry` | | 新日志条目 |

### 结构模式

**Rust 模块组织：** 按功能域（feature-based）
- 每个功能域一个目录：`napcat/`, `engine/`, `onebot/`, `db/`, `webhook/`
- 每个目录必须有 `mod.rs` 作为公开接口
- 跨域共享类型放 `src/types.rs`

**React 组件组织：** 按页面 + 共享组件
- 页面组件：`src/pages/{PageName}.tsx`
- 共享组件：`src/components/{ComponentName}.tsx`
- UI 基础组件：`src/components/ui/` (shadcn/ui)
- 组件测试：与组件同目录 `{ComponentName}.test.tsx`

### 数据格式规范

**Rust ↔ JSON 序列化约定：**
所有 Rust 结构体必须添加 `#[serde(rename_all = "camelCase")]`，确保 JSON 输出为 camelCase：

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStats {
    pub total_liked: i32,       // → JSON: "totalLiked"
    pub scheduled_count: i32,   // → JSON: "scheduledCount"
    pub reply_count: i32,       // → JSON: "replyCount"
}
```

**日期时间格式：**
- 数据库存储：SQLite DATETIME（`CURRENT_TIMESTAMP`）
- JSON 传输：ISO 8601 字符串（`2026-03-10T00:05:00Z`）
- 前端显示：根据用户本地化格式化

**布尔值：**
- Rust/JSON：`true` / `false`
- SQLite：`0` / `1`（rusqlite 自动转换）

### 通信模式

**Tauri IPC 命令模式：**
- 前端调用：`invoke<ReturnType>("command_name", { args })`
- 命令返回：`Result<T, String>`（Tauri 要求错误类型为 String）
- 所有命令函数放在 `src-tauri/src/commands/` 下按功能域分文件

**Tauri 事件推送模式：**
- 后端推送：`app_handle.emit("event:name", payload)`
- 前端监听：`listen("event:name", callback)` → 存入 Zustand store

**Zustand Store 模式：** 每个域一个 store
```typescript
interface LikeStore {
  // 状态
  dailyStats: DailyStats | null;
  isRunning: boolean;
  // 操作
  fetchDailyStats: () => Promise<void>;
  startBatchLike: () => Promise<void>;
}

export const useLikeStore = create<LikeStore>((set) => ({
  dailyStats: null,
  isRunning: false,
  fetchDailyStats: async () => { /* invoke... */ },
  startBatchLike: async () => { /* invoke... */ },
}));
```

### 错误处理模式

**Rust 后端分层：**
| 层 | 方案 | 说明 |
|----|------|------|
| 库层（db, onebot, napcat） | `thiserror` | 定义具体错误枚举 |
| 应用层（engine, webhook） | `anyhow` | 错误链传播 |
| Tauri 命令层（commands/） | `Result<T, String>` | `.map_err(\|e\| e.to_string())` |

**前端错误处理：**
- Tauri invoke 异常统一 try-catch
- 用户可见错误通过 toast 通知展示
- 后台错误写入日志 store

### 强制规则（所有 AI Agent 必须遵守）

1. **Rust 结构体序列化**：所有对外暴露的结构体必须标注 `#[serde(rename_all = "camelCase")]`
2. **Tauri 命令签名**：返回值类型统一为 `Result<T, String>`
3. **事件命名**：严格使用 `namespace:action` 格式，namespace 对应功能域
4. **数据库表名**：snake_case 复数形式，列名 snake_case
5. **React 组件文件名**：PascalCase，与组件名一致
6. **Zustand store**：每个功能域独立 store，状态和操作在同一接口
7. **日志记录**：使用 `tracing` 宏（`info!`, `warn!`, `error!`），禁止 `println!`

### 反模式清单

| 不要这样做 | 应该这样做 |
|------------|-----------|
| `println!("debug info")` | `tracing::debug!("debug info")` |
| 在 Tauri command 中直接操作 DB | 通过 db 模块封装的函数访问 |
| 手动构造 JSON 字符串 | 使用 serde 序列化 |
| `unwrap()` / `expect()` 在生产代码 | `?` 操作符 + 错误传播 |
| 在前端硬编码 OneBot API URL | 通过 Tauri command 代理调用 |
| 全局可变状态 (`static mut`) | `Arc<Mutex<T>>` 或 Tauri State 管理 |
| 前端直接 HTTP 调用后端 | 统一通过 `invoke()` IPC 通信 |

## 项目结构与边界

### 需求到组件的映射

| 功能域 | 用户故事 | Rust 模块 | React 页面/组件 |
|--------|---------|-----------|----------------|
| 环境引导 | US-001 首次启动 | `napcat/` | `pages/Setup.tsx` |
| 点赞引擎 | US-002 定时点赞, US-003 回赞, US-004 名额 | `engine/`, `onebot/`, `webhook/` | `pages/Dashboard.tsx` |
| 系统集成 | US-005 托盘, US-009 进程管理, US-012 自启 | `tray/`, `napcat/` | — (系统级) |
| 管理面板 | US-006 仪表盘, US-007 设置, US-008 日志 | `commands/`, `config/` | `pages/Dashboard/Settings/Logs.tsx` |
| 数据管理 | US-010 标签, US-011 统计 | `friends/`, `stats/`, `db/` | `pages/Friends/Statistics.tsx` |

### 完整项目目录结构

```
qq-auto-like-plus/
├── .github/
│   └── workflows/
│       ├── ci.yml                    # PR 检查：lint + test + build
│       └── release.yml               # tag 触发：构建安装包 + GitHub Release
├── docs/
│   ├── prd/                          # 产品需求文档（已分片）
│   └── archive/                      # 归档文档
├── .bmad-method/
│   └── planning-artifacts/           # 架构文档等规划产物
│
├── src-tauri/                        # ===== Rust 后端 =====
│   ├── Cargo.toml                    # Rust 依赖声明
│   ├── Cargo.lock
│   ├── tauri.conf.json               # Tauri 配置（窗口、权限、插件）
│   ├── capabilities/                 # Tauri 2.0 权限声明
│   │   └── default.json
│   ├── icons/                        # 应用图标（各尺寸）
│   │   ├── icon.ico
│   │   ├── icon.png
│   │   └── tray-*.png                # 托盘图标（绿/黄/红）
│   ├── migrations/                   # SQL 迁移文件
│   │   └── 001_init.sql
│   └── src/
│       ├── main.rs                   # Windows 入口（隐藏控制台窗口）
│       ├── lib.rs                    # Tauri App 构建 + 插件注册
│       ├── types.rs                  # 跨模块共享类型定义
│       ├── errors.rs                 # 统一错误类型 (thiserror)
│       │
│       ├── commands/                 # Tauri IPC 命令层
│       │   ├── mod.rs
│       │   ├── like.rs               # 点赞相关：start_batch, get_progress
│       │   ├── friends.rs            # 好友管理：get_list, update_tags
│       │   ├── stats.rs              # 统计查询：daily, weekly, monthly
│       │   ├── settings.rs           # 设置读写：get_config, update_config
│       │   └── napcat.rs             # NapCat 管理：get_status, restart
│       │
│       ├── db/                       # 数据库访问层
│       │   ├── mod.rs                # 连接池初始化 + WAL 设置
│       │   ├── migrations.rs         # 版本化迁移执行器
│       │   └── models.rs             # CRUD 操作（friends, tags, history, state, config）
│       │
│       ├── napcat/                   # NapCat 进程管理
│       │   ├── mod.rs                # NapCatManager 公开接口
│       │   ├── downloader.rs         # 下载 + 解压 OneKey 包
│       │   ├── process.rs            # 进程启停 + 健康检查 + 自动重启
│       │   └── config.rs             # 生成 NapCat OneBot 配置文件
│       │
│       ├── onebot/                   # OneBot 11 API 客户端
│       │   ├── mod.rs
│       │   ├── client.rs             # reqwest HTTP 封装
│       │   └── types.rs              # OneBot 请求/响应结构体
│       │
│       ├── engine/                   # 业务引擎
│       │   ├── mod.rs                # LikeEngine 统一入口
│       │   ├── scheduler.rs          # tokio-cron-scheduler 定时任务
│       │   ├── like_executor.rs      # 批量点赞逻辑
│       │   ├── reply_handler.rs      # 回赞处理
│       │   └── quota.rs              # 名额管理算法
│       │
│       ├── friends/                  # 好友管理
│       │   ├── mod.rs
│       │   ├── tags.rs               # 标签 CRUD
│       │   └── strategy.rs           # 按标签的点赞策略
│       │
│       ├── stats/                    # 数据统计
│       │   ├── mod.rs
│       │   └── queries.rs            # 聚合查询
│       │
│       ├── webhook/                  # axum Webhook 服务器
│       │   └── mod.rs                # POST /webhook → profile_like → 触发回赞
│       │
│       ├── tray/                     # 系统托盘
│       │   └── mod.rs                # 图标状态切换 + 右键菜单
│       │
│       └── config/                   # 应用配置
│           └── mod.rs                # 从 SQLite config 表读写 + 热更新
│
├── src/                              # ===== React 前端 =====
│   ├── main.tsx                      # React 挂载入口
│   ├── App.tsx                       # 路由定义 + 全局 layout
│   ├── index.css                     # Tailwind 入口 + 全局样式
│   ├── vite-env.d.ts                 # Vite 类型声明
│   │
│   ├── pages/                        # 页面组件
│   │   ├── Dashboard.tsx             # 仪表盘
│   │   ├── Friends.tsx               # 好友管理
│   │   ├── Statistics.tsx            # 数据统计
│   │   ├── Logs.tsx                  # 运行日志
│   │   ├── Settings.tsx              # 设置面板
│   │   └── Setup.tsx                 # 首次启动引导
│   │
│   ├── components/                   # 共享组件
│   │   ├── ui/                       # shadcn/ui 基础组件
│   │   ├── Layout.tsx                # 应用布局（侧边栏 + 主内容区）
│   │   ├── StatusCard.tsx            # 状态统计卡片
│   │   ├── NapCatStatus.tsx          # NapCat 连接状态指示器
│   │   ├── FriendList.tsx            # 好友列表
│   │   ├── TagManager.tsx            # 标签管理
│   │   ├── ChartPanel.tsx            # 统计图表容器
│   │   ├── LogViewer.tsx             # 日志查看器
│   │   ├── QrCodeDisplay.tsx         # NapCat 二维码展示
│   │   └── DownloadProgress.tsx      # 下载进度条
│   │
│   ├── stores/                       # Zustand 状态管理
│   │   ├── useLikeStore.ts           # 点赞引擎状态
│   │   ├── useFriendsStore.ts        # 好友与标签状态
│   │   ├── useNapCatStore.ts         # NapCat 进程状态
│   │   ├── useSettingsStore.ts       # 应用配置状态
│   │   ├── useStatsStore.ts          # 统计数据状态
│   │   └── useLogStore.ts            # 日志状态
│   │
│   ├── hooks/                        # 自定义 hooks
│   │   ├── useTauriEvent.ts          # Tauri 事件监听封装
│   │   └── useTauriCommand.ts        # Tauri invoke 封装
│   │
│   ├── lib/                          # 工具函数
│   │   ├── api.ts                    # Tauri command 调用层
│   │   ├── constants.ts              # 前端常量
│   │   └── utils.ts                  # 通用工具
│   │
│   └── types/                        # TypeScript 类型定义
│       └── index.ts                  # 与 Rust 结构体对应的前端类型
│
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── components.json                   # shadcn/ui 配置
├── eslint.config.js
├── .gitignore
└── README.md
```

### 架构边界

**API 边界：**

| 边界 | 接口 | 方向 | 说明 |
|------|------|------|------|
| 前端 → Rust | Tauri `invoke()` | 同步请求/响应 | 所有前端操作通过 commands/ 层 |
| Rust → 前端 | Tauri `emit()` | 异步事件推送 | 点赞进度、状态变更 |
| Rust → NapCat | reqwest HTTP POST | 同步请求 | OneBot 11 API 调用 |
| NapCat → Rust | axum HTTP POST | 异步 Webhook | 事件回调（点赞通知） |
| Rust → SQLite | rusqlite | 同步（WAL） | 数据持久化 |

**组件边界规则：**
- `commands/` 是唯一的前端入口点 — 前端不直接访问任何其他 Rust 模块
- `db/models.rs` 是唯一的数据库访问点 — 其他模块不直接执行 SQL
- `onebot/client.rs` 是唯一的 OneBot API 出口 — 其他模块不直接发 HTTP 请求
- `webhook/` 收到事件后只调用 `engine/` — 不直接操作数据库或 OneBot

**数据流：**
```
[React 前端] ←invoke/emit→ [commands/] → [engine/] → [onebot/] → [NapCat]
                                ↕                        ↕
                             [config/]               [webhook/]
                                ↕                        ↑
                              [db/] ←←←←←←←←←←←←←←←←  [NapCat 推送]
```

### 需求到文件的精确映射

| 用户故事 | 关键文件 |
|---------|---------|
| US-001 首次启动 | `napcat/downloader.rs`, `napcat/config.rs`, `pages/Setup.tsx`, `QrCodeDisplay.tsx` |
| US-002 定时点赞 | `engine/scheduler.rs`, `engine/like_executor.rs`, `engine/quota.rs` |
| US-003 自动回赞 | `webhook/mod.rs`, `engine/reply_handler.rs` |
| US-004 名额管理 | `engine/quota.rs`, `db/models.rs (daily_state)` |
| US-005 系统托盘 | `tray/mod.rs`, `icons/tray-*.png` |
| US-006 仪表盘 | `commands/like.rs`, `commands/stats.rs`, `pages/Dashboard.tsx`, `StatusCard.tsx` |
| US-007 设置 | `commands/settings.rs`, `config/mod.rs`, `pages/Settings.tsx` |
| US-008 日志 | `pages/Logs.tsx`, `LogViewer.tsx` |
| US-009 NapCat 管理 | `napcat/process.rs`, `commands/napcat.rs`, `NapCatStatus.tsx` |
| US-010 好友标签 | `friends/tags.rs`, `friends/strategy.rs`, `pages/Friends.tsx`, `TagManager.tsx` |
| US-011 数据统计 | `stats/queries.rs`, `commands/stats.rs`, `pages/Statistics.tsx`, `ChartPanel.tsx` |
| US-012 开机自启 | `lib.rs` (tauri-plugin-autostart), `pages/Settings.tsx` |

### 跨切面关注点映射

| 关注点 | 涉及文件 |
|--------|---------|
| NapCat 生命周期 | `napcat/process.rs`, `napcat/downloader.rs`, `commands/napcat.rs`, `tray/mod.rs` |
| 状态同步 | 所有 `stores/*.ts` + Tauri events（`emit`/`listen`） |
| 错误恢复 | `napcat/process.rs`（重启）, `engine/like_executor.rs`（跳过失败） |
| 配置热更新 | `config/mod.rs` → `emit("config:updated")` → `useSettingsStore.ts` |
| 统一日志 | `tracing` + `tauri-plugin-log` → `emit("log:entry")` → `useLogStore.ts` |
| 数据持久化 | `db/mod.rs`（WAL）, `db/migrations.rs`, `db/models.rs` |

## 架构验证结果

### 一致性验证 ✅

**技术选型兼容性：**
- Tauri 2.0 + axum + reqwest + tokio-cron-scheduler 共享同一 Tokio 异步运行时，无冲突
- rusqlite (bundled) 通过 Tauri State 注入，线程安全
- shadcn/ui + Tailwind CSS 4.x 原生配合
- Zustand + React 18+ 兼容 concurrent mode
- serde `camelCase` 自动对齐 TypeScript 命名约定

**模式一致性：**
- 命名规范覆盖 Rust/TypeScript/DB/Events 四个层面，无遗漏
- Rust feature-based 模块组织与 Zustand 域级 store 一一对应
- 错误处理三层模型（thiserror → anyhow → String）链路清晰

**结构对齐：**
- 目录结构完整反映所有架构决策的技术选型
- `commands/` 作为唯一 IPC 桥梁，边界清晰
- `db/models.rs` 集中数据访问，防止 SQL 散落

### 需求覆盖验证 ✅

**用户故事覆盖：12/12 全覆盖**

| 用户故事 | 架构支撑 | 状态 |
|---------|---------|------|
| US-001 首次启动引导 | napcat/ 模块完整覆盖下载→解压→启动→扫码 | ✅ |
| US-002 定时全量点赞 | engine/scheduler + like_executor + quota | ✅ |
| US-003 自动回赞 | webhook/ → engine/reply_handler | ✅ |
| US-004 名额管理 | engine/quota + daily_state 表 | ✅ |
| US-005 系统托盘 | tray/ + tray-icon 插件 + 多状态图标 | ✅ |
| US-006 仪表盘 | Dashboard.tsx + StatusCard + NapCatStatus | ✅ |
| US-007 设置面板 | Settings.tsx + config/ + 热更新事件 | ✅ |
| US-008 运行日志 | Logs.tsx + LogViewer + tracing + log 插件 | ✅ |
| US-009 NapCat 管理 | napcat/process.rs（健康检查+重启+通知） | ✅ |
| US-010 好友标签 | friends/ + 3 张表（friends, tags, friend_tags） | ✅ |
| US-011 数据统计 | stats/queries.rs + Recharts + like_history 表 | ✅ |
| US-012 开机自启 | tauri-plugin-autostart | ✅ |

**非功能需求覆盖：8/8 全覆盖**

| NFR | 架构支撑 | 状态 |
|-----|---------|------|
| 安装包 <10MB | Tauri 2.0 base ~3MB + Rust 编译产物 | ✅ |
| 运行内存 <50MB | Rust 低开销 + Zustand ~1KB + SQLite 轻量 | ✅ |
| 冷启动 <3s | Tauri 原生窗口 + Vite 预构建 | ✅ |
| NapCat 3 次重试 | napcat/process.rs 递增间隔重启 | ✅ |
| SQLite WAL 防丢 | db/mod.rs 初始化启用 WAL 模式 | ✅ |
| Win10 x64+ | Tauri 2.0 WebView2 bootstrapper | ✅ |
| 不存储密码 | 设计层面排除，NapCat 管理 session | ✅ |
| 日志轮转 <10MB | tauri-plugin-log 配置轮转策略 | ✅ |

### 实现就绪度验证 ✅

**决策完整度：** 12 项关键/重要决策全部文档化，附带选型理由和版本
**结构完整度：** Rust 12 模块 30+ 文件 + React 6 页面 10+ 组件 6 stores
**模式完整度：** 7 条强制规则 + 7 条反模式 + 5 类命名/结构/格式/通信/错误模式

### 缺口分析

**关键缺口：** 无

**重要缺口（非阻塞，实现时解决）：**
1. **Rust 测试目录** — 模块内 `#[cfg(test)]` 单元测试 + 顶层 `tests/` 集成测试，可在实现时按需添加
2. **前端测试框架** — 建议 vitest（与 Vite 原生集成），实现时确定
3. **NapCat 二维码获取** — 需调研 OneKey 具体二维码输出机制（文件/stdout/API），实现 US-001 时解决

### 架构完整性清单

**✅ 需求分析**
- [x] 项目上下文深入分析
- [x] 规模与复杂度评估
- [x] 技术约束识别
- [x] 跨切面关注点映射

**✅ 架构决策**
- [x] 关键决策文档化（含版本号）
- [x] 技术栈完整指定
- [x] 集成模式定义
- [x] 性能考量覆盖

**✅ 实现模式**
- [x] 命名规范建立
- [x] 结构模式定义
- [x] 通信模式指定
- [x] 流程模式文档化

**✅ 项目结构**
- [x] 完整目录树定义
- [x] 组件边界建立
- [x] 集成点映射
- [x] 需求到结构的映射完成

### 架构就绪度评估

**总体状态：** 可进入实现阶段

**信心等级：** 高

**核心优势：**
- 技术栈成熟稳定，Tauri 2.0 + Rust + React 经过大量生产验证
- 模块边界清晰，功能域之间低耦合
- NapCat 进程隔离设计，崩溃不影响主应用
- WAL 模式 + 事务保证数据安全
- 完整的 AI Agent 实现规范，减少多人协作冲突

**未来增强方向：**
- 自动更新机制（V2）
- 多账号支持（V2）
- 跨平台扩展（macOS/Linux）
- 插件系统

### 实现交接指南

**AI Agent 实现指引：**
- 严格遵循本文档所有架构决策
- 使用实现模式章节的规范保证代码一致性
- 遵守项目结构和边界规则
- 所有架构疑问以本文档为准

**首要实现步骤：**
```bash
pnpm create tauri-app qq-auto-like-plus --template react-ts
```
按照「核心架构决策 → 实现顺序」章节的 12 步序列推进。
