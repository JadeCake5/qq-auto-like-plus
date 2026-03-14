# QQ Auto Like Plus — 产品需求文档 (PRD)

> 文档版本：v1.0 | 创建日期：2026-03-10 | 作者：John (Product Manager)
> 基于项目简报：`docs/project-brief.md`

---

## 1. 概述

### 1.1 产品名称

**QQ Auto Like Plus**

### 1.2 电梯演讲

一款 Windows 桌面应用程序，自动为 QQ 好友主页点赞。双击即用，后台静默运行，支持定时全量点赞、自动回赞、好友分组策略、数据统计，像一个真正的桌面小工具一样安静高效地工作。

### 1.3 问题陈述

QQ 好友之间的主页点赞是一种社交互动方式，但手动逐个点赞费时费力。现有的开源方案（包括本项目前身）存在以下问题：

| 痛点 | 影响 |
|------|------|
| 需要 Python 环境 + 手动装依赖 | 非技术用户无法使用 |
| Docker Desktop 体积大、配置复杂 | Windows 用户体验极差 |
| 前端是单文件内联 HTML | 难以维护、扩展、交互简陋 |
| 进程管理依赖 bat/vbs 脚本 | 不稳定、无法优雅重启 |
| 两个项目功能分散 | 维护成本高、功能不完整 |

### 1.4 产品愿景

**让 QQ 点赞像呼吸一样自然** — 用户只需双击一个 exe 文件，扫码登录后一切自动化，全程在系统托盘静默运行，需要时打开面板即可查看和管理。

---

## 2. 用户画像

### 2.1 主要用户：小明（QQ 社交活跃用户）

- **年龄**：16-30 岁
- **技术水平**：基础到中等，会安装和使用 Windows 软件，但不熟悉命令行或 Docker
- **使用场景**：希望自动为好友点赞以维护社交关系，同时希望"被赞的也赞回去"
- **核心诉求**：一键启动、不需要折腾、后台静默运行
- **设备**：Windows 10/11 个人电脑

### 2.2 次要用户：老王（技术爱好者）

- **年龄**：20-35 岁
- **技术水平**：较高，了解 OneBot 协议和 NapCat 生态
- **使用场景**：已有 NapCat 环境，希望集成更好的点赞管理工具
- **核心诉求**：可定制配置、数据统计、灵活的分组策略

---

## 3. 用户故事

### 3.1 P0 — 核心功能（必须实现）

#### US-001：首次启动与引导

> **作为**一个新用户，**我希望**双击 exe 后自动完成所有环境准备，**以便**我无需了解技术细节就能开始使用。

**验收标准：**
- [ ] 双击 exe 启动应用
- [ ] 检测到首次运行，自动下载 NapCat Shell OneKey 包（显示下载进度）
- [ ] 下载完成后自动解压到应用数据目录（`%APPDATA%/qq-auto-like-plus/napcat/`）
- [ ] 启动 NapCat 进程
- [ ] 弹出扫码登录界面（展示 NapCat 生成的二维码）
- [ ] 扫码成功后自动检测 QQ 号并保存到配置
- [ ] 进入主面板，展示引导向导（简要说明功能和基本设置）

#### US-002：定时全量点赞

> **作为**用户，**我希望**应用每天自动为我的 QQ 好友点赞，**以便**我不需要手动操作。

**验收标准：**
- [ ] 可配置定时触发时间（默认每天 00:05）
- [ ] 获取完整好友列表，随机打乱顺序
- [ ] 逐个点赞，每人之间可配置间隔（默认 3 秒）
- [ ] 每人点赞次数可配置（默认 10 次，范围 1-20）
- [ ] 跳过当天已赞过的好友
- [ ] 遵守每日总名额限制
- [ ] 点赞过程中实时更新面板状态
- [ ] 异常时记录日志并继续处理下一个好友

#### US-003：自动回赞

