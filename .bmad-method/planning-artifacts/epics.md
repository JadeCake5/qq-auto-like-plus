---
stepsCompleted: [1, 2, 3, 4]
status: 'complete'
completedAt: '2026-03-10'
inputDocuments:
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
  - .bmad-method/planning-artifacts/architecture.md
  - .bmad-method/planning-artifacts/ux-design-specification.md
---

# qq-auto-like-plus - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for qq-auto-like-plus, decomposing the requirements from the PRD, UX Design, and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: 首次运行时自动下载 NapCat Shell OneKey 包并显示下载进度（US-001）
FR2: 下载完成后自动解压到应用数据目录（US-001）
FR3: 启动 NapCat 进程并展示扫码登录界面（US-001）
FR4: 扫码成功后自动检测 QQ 号并保存配置，进入引导向导（US-001）
FR5: 可配置定时触发时间，默认每天 00:05（US-002）
FR6: 获取完整好友列表并随机打乱顺序（US-002）
FR7: 逐个点赞，每人间隔可配置（默认 3 秒），每人次数可配置（默认 10 次）（US-002）
FR8: 跳过当天已赞过的好友（US-002）
FR9: 遵守每日总名额限制（US-002/US-004）
FR10: 点赞过程中实时更新面板状态（US-002）
FR11: 异常时记录日志并继续处理下一个好友（US-002）
FR12: 接收 NapCat Webhook 推送的 profile_like 事件（US-003）
FR13: 自动对点赞者执行回赞，回赞次数和延迟可配置（US-003）
FR14: 回赞使用独立的预留名额池（US-003/US-004）
FR15: 当天已赞过的好友不重复回赞（US-003）
FR16: 可配置每日点赞总人数上限（默认 50 人）（US-004）
FR17: 可配置为回赞预留的名额数（默认 10 人）（US-004）
FR18: 名额在每日零点自动重置（US-004）
FR19: 面板实时显示今日名额使用情况（已用/总量）（US-004）
FR20: 系统托盘显示图标，颜色反映状态（绿/黄/红）（US-005）
FR21: 托盘右键菜单：打开面板、立即点赞、暂停/恢复、NapCat 状态、退出（US-005）
FR22: 双击托盘图标打开管理面板（US-005）
FR23: 关闭面板窗口时最小化到托盘（US-005）
FR24: 退出前弹出确认对话框（US-005）
FR25: 仪表盘显示今日统计卡片：已赞人数、回赞人数、手动点赞数、剩余名额（US-006）
FR26: 仪表盘显示 NapCat 连接状态（US-006）
FR27: 仪表盘显示当前登录 QQ 号和昵称（US-006）
FR28: 仪表盘显示下次定时点赞倒计时（US-006）
FR29: 仪表盘提供"立即点赞"快捷按钮（US-006）
FR30: 仪表盘提供"暂停/恢复"开关（US-006）
FR31: 设置面板包含点赞设置、回赞设置、系统设置、NapCat 设置四组（US-007）
FR32: 设置修改后实时生效（热更新），无需重启（US-007）
FR33: 设置提供"恢复默认"按钮（US-007）
FR34: 实时显示运行日志（自动滚动到底部）（US-008）
FR35: 日志分级显示：INFO（蓝）、WARN（黄）、ERROR（红）（US-008）
FR36: 日志支持关键字搜索和级别过滤（US-008）
FR37: 日志持久化到文件（按日期轮转），提供"清空日志"按钮（US-008）
FR38: 应用启动时自动启动 NapCat Shell 进程（US-009）
FR39: 定期健康检查 NapCat（每 30 秒调用 /get_login_info）（US-009）
FR40: NapCat 异常退出时自动重启（最多 3 次，间隔递增）（US-009）
FR41: 超过重启次数后发送系统通知告警（US-009）
FR42: 检测到 QQ 掉线（30 天强制重登）时通知提醒扫码（US-009）
FR43: 好友列表页展示所有好友（头像、昵称、备注、标签、今日是否已赞）（US-010）
FR44: 支持创建、编辑、删除自定义标签（US-010）
FR45: 每个标签可配置独立点赞策略（次数、优先级、是否参与点赞/回赞）（US-010）
FR46: 新好友自动归入"默认"标签（US-010）
FR47: 数据统计面板：日/周/月视图，点赞趋势图表（US-011）
FR48: 点赞类型占比饼图（定时/回赞/手动）（US-011）
FR49: 好友互动排行 TOP 10（US-011）
FR50: 数据至少保留 90 天（US-011）
FR51: 设置页提供"开机自启"开关（US-012）
FR52: 自启后直接最小化到系统托盘（US-012）
FR53: 自启后自动连接 NapCat 并恢复运行状态（US-012）