> **作为**用户，**我希望**当有人赞了我之后自动赞回去，**以便**维护社交互动的礼尚往来。

**验收标准：**
- [ ] 接收 NapCat webhook 推送的 `profile_like` 事件
- [ ] 自动对点赞者执行回赞
- [ ] 回赞次数可配置（默认 10 次，范围 1-20）
- [ ] 回赞使用独立的预留名额池
- [ ] 当天已赞过的好友不重复回赞
- [ ] 回赞延迟可配置（默认即时，可设置 0-60 秒随机延迟）

#### US-004：名额管理

> **作为**用户，**我希望**能控制每天的点赞数量，**以便**降低风控风险。

**验收标准：**
- [ ] 可配置每日点赞总人数上限（默认 50 人）
- [ ] 可配置为回赞预留的名额数（默认 10 人）
- [ ] 定时点赞消耗名额 = 总名额 - 回赞预留名额
- [ ] 名额在每日零点自动重置
- [ ] 面板实时显示今日名额使用情况（已用/总量）

#### US-005：系统托盘

> **作为**用户，**我希望**应用像 QQ 一样在系统托盘静默运行，**以便**不占用任务栏空间。

**验收标准：**
- [ ] 应用启动后在系统托盘显示图标
- [ ] 托盘图标颜色反映状态：绿色=运行中 / 黄色=登录中或下载中 / 红色=异常
- [ ] 右键菜单包含：打开面板、立即点赞、暂停/恢复、NapCat 状态、退出
- [ ] 双击托盘图标打开管理面板
- [ ] 关闭面板窗口时最小化到托盘（而非退出应用）
- [ ] 退出前弹出确认对话框

#### US-006：管理面板 — 仪表盘

> **作为**用户，**我希望**有一个直观的仪表盘查看运行状态，**以便**随时了解点赞进度。

**验收标准：**
- [ ] 显示今日统计卡片：已赞人数、回赞人数、手动点赞数、剩余名额
- [ ] 显示 NapCat 连接状态（已连接/断开/重连中）
- [ ] 显示当前登录的 QQ 号和昵称
- [ ] 显示下次定时点赞的倒计时
- [ ] 提供"立即点赞"快捷按钮
- [ ] 提供"暂停/恢复"开关

#### US-007：管理面板 — 设置

> **作为**用户，**我希望**通过面板修改配置，**以便**不需要手动编辑配置文件。

**验收标准：**
- [ ] 点赞设置：每日名额、每人次数、回赞预留、定时时间、批次间隔
- [ ] 回赞设置：开关、回赞次数、回赞延迟范围、分组过滤
- [ ] 系统设置：开机自启开关、最小化到托盘开关
- [ ] NapCat 设置：NapCat 路径、API 地址、端口配置
- [ ] 设置修改后实时生效（热更新），无需重启应用
- [ ] 提供"恢复默认"按钮

#### US-008：管理面板 — 日志

> **作为**用户，**我希望**查看运行日志，**以便**在出现问题时排查。

**验收标准：**
- [ ] 实时显示运行日志（自动滚动到底部）
- [ ] 日志分级显示：INFO（蓝）、WARN（黄）、ERROR（红）
- [ ] 支持关键字搜索过滤
- [ ] 支持日志级别过滤
- [ ] 日志持久化到文件（按日期轮转）
- [ ] 提供"清空日志"按钮

#### US-009：NapCat 进程管理

> **作为**用户，**我希望**应用自动管理 NapCat 的运行，**以便**我不需要手动启停。

**验收标准：**
- [ ] 应用启动时自动启动 NapCat Shell 进程
- [ ] 定期健康检查（每 30 秒调用 `/get_login_info`）
- [ ] NapCat 异常退出时自动重启（最多 3 次，间隔递增）
- [ ] 超过重启次数后发送系统通知告警
- [ ] 应用退出时优雅停止 NapCat 进程
- [ ] 检测到 QQ 掉线（30 天强制重登）时发送系统通知提醒扫码

### 3.2 P1 — 增强功能（V1 目标）

#### US-010：好友分组与标签

> **作为**用户，**我希望**给好友打标签分组，**以便**对不同好友使用不同的点赞策略。

**验收标准：**
- [ ] 好友列表页展示所有好友（头像、昵称、备注、标签、今日是否已赞）
- [ ] 支持创建自定义标签（如"重要"、"普通"、"不赞"）
- [ ] 支持为好友添加/移除标签（支持多标签）
- [ ] 支持按标签筛选好友列表
- [ ] 每个标签可配置独立的点赞策略：
  - 点赞次数（1-20）
  - 优先级（高/中/低）— 高优先级的好友先被点赞
  - 是否参与定时点赞
  - 是否参与回赞
- [ ] "不赞"标签的好友跳过所有自动点赞
- [ ] 新好友自动归入"默认"标签

#### US-011：数据统计面板

> **作为**用户，**我希望**查看历史点赞数据和趋势，**以便**了解我的社交互动情况。

**验收标准：**
- [ ] 日视图：当天每小时点赞分布柱状图
- [ ] 周视图：近 7 天每日点赞趋势折线图
- [ ] 月视图：近 30 天每日点赞趋势折线图
- [ ] 区分展示定时点赞、回赞、手动点赞的占比（饼图）
- [ ] 好友互动排行：被赞次数 TOP 10 好友列表
- [ ] 回赞率统计：赞了我但我没赞回的比例
- [ ] 数据至少保留 90 天

#### US-012：开机自启

> **作为**用户，**我希望**应用可以随开机自动启动，**以便**不用每次手动打开。

**验收标准：**
- [ ] 设置页提供"开机自启"开关
- [ ] 通过 Windows 注册表（`Run` 键）或启动文件夹实现
- [ ] 自启后直接最小化到系统托盘（不弹出面板）
- [ ] 自启后自动连接 NapCat 并恢复上次的运行状态

### 3.3 P2 — 未来迭代（V1 不含）

以下功能明确排除在 V1 范围外，记录于此作为后续路线图参考：

| 功能 | 说明 |
|------|------|
| 多账号支持 | 同时管理多个 QQ 小号的点赞任务 |
| 插件系统 | 类似旧 SendLike 的"赞我"指令响应 |
| 自动更新 | 应用内检测新版本并自动升级 |
| 数据导出 | 导出点赞历史为 CSV/Excel |
| 深色模式 | UI 暗色主题 |
| 跨平台 | macOS/Linux 支持 |

---

## 4. 功能规格详述

### 4.1 配置参数表

| 参数 | 类型 | 默认值 | 范围 | 说明 |
|------|------|--------|------|------|
| `daily_limit` | int | 50 | 1-200 | 每日点赞总人数上限 |
| `times_per_friend` | int | 10 | 1-20 | 每人点赞次数 |
| `schedule_hour` | int | 0 | 0-23 | 定时任务触发小时 |
| `schedule_minute` | int | 5 | 0-59 | 定时任务触发分钟 |
| `batch_interval` | int | 3 | 1-60 | 批量点赞间隔（秒） |
| `reserved_for_reply` | int | 10 | 0-100 | 为回赞预留的名额 |
| `reply_times` | int | 10 | 1-20 | 回赞次数 |
| `reply_delay_min` | int | 0 | 0-60 | 回赞最小延迟（秒） |
| `reply_delay_max` | int | 0 | 0-60 | 回赞最大延迟（秒） |
| `auto_start` | bool | false | — | 开机自启 |
| `minimize_to_tray` | bool | true | — | 关闭窗口时最小化到托盘 |
| `napcat_path` | string | auto | — | NapCat 安装路径 |
| `onebot_api_url` | string | `http://127.0.0.1:3000` | — | OneBot API 地址 |
| `webhook_port` | int | 8080 | 1024-65535 | Webhook 监听端口 |

### 4.2 API 依赖（OneBot 11）