### NonFunctional Requirements

NFR1: 应用本体安装包大小 < 10 MB
NFR2: 应用运行时内存占用 < 50 MB（不含 NapCat）
NFR3: 冷启动到托盘就绪 < 3 秒（不含 NapCat）
NFR4: 面板页面加载 < 1 秒
NFR5: NapCat 异常 3 次重试内自动恢复
NFR6: 异常退出后 SQLite 数据完整（WAL 模式）
NFR7: 兼容 Windows 10 x64 及以上
NFR8: WebView2 运行时自动安装（Tauri 内置）
NFR9: QQ 密码/Token 不存储，由 NapCat 管理 session
NFR10: 日志轮转单文件 < 10 MB，保留 7 天

### Additional Requirements

**架构要求：**
- AR1: 使用 `pnpm create tauri-app qq-auto-like-plus --template react-ts` 初始化项目
- AR2: SQLite WAL 模式 + 嵌入式版本化迁移（启动时自动执行）
- AR3: Tauri IPC（invoke + events）前后端通信
- AR4: axum 作为 Webhook 服务器（与 Tokio 运行时统一）
- AR5: tokio-cron-scheduler 定时任务调度
- AR6: Tauri State 管理共享状态（Arc<Mutex<T>>）
- AR7: 所有 Rust 结构体 serde(rename_all = "camelCase") 序列化
- AR8: tracing + tauri-plugin-log 统一日志
- AR9: thiserror（库层）+ anyhow（应用层）分层错误处理
- AR10: Zustand 域级状态管理（每个功能域独立 store）
- AR11: tauri-plugin-single-instance 防止多开
- AR12: tauri-plugin-notification 系统通知

**UX 设计要求：**
- UX1: Kawaii/ACG 暗色主题（马卡龙色系 + 深紫底色 #1a1625）
- UX2: Mascot 角色状态系统（happy/sleeping/worried/cheering/waiting/waving）
- UX3: shadcn/ui 组件 Kawaii 定制（偏大圆角、渐变、弹性微交互）
- UX4: 侧边栏图标导航（56px）+ 主内容区布局（900×600 固定窗口）
- UX5: 渐变统计卡片 + sparkline 迷你趋势图
- UX6: 撒花/庆祝微交互动画
- UX7: WCAG 2.1 AA 无障碍（对比度 ≥ 4.5:1）
- UX8: 键盘导航 + ARIA 标签 + focus-visible 样式
- UX9: NapCat 术语隐藏（对用户展示为"运行环境"）
- UX10: 情感化空状态（Mascot 插画替代纯文字）

### FR Coverage Map

FR1-FR4: Epic 1 — 首次启动引导（NapCat 下载、扫码登录、引导向导）
FR5-FR11: Epic 2 — 定时批量点赞（调度、执行、名额、进度）
FR16-FR19: Epic 2 — 名额管理（配置、重置、显示）
FR12-FR15: Epic 5 — 自动回赞（Webhook、回赞逻辑、预留名额）
FR20-FR24: Epic 3 — 系统托盘与后台运行
FR38-FR42: Epic 3 — NapCat 进程管理（健康检查、重启、通知）
FR51-FR53: Epic 3 — 开机自启
FR25-FR30: Epic 4 — 仪表盘
FR31-FR33: Epic 4 — 设置面板
FR34-FR37: Epic 4 — 日志页面
FR43-FR46: Epic 6 — 好友管理与标签系统
FR47-FR50: Epic 7 — 数据统计与可视化

## Epic List

### Epic 1: 项目初始化与首次启动引导
用户可以双击 exe 启动应用，自动完成运行环境下载和扫码登录，开始使用应用。
**FRs covered:** FR1, FR2, FR3, FR4
**架构要求:** AR1, AR2, AR3, AR6, AR7, AR8, AR9

### Epic 2: 核心点赞引擎与名额管理
应用可以每天自动定时为好友点赞，合理管理每日名额，实时推送点赞进度。
**FRs covered:** FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR16, FR17, FR18, FR19
**架构要求:** AR5

### Epic 3: 系统托盘、后台运行与进程管理
应用像 QQ 一样在系统托盘静默运行，自动管理 NapCat 进程生命周期，支持开机自启。
**FRs covered:** FR20, FR21, FR22, FR23, FR24, FR38, FR39, FR40, FR41, FR42, FR51, FR52, FR53
**架构要求:** AR11, AR12