| 端点 | 方法 | 用途 |
|------|------|------|
| `/send_like` | POST | 执行点赞 `{ user_id, times }` |
| `/get_friend_list` | POST | 获取好友列表 |
| `/get_login_info` | POST | 获取登录信息（健康检查） |

### 4.3 Webhook 事件

| 事件类型 | 触发条件 | 处理逻辑 |
|----------|---------|----------|
| `notice.notify.profile_like` | 有人给机器人点赞 | 触发回赞流程 |

---

## 5. 技术架构

### 5.1 技术栈

| 层 | 技术 | 版本 |
|----|------|------|
| 桌面框架 | Tauri | 2.x |
| 后端语言 | Rust | stable |
| 前端框架 | React | 18+ |
| 前端语言 | TypeScript | 5.x |
| 构建工具 | Vite | 5.x |
| 数据库 | SQLite | 3.x (via rusqlite) |
| 异步运行时 | Tokio | 1.x |
| HTTP 客户端 | reqwest | 0.12+ |
| 定时任务 | tokio-cron-scheduler | latest |
| 系统托盘 | Tauri tray plugin | 2.x |
| 自启动 | Tauri autostart plugin | 2.x |

### 5.2 模块划分

```
src-tauri/                     Rust 后端
├── src/
│   ├── main.rs                应用入口
│   ├── lib.rs                 Tauri 应用构建
│   ├── commands/              Tauri invoke 命令（前端调后端的桥梁）
│   │   ├── mod.rs
│   │   ├── like.rs            点赞相关命令
│   │   ├── friends.rs         好友管理命令
│   │   ├── stats.rs           统计查询命令
│   │   ├── settings.rs        设置命令
│   │   └── napcat.rs          NapCat 管理命令
│   ├── napcat/                NapCat 进程管理
│   │   ├── mod.rs
│   │   ├── downloader.rs      下载与解压
│   │   ├── process.rs         进程启停与监控
│   │   └── config.rs          NapCat 配置生成
│   ├── onebot/                OneBot API 客户端
│   │   ├── mod.rs
│   │   ├── client.rs          HTTP API 调用封装
│   │   └── types.rs           请求/响应类型定义
│   ├── engine/                业务引擎
│   │   ├── mod.rs
│   │   ├── scheduler.rs       定时任务编排
│   │   ├── like_executor.rs   点赞执行逻辑
│   │   ├── reply_handler.rs   回赞处理
│   │   └── quota.rs           名额管理
│   ├── friends/               好友管理
│   │   ├── mod.rs
│   │   ├── tags.rs            标签/分组 CRUD
│   │   └── strategy.rs        按标签的点赞策略
│   ├── stats/                 数据统计
│   │   ├── mod.rs
│   │   └── queries.rs         聚合查询
│   ├── db/                    数据库层
│   │   ├── mod.rs
│   │   ├── migrations.rs      表结构迁移
│   │   └── models.rs          数据模型
│   ├── webhook/               Webhook 服务器
│   │   └── mod.rs
│   ├── tray/                  系统托盘
│   │   └── mod.rs
│   └── config/                应用配置
│       └── mod.rs
│
src/                           React 前端
├── App.tsx                    路由入口
├── main.tsx                   React 挂载
├── pages/
│   ├── Dashboard.tsx          仪表盘
│   ├── Friends.tsx            好友管理
│   ├── Statistics.tsx         数据统计
│   ├── Logs.tsx               运行日志
│   └── Settings.tsx           设置
├── components/
│   ├── Layout.tsx             布局框架（侧边栏导航）
│   ├── StatusCard.tsx         状态卡片
│   ├── FriendList.tsx         好友列表
│   ├── TagManager.tsx         标签管理
│   ├── ChartPanel.tsx         图表组件
│   └── LogViewer.tsx          日志查看器
├── hooks/
│   └── useTauriCommand.ts     Tauri invoke 封装 hook
├── lib/
│   └── api.ts                 前端 API 层（调用 Tauri commands）
└── types/
    └── index.ts               共享类型定义
```