### Epic 4: 管理面板（仪表盘、设置、日志）
用户可以通过面板查看运行状态、修改所有配置、查看运行日志。
**FRs covered:** FR25, FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR33, FR34, FR35, FR36, FR37
**UX 要求:** UX1, UX2, UX3, UX4, UX5, UX7, UX8, UX10

### Epic 5: 自动回赞
有人赞了用户后自动赞回去，维护社交礼尚往来。
**FRs covered:** FR12, FR13, FR14, FR15
**架构要求:** AR4

### Epic 6: 好友管理与标签系统
用户可以给好友打标签分组，对不同好友使用不同的点赞策略。
**FRs covered:** FR43, FR44, FR45, FR46

### Epic 7: 数据统计与可视化
用户可以查看历史点赞数据和趋势图表，了解社交互动情况。
**FRs covered:** FR47, FR48, FR49, FR50

## Epic 1: 项目初始化与首次启动引导

用户可以双击 exe 启动应用，自动完成运行环境下载和扫码登录，开始使用应用。这是整个产品的基础，包括项目脚手架搭建、数据库初始化、NapCat 环境管理和首次引导 UI。

### Story 1.1: 使用 Tauri 模板初始化项目并安装核心依赖

As a 开发者,
I want 使用官方 create-tauri-app 初始化项目并安装所有核心依赖,
So that 项目有一个干净、标准的基础结构可以开始开发。

**Acceptance Criteria:**

**Given** 开发环境已安装 Node.js、pnpm 和 Rust 工具链
**When** 执行 `pnpm create tauri-app qq-auto-like-plus --template react-ts`
**Then** 项目结构包含 `src/`（React 前端）和 `src-tauri/`（Rust 后端）
**And** 前端依赖已安装：tailwindcss、shadcn/ui、zustand、react-router-dom、recharts、lucide-react
**And** Rust 依赖已添加到 Cargo.toml：rusqlite(bundled)、reqwest(json)、tokio(full)、serde/serde_json、tracing/tracing-subscriber、anyhow、thiserror、zip、tokio-cron-scheduler、axum
**And** Tauri 插件已添加：tray-icon、autostart、notification、shell、single-instance、log
**And** Tailwind CSS 4.x 配置完成，Kawaii 设计 token（马卡龙色板、圆角、字体栈）已写入 @theme
**And** shadcn/ui 已初始化，components.json 配置完成
**And** 项目可以通过 `pnpm tauri dev` 成功启动显示默认页面
**And** ESLint 配置完成

### Story 1.2: SQLite 数据库层与应用配置管理

As a 用户,
I want 应用有可靠的数据存储和配置管理,
So that 我的设置和数据不会丢失。

**Acceptance Criteria:**

**Given** 项目基础结构已就位（Story 1.1）
**When** 应用首次启动
**Then** SQLite 数据库文件在 `%APPDATA%/qq-auto-like-plus/data.db` 创建
**And** 数据库启用 WAL 模式
**And** 嵌入式迁移自动执行，创建初始表：config、daily_state
**And** config 表写入所有默认配置值（daily_limit=50、times_per_friend=10、schedule_hour=0、schedule_minute=5 等）
**And** db 模块提供 CRUD 封装函数，其他模块通过 db 模块访问数据
**And** 数据库连接通过 Tauri State 注入，线程安全（Arc<Mutex<Connection>>）
**And** 提供 Tauri commands：get_config、update_config，前端可通过 invoke 读写配置
**And** 配置更新后通过 Tauri event emit("config:updated") 通知前端
**And** 错误处理：db 层使用 thiserror，command 层返回 Result<T, String>

### Story 1.3: NapCat 下载与解压

As a 新用户,
I want 应用自动下载运行环境而不需要我手动操作,
So that 我不需要了解任何技术细节。

**Acceptance Criteria:**

**Given** 应用检测到 NapCat 目录不存在（`%APPDATA%/qq-auto-like-plus/napcat/`）
**When** 首次启动触发下载流程
**Then** 从预配置的 URL 下载 NapCat Shell OneKey 包（.zip）
**And** 下载进度通过 Tauri event emit("napcat:download-progress") 实时推送（百分比、速度、剩余时间）
**And** 下载完成后自动解压到 `%APPDATA%/qq-auto-like-plus/napcat/` 目录
**And** 解压进度通过 Tauri event 推送
**And** 下载失败时记录日志并通知前端，提供"重试"和"手动导入"选项
**And** 支持用户手动指定本地 NapCat 安装包路径（跳过下载）
**And** 对用户展示为"正在准备运行环境..."（隐藏 NapCat 术语）
**And** 提供 Tauri command：download_napcat、get_napcat_status

### Story 1.4: NapCat 进程启动与扫码登录

As a 用户,
I want 扫码登录 QQ 后应用自动开始工作,
So that 我只需要扫一次码就能用起来。

**Acceptance Criteria:**

**Given** NapCat 已解压到目标目录（Story 1.3）
**When** 启动 NapCat 进程
**Then** 自动生成 NapCat OneBot 配置文件（HTTP API 端口 3000、Webhook 端口 8080、仅监听 localhost）
**And** 通过 Tauri shell 插件启动 NapCat Shell 子进程
**And** NapCat 生成的二维码图片路径通过 Tauri event 推送给前端展示
**And** 前端展示二维码和 Mascot 等待动画
**And** 轮询 `/get_login_info` 检测登录状态
**And** 扫码成功后获取 QQ 号和昵称，保存到 config 表
**And** 登录成功通过 Tauri event emit("napcat:login-success") 通知前端
**And** 应用退出时优雅停止 NapCat 子进程

### Story 1.5: 首次启动引导 UI

As a 新用户,
I want 一个简单的引导流程告诉我应用的功能和基本设置,
So that 我能快速了解应用并开始使用。

**Acceptance Criteria:**

**Given** 用户首次启动应用且环境准备完成
**When** 进入引导界面
**Then** Setup 页面分步展示：欢迎（Mascot 挥手）→ 下载运行环境（进度条）→ 扫码登录（二维码展示）→ 基本设置（定时时间、每人次数）→ 完成（Mascot 欢呼 + 撒花）
**And** 每个步骤有进度指示器
**And** Mascot 角色在每个步骤有对应表情/动作
**And** 基本设置步骤使用合理默认值，用户可直接跳过
**And** 完成后自动进入 Dashboard 页面
**And** 引导状态保存到 config 表，后续启动不再显示
**And** 页面使用 Kawaii 暗色主题（深紫底色 + 马卡龙色点缀）
**And** 支持键盘导航（Tab 切换、Enter 确认）

## Epic 2: 核心点赞引擎与名额管理

应用可以每天自动定时为好友点赞，合理管理每日名额，实时推送点赞进度。这是产品的核心业务逻辑。

### Story 2.1: OneBot API 客户端

As a 应用,
I want 有一个可靠的 OneBot API 通信层,
So that 可以调用 NapCat 的点赞和查询接口。

**Acceptance Criteria:**

**Given** NapCat 进程已启动并监听 HTTP API（端口 3000）
**When** 应用需要调用 OneBot API
**Then** onebot/client.rs 封装 reqwest HTTP POST 请求
**And** 支持三个端点：`/send_like`（点赞）、`/get_friend_list`（好友列表）、`/get_login_info`（登录检查）
**And** 请求/响应类型定义在 onebot/types.rs，使用 serde(rename_all = "camelCase")
**And** 请求超时设置为 10 秒
**And** 网络错误使用 thiserror 定义具体错误枚举（ConnectionRefused、Timeout、ApiError）
**And** 提供重试机制（最多 2 次，间隔 1 秒）
**And** 所有 API 调用记录 tracing 日志（info 级别请求、error 级别失败）

### Story 2.2: 名额管理模块

As a 用户,
I want 应用合理管理每天的点赞数量,
So that 降低风控风险。

**Acceptance Criteria:**

**Given** 数据库 daily_state 表和 config 表已就位
**When** 新的一天开始（零点）
**Then** engine/quota.rs 自动重置当日名额（创建新的 daily_state 记录）
**And** 可用名额 = daily_limit - reserved_for_reply
**And** 每次点赞消耗名额时更新 daily_state.total_liked 和对应计数
**And** 回赞消耗从预留池扣减（daily_state.reply_count）
**And** 名额耗尽时返回 QuotaExhausted 错误，不执行点赞
**And** 提供查询接口：get_quota_status() 返回今日已用/剩余/总量
**And** 提供 Tauri command：get_daily_stats 供前端查询
**And** 创建 like_history 表用于记录每次点赞结果

### Story 2.3: 定时批量点赞执行器

As a 用户,
I want 应用自动为我的好友点赞,
So that 我不需要手动操作。

**Acceptance Criteria:**