### 5.3 数据模型

```sql
-- 好友信息
CREATE TABLE friends (
    user_id     TEXT PRIMARY KEY,
    nickname    TEXT NOT NULL,
    remark      TEXT,
    avatar_url  TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 标签定义
CREATE TABLE tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT DEFAULT '#3B82F6',    -- 显示颜色
    like_times  INTEGER DEFAULT 10,        -- 该标签下好友的点赞次数
    priority    INTEGER DEFAULT 1,         -- 0=高 1=中 2=低
    auto_like   BOOLEAN DEFAULT 1,         -- 是否参与定时点赞
    auto_reply  BOOLEAN DEFAULT 1,         -- 是否参与回赞
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 好友-标签关联（多对多）
CREATE TABLE friend_tags (
    friend_id   TEXT NOT NULL,
    tag_id      INTEGER NOT NULL,
    PRIMARY KEY (friend_id, tag_id),
    FOREIGN KEY (friend_id) REFERENCES friends(user_id),
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 点赞历史
CREATE TABLE like_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     TEXT NOT NULL,
    times       INTEGER NOT NULL,          -- 本次点赞次数
    like_type   TEXT NOT NULL,             -- 'scheduled' | 'reply' | 'manual'
    success     BOOLEAN DEFAULT 1,
    error_msg   TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 每日状态
CREATE TABLE daily_state (
    date              TEXT PRIMARY KEY,    -- 'YYYY-MM-DD'
    total_liked       INTEGER DEFAULT 0,
    scheduled_count   INTEGER DEFAULT 0,
    reply_count       INTEGER DEFAULT 0,
    manual_count      INTEGER DEFAULT 0,
    failed_count      INTEGER DEFAULT 0
);

-- 应用配置（KV 存储）
CREATE TABLE config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 6. 非功能需求

| 类别 | 需求 | 目标值 | 验证方式 |
|------|------|--------|----------|
| **性能** | 应用本体安装包大小 | < 10 MB | 构建产物检查 |
| **性能** | 应用运行时内存占用 | < 50 MB（不含 NapCat） | 任务管理器观察 |
| **性能** | 冷启动到托盘就绪 | < 3 秒（不含 NapCat） | 计时测试 |
| **性能** | 面板页面加载 | < 1 秒 | 用户体感 |
| **可靠性** | NapCat 异常自动恢复 | 3 次重试内恢复 | 模拟 kill 进程测试 |
| **可靠性** | 数据不丢失 | 异常退出后状态完整 | 强制退出后验证 SQLite |
| **兼容性** | Windows 版本支持 | Windows 10 x64 及以上 | 多版本测试 |
| **兼容性** | WebView2 运行时 | 自动安装（Tauri 内置支持） | Win10 LTSC 测试 |
| **安全性** | QQ 密码/Token 不存储 | 由 NapCat 管理 session | 检查应用数据目录 |
| **可维护性** | 日志轮转 | 单文件 < 10 MB，保留 7 天 | 持续运行观察 |

---

## 7. 用户体验流程

### 7.1 首次启动流程

```
[双击 exe]
    │
    ▼
[检测运行环境]──── 已安装 NapCat ───→ [启动 NapCat]
    │                                      │
    │ 未安装                               │
    ▼                                      │
[显示欢迎页 + 下载进度]                    │
    │                                      │
    ▼                                      │
[下载 NapCat OneKey]                       │
    │                                      │
    ▼                                      │
[解压到数据目录]                           │
    │                                      │
    ▼                                      ▼
[启动 NapCat]──→ [展示扫码登录]──→ [登录成功]
                                           │
                                           ▼
                                   [引导设置向导]
                                           │
                                           ▼
                                   [进入仪表盘]
                                           │
                                           ▼
                                   [最小化到托盘]