**Given** OneBot 客户端和名额管理模块已就位
**When** 触发批量点赞（定时或手动）
**Then** engine/like_executor.rs 通过 `/get_friend_list` 获取完整好友列表
**And** 随机打乱好友顺序
**And** 逐个调用 `/send_like`，每人之间间隔 batch_interval 秒（默认 3 秒）
**And** 每人点赞 times_per_friend 次（默认 10 次）
**And** 跳过当天已赞过的好友（查询 like_history 表）
**And** 每次点赞前检查名额，名额耗尽时停止
**And** 每次点赞结果写入 like_history 表（user_id、times、like_type='scheduled'、success、error_msg）
**And** 更新 daily_state 计数
**And** 单个好友点赞失败时记录日志并继续下一个（不中断整体流程）
**And** 通过 Tauri event emit("like:progress") 实时推送进度（当前/总数、当前好友昵称）
**And** 批量完成后 emit("like:batch-complete") 通知前端
**And** 创建 friends 表存储好友信息

### Story 2.4: 定时任务调度器

As a 用户,
I want 应用在我配置的时间自动开始点赞,
So that 我不需要手动触发。

**Acceptance Criteria:**

**Given** 批量点赞执行器已就位（Story 2.3）
**When** 应用启动
**Then** engine/scheduler.rs 使用 tokio-cron-scheduler 注册定时任务
**And** 定时时间从 config 表读取（默认 00:05）
**And** 到达定时时间时自动触发批量点赞
**And** 配置热更新：前端修改定时时间后，scheduler 重新注册任务（无需重启）
**And** 提供 Tauri commands：start_batch_like（手动触发）、pause_engine、resume_engine
**And** 暂停状态持久化到 config 表，重启后恢复
**And** 提供查询：get_next_run_time() 返回下次执行时间
**And** 调度器状态变化通过 Tauri event emit 通知前端

## Epic 3: 系统托盘、后台运行与进程管理

应用像 QQ 一样在系统托盘静默运行，自动管理 NapCat 进程生命周期，支持开机自启。用户无需关注底层进程状态。

### Story 3.1: 系统托盘基础（图标与右键菜单）

As a 用户,
I want 应用在系统托盘显示图标并提供快捷操作,
So that 应用不占用任务栏空间，且我能快速操作。

**Acceptance Criteria:**

**Given** 应用已启动
**When** 应用初始化完成
**Then** 系统托盘显示应用图标
**And** 图标颜色反映当前状态：绿色=运行中、黄色=登录中或下载中、红色=异常
**And** 右键菜单包含：打开面板、立即点赞、暂停/恢复、NapCat 状态（显示文本）、退出
**And** "立即点赞"调用 start_batch_like command
**And** "暂停/恢复"切换引擎状态并更新菜单文字
**And** 托盘图标状态通过监听 Tauri events 自动更新
**And** 准备三套图标文件：tray-green.png、tray-yellow.png、tray-red.png

### Story 3.2: 面板窗口管理与托盘交互

As a 用户,
I want 双击托盘打开面板，关闭面板回到托盘,
So that 面板像 QQ 一样方便地打开和关闭。

**Acceptance Criteria:**

**Given** 应用在系统托盘运行中
**When** 用户双击托盘图标
**Then** 管理面板窗口弹出（如果已隐藏则显示，如果不存在则创建）
**And** 面板窗口固定大小 900×600，不可调整
**And** 面板窗口居中显示
**When** 用户点击面板窗口的关闭按钮（×）
**Then** 面板窗口隐藏（最小化到托盘），不退出应用
**When** 用户通过托盘菜单选择"退出"
**Then** 弹出确认对话框："确定退出 QQ Auto Like Plus？退出后将停止自动点赞。"
**And** 确认后优雅停止 NapCat 进程，关闭数据库连接，退出应用
**And** 取消则返回托盘运行

### Story 3.3: NapCat 健康检查与自动重启

As a 用户,
I want 应用自动管理运行环境的稳定性,
So that 我不需要手动排查问题。

**Acceptance Criteria:**

**Given** NapCat 进程已启动并运行
**When** 应用处于运行状态
**Then** napcat/process.rs 每 30 秒调用 `/get_login_info` 进行健康检查
**And** 健康检查失败时判断 NapCat 进程是否存活
**And** NapCat 进程异常退出时自动重启（最多 3 次）
**And** 重启间隔递增：第 1 次 5 秒、第 2 次 15 秒、第 3 次 30 秒
**And** 每次重启通过 Tauri event emit("napcat:status-changed") 通知前端
**And** 超过 3 次重启失败后通过 tauri-plugin-notification 发送 Windows 系统通知告警
**And** 托盘图标切换为红色
**And** 检测到 QQ 掉线（30 天强制重登）时发送系统通知提醒扫码
**And** 掉线通知通过 emit("napcat:login-required") 通知前端弹出扫码界面
**And** 所有状态变化记录 tracing 日志

### Story 3.4: 开机自启与单实例

As a 用户,
I want 应用随开机自动启动,
So that 我不用每次手动打开。

**Acceptance Criteria:**

**Given** 用户在设置中开启"开机自启"
**When** Windows 系统启动
**Then** 应用通过 tauri-plugin-autostart 自动启动
**And** 自启后直接最小化到系统托盘（不弹出面板窗口）
**And** 自启后自动启动 NapCat 并恢复上次运行状态（暂停/运行）
**And** tauri-plugin-single-instance 防止多开，二次打开时激活已有实例窗口
**And** 设置页的"开机自启"开关通过 Tauri command 控制 autostart 插件
**And** 开关状态持久化到 config 表

## Epic 4: 管理面板（仪表盘、设置、日志）

用户可以通过管理面板直观地查看运行状态、修改所有配置参数、查看运行日志。面板使用 Kawaii/ACG 暗色主题。

### Story 4.1: 应用布局框架与路由

As a 用户,
I want 面板有清晰的导航和一致的布局,
So that 我能快速找到需要的功能。

**Acceptance Criteria:**

**Given** 用户打开管理面板
**When** 面板窗口显示
**Then** 左侧显示 56px 宽的图标导航栏（深色背景 #16121f）
**And** 导航栏包含 5 个图标按钮：仪表盘、好友管理、数据统计、运行日志、设置
**And** 当前选中的导航项有高亮样式（渐变背景 + 发光效果）
**And** 鼠标悬停图标显示 tooltip 文字
**And** 右侧主内容区根据路由渲染对应页面
**And** 使用 React Router v7 管理路由（/dashboard、/friends、/statistics、/logs、/settings）
**And** 默认路由为 /dashboard
**And** 顶部 24px 状态条显示 NapCat 连接状态指示器 + 登录 QQ 信息
**And** 全局样式使用 Kawaii 暗色主题（深紫底色 #1a1625、马卡龙色点缀）
**And** 所有文字使用 HarmonyOS Sans SC 字体栈
**And** 支持键盘导航（Tab 切换导航项，Enter 选中）
**And** 创建 Zustand stores：useLikeStore、useNapCatStore、useSettingsStore、useLogStore
**And** 创建 useTauriEvent hook 封装 Tauri 事件监听

### Story 4.2: 仪表盘页面

As a 用户,
I want 一个直观的仪表盘查看运行状态,
So that 我能一秒获取关键数据。

**Acceptance Criteria:**

**Given** 用户进入仪表盘页面
**When** 页面加载
**Then** 顶部 Hero Banner 区域（120px）显示 Mascot 角色 + 欢迎文案 + 状态一句话
**And** Mascot 表情根据状态切换：😊 运行中、😴 暂停、😰 异常
**And** 显示 4 张渐变统计卡片（双列布局）：
  - 今日已赞人数（樱花渐变 #f2a7c3→#c3a7f2）
  - 回赞人数（蜜桃渐变 #f2cfa7→#f2a7c3）
  - 剩余名额（天空渐变 #a7c7f2→#a7f2d4）
  - 下次点赞倒计时（薰衣草渐变 #c3a7f2→#f2a7c3）
**And** 统计数据通过 invoke("get_daily_stats") 获取
**And** 显示"立即点赞"按钮和"暂停/恢复"开关
**And** "立即点赞"按钮点击后调用 invoke("start_batch_like")
**And** 点赞进度通过监听 like:progress 事件实时更新
**And** NapCat 连接状态实时显示（已连接/断开/重连中）
**And** 当前登录 QQ 号和昵称显示在状态区
**And** 数据自动刷新（每 30 秒 + 事件驱动）
**And** 页面加载 < 1 秒
**And** 空状态使用 Mascot 插画（"还没有开始点赞哦~"）

### Story 4.3: 设置面板

As a 用户,
I want 通过面板修改配置,
So that 不需要手动编辑文件。

**Acceptance Criteria:**

**Given** 用户进入设置页面
**When** 页面加载
**Then** 设置项分为 4 个分组卡片（圆角 14px、深色背景）：
**And** **点赞设置组**：每日名额（滑块 1-200）、每人次数（滑块 1-20）、定时时间（时:分选择器）、批次间隔（滑块 1-60 秒）
**And** **回赞设置组**：回赞开关（Switch）、回赞次数（滑块 1-20）、预留名额（滑块 0-100）、回赞延迟范围（双滑块 0-60 秒）
**And** **系统设置组**：开机自启开关、最小化到托盘开关
**And** **运行环境设置组**：NapCat 路径（显示当前路径 + 修改按钮）、API 地址、端口配置
**And** 所有设置显示当前值，修改后即时保存（调用 invoke("update_config")）
**And** 保存成功显示 toast 通知（Kawaii 样式）
**And** 设置修改通过 config:updated 事件实时生效，无需重启
**And** 每个分组卡片右上角有"恢复默认"按钮
**And** 表单校验：数值范围、必填项
**And** 对用户隐藏 NapCat 术语，展示为"运行环境"