```

### 7.2 日常运行流程

```
[开机自启 / 手动启动]
    │
    ▼
[系统托盘图标 (绿色)]
    │
    ├──── 定时触发 ────→ [获取好友列表] → [随机排序] → [逐个点赞] → [更新统计]
    │
    ├──── 收到赞 ──────→ [Webhook 通知] → [检查名额] → [执行回赞] → [更新统计]
    │
    ├──── 用户操作 ────→ [双击托盘] → [打开面板] → [查看/管理]
    │
    └──── 异常检测 ────→ [NapCat 掉线] → [自动重启] → [超次数则通知用户]
```

---

## 8. 风险与缓解

| 风险 | 概率 | 影响 | 缓解策略 |
|------|------|------|----------|
| **QQ 账号风控/封禁** | 中 | 高 | 应用内明确提示使用小号；提供保守的默认参数；支持自定义间隔降低频率 |
| **NapCat 版本不兼容** | 中 | 高 | 锁定经过验证的 NapCat 版本；支持用户手动指定路径；应用内检测 API 兼容性 |
| **NapCat 下载地址失效** | 低 | 中 | 支持用户手动导入 NapCat 安装包；提供备用下载镜像配置 |
| **30 天强制重新登录** | 确定 | 低 | 检测掉线 → 系统通知 → 引导用户扫码重登 |
| **WebView2 缺失** | 低 | 中 | Tauri 2.0 内置 WebView2 bootstrapper 自动安装 |
| **NapCat 进程残留** | 低 | 低 | 应用启动时检测并清理孤立的 NapCat 进程 |

---

## 9. 成功指标

由于本项目为个人/小众工具，成功指标以功能完整度和用户体验为主：

| 指标 | 目标 |
|------|------|
| 首次启动到正常运行 | 全程 < 5 分钟（含下载 NapCat） |
| 日常使用零干预 | 连续运行 7 天无需人工介入 |
| 点赞成功率 | > 95%（排除风控因素） |
| 应用崩溃率 | 0 次 / 周 |
| GitHub Stars | 发布后 3 个月内 > 50（参考指标） |

---

## 10. 发布与分发

### 10.1 分发渠道

- **GitHub Releases**：主要分发渠道
  - 提供 `.exe` 安装包（Tauri NSIS installer）
  - 提供 `.msi` 安装包（可选）
  - 提供便携版 `.zip`（免安装）

### 10.2 发布产物

| 文件 | 说明 |
|------|------|
| `qq-auto-like-plus_x.y.z_x64-setup.exe` | Windows x64 安装包 |
| `qq-auto-like-plus_x.y.z_x64_portable.zip` | 便携版 |
| `CHANGELOG.md` | 更新日志 |

### 10.3 版本策略

- 遵循语义化版本 (SemVer)：`MAJOR.MINOR.PATCH`
- V1.0.0：核心功能完整可用
- V1.x.0：增量功能迭代

---

## 11. 开放问题

| # | 问题 | 状态 | 决策 |
|---|------|------|------|
| 1 | QQ 每人每天可被赞的具体上限是否为 20 次？ | 待确认 | 默认 10 次，用户可自行调整至 1-20 |
| 2 | NapCat 版本更新时是否需要应用内通知用户？ | 待定 | V1 暂不实现自动更新，手动替换 |
| 3 | 是否需要支持代理（Proxy）下载 NapCat？ | 待定 | V1 暂不支持，后续按需添加 |

---

## 12. 参考文档

| 文档 | 路径/链接 |
|------|----------|
| 项目简报 | `docs/project-brief.md` |
| 旧项目分析 | `PROJECT_SUMMARY.md` |
| NapCat 官方文档 | https://napneko.github.io/ |
| NapCat Shell 教程 | https://napneko.github.io/guide/boot/Shell |
| Tauri 2.0 文档 | https://v2.tauri.app/ |
| OneBot 11 协议 | https://github.com/botuniverse/onebot-11 |