### Story 4.4: 运行日志页面

As a 用户,
I want 查看运行日志,
So that 出现问题时能排查原因。

**Acceptance Criteria:**

**Given** 用户进入日志页面
**When** 页面加载
**Then** 显示实时运行日志列表（自动滚动到底部）
**And** 日志通过监听 log:entry 事件实时追加
**And** 日志按级别着色：INFO（天空蓝 #a7c7f2）、WARN（蜜桃橙 #f2cfa7）、ERROR（珊瑚红 #f28b8b）
**And** 顶部工具栏包含：搜索框（关键字过滤）、级别下拉筛选（全部/INFO/WARN/ERROR）、"清空日志"按钮
**And** 搜索实时过滤当前显示的日志条目
**And** 日志条目格式：[时间] [级别] 消息内容
**And** 日志使用等宽字体（JetBrains Mono / Cascadia Code）
**And** Rust 后端使用 tracing + tauri-plugin-log 记录日志
**And** 日志持久化到文件（%APPDATA%/qq-auto-like-plus/logs/）
**And** 日志按日期轮转，单文件 < 10 MB，保留 7 天
**And** "清空日志"仅清空当前显示，不影响日志文件
**And** 大量日志（>1000 条）使用虚拟滚动优化性能

## Epic 5: 自动回赞

有人赞了用户后自动赞回去，维护社交礼尚往来。需要 Webhook 服务器接收 NapCat 推送的事件。

### Story 5.1: Webhook 服务器与事件接收

As a 用户,
I want 应用能接收到别人给我点赞的通知,
So that 可以自动赞回去。

**Acceptance Criteria:**

**Given** NapCat 已配置 Webhook 回调地址（http://127.0.0.1:8080）
**When** 应用启动
**Then** webhook/mod.rs 使用 axum 启动 HTTP 服务器监听 webhook_port（默认 8080）
**And** 仅监听 127.0.0.1（localhost），不暴露到网络
**And** 接收 POST /webhook 请求
**And** 解析 OneBot 事件 JSON，识别 `notice.notify.profile_like` 事件类型
**And** 提取点赞者 user_id
**And** 非 profile_like 事件静默忽略
**And** Webhook 端口可通过设置页配置
**And** 端口被占用时记录错误日志并通知用户
**And** 事件接收记录 tracing 日志

### Story 5.2: 回赞处理逻辑

As a 用户,
I want 有人赞我后自动赞回去,
So that 维护社交互动的礼尚往来。

**Acceptance Criteria:**

**Given** Webhook 服务器收到 profile_like 事件（Story 5.1）
**When** 触发回赞流程
**Then** engine/reply_handler.rs 检查回赞预留名额是否充足
**And** 检查该好友当天是否已被赞过（查询 like_history），已赞则跳过
**And** 检查回赞开关是否开启（config 表）
**And** 添加可配置的随机延迟（reply_delay_min ~ reply_delay_max 秒，默认即时）
**And** 调用 `/send_like` 执行回赞，次数为 reply_times（默认 10 次）
**And** 回赞结果写入 like_history 表（like_type='reply'）
**And** 更新 daily_state.reply_count
**And** 回赞成功/失败记录 tracing 日志
**And** 通过 Tauri event 通知前端更新回赞计数

## Epic 6: 好友管理与标签系统

用户可以给好友打标签分组，对不同好友使用不同的点赞策略。提供好友列表展示和标签管理功能。

### Story 6.1: 好友列表展示与同步

As a 用户,
I want 查看我的 QQ 好友列表,
So that 了解哪些好友在被点赞。

**Acceptance Criteria:**

**Given** 用户进入好友管理页面
**When** 页面加载
**Then** 通过 invoke("get_friends") 获取好友列表数据
**And** 首次加载时调用 `/get_friend_list` 同步好友信息到 friends 表
**And** 好友列表卡片显示：头像（圆形 40px）、昵称、备注名、标签（彩色 badge）、今日是否已赞（绿色勾/灰色叉）
**And** 支持搜索框按昵称/备注筛选
**And** 支持按标签筛选（标签下拉多选）
**And** 好友列表使用虚拟滚动（支持 500+ 好友）
**And** 新好友自动归入"默认"标签（friends 表新增记录时自动关联）
**And** 创建 useFriendsStore（Zustand）管理好友和标签状态
**And** 空状态显示 Mascot 插画（"还没有好友数据，请先登录 QQ~"）

### Story 6.2: 标签 CRUD 与好友标签管理

As a 用户,
I want 创建自定义标签并给好友打标签,
So that 可以对不同好友进行分组管理。

**Acceptance Criteria:**

**Given** 好友管理页面已加载
**When** 用户操作标签
**Then** 页面顶部显示标签管理区域，列出所有标签（彩色 badge）
**And** 支持创建标签：输入名称、选择颜色（马卡龙色选择器）
**And** 支持编辑标签：修改名称和颜色
**And** 支持删除标签：确认对话框后删除，关联的 friend_tags 记录级联删除
**And** 预置标签：默认（不可删除）、重要、不赞
**And** 好友列表中每个好友可点击添加/移除标签（多选弹出面板）
**And** 支持多标签：一个好友可以同时属于多个标签
**And** 标签操作通过 Tauri commands 持久化到 tags 和 friend_tags 表
**And** 操作成功显示 toast 通知
**And** 提供 Tauri commands：create_tag、update_tag、delete_tag、set_friend_tags

### Story 6.3: 基于标签的点赞策略

As a 用户,
I want 对不同标签的好友使用不同的点赞策略,
So that 重要的好友可以优先点赞，不想赞的可以跳过。

**Acceptance Criteria:**

**Given** 标签系统已就位（Story 6.2）
**When** 用户为标签配置点赞策略
**Then** 每个标签可配置：点赞次数（1-20）、优先级（高/中/低）、是否参与定时点赞（开关）、是否参与回赞（开关）
**And** 标签设置通过标签编辑面板修改
**And** friends/strategy.rs 在批量点赞时按策略排序好友：高优先级 → 中优先级 → 低优先级
**And** "不赞"标签的好友（auto_like=false）跳过所有定时点赞
**And** auto_reply=false 的标签好友跳过回赞
**And** 不同标签的好友使用各自的 like_times 而非全局默认值
**And** 好友同时属于多个标签时，取最高优先级标签的策略
**And** 策略变更通过 config:updated 事件通知引擎实时生效
**And** 提供 Tauri command：update_tag_strategy

## Epic 7: 数据统计与可视化

用户可以查看历史点赞数据和趋势图表，了解社交互动情况。

### Story 7.1: 统计数据聚合查询

As a 用户,
I want 应用能汇总我的点赞历史数据,
So that 可以看到直观的统计信息。

**Acceptance Criteria:**

**Given** like_history 和 daily_state 表有历史数据
**When** 前端请求统计数据
**Then** stats/queries.rs 提供以下聚合查询：
**And** 日视图：当天每小时点赞数分布（GROUP BY hour）
**And** 周视图：近 7 天每日点赞数（GROUP BY date）
**And** 月视图：近 30 天每日点赞数（GROUP BY date）
**And** 点赞类型占比：定时(scheduled) / 回赞(reply) / 手动(manual) 各自总数
**And** 好友互动排行：被赞次数 TOP 10（GROUP BY user_id ORDER BY count DESC）
**And** 数据保留策略：定期清理 90 天前的 like_history 记录
**And** 提供 Tauri commands：get_stats_daily、get_stats_weekly、get_stats_monthly、get_like_type_ratio、get_friend_ranking
**And** 查询使用 SQLite 索引优化（idx_like_history_created_at、idx_like_history_user_id）
**And** 创建 useStatsStore（Zustand）管理统计数据状态

### Story 7.2: 数据统计页面（图表可视化）

As a 用户,
I want 通过图表查看点赞趋势和数据分析,
So that 了解我的社交互动情况。

**Acceptance Criteria:**

**Given** 用户进入数据统计页面
**When** 页面加载
**Then** 顶部显示时间范围切换：日 / 周 / 月（Tab 样式，默认周视图）
**And** 主图表区域使用 Recharts 渲染：
  - 日视图：柱状图（每小时点赞数）
  - 周视图：折线图（近 7 天每日点赞趋势）
  - 月视图：折线图（近 30 天每日点赞趋势）
**And** 图表配色使用马卡龙色系（樱花粉折线 + 薰衣草紫填充区域）
**And** 旁边显示点赞类型占比饼图（定时/回赞/手动，三色区分）
**And** 底部显示好友互动排行 TOP 10 列表（头像 + 昵称 + 被赞次数 + 进度条）
**And** 数据通过 invoke 调用对应的 stats commands 获取
**And** 图表支持 hover 显示详细数值（tooltip）
**And** 无数据时显示 Mascot 空状态插画（"还没有点赞数据，等明天再来看看吧~"）
**And** 页面加载 < 1 秒
