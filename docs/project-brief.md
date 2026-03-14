# QQ Auto Like Plus — 项目简报

> 文档版本：v1.0 | 创建日期：2026-03-10 | 作者：Mary (Business Analyst)

---

## 1. 项目概述

### 1.1 项目名称
**QQ Auto Like Plus** — QQ 主页自动点赞桌面应用

### 1.2 项目愿景
将现有的 Python 脚本工具（qq-auto-like + SendLike）重构为一个现代化的 Windows 桌面应用程序，提供传统 exe 应用的完整体验：系统托盘图标、后台静默运行、美观的管理 UI 面板，让用户双击即用、零门槛部署。

### 1.3 项目背景
现有项目存在以下痛点：
- **部署复杂**：需要 Python 环境、手动安装依赖、配置 bat 脚本
- **前端简陋**：单文件 HTML 内联 CSS/JS，难以维护和扩展
- **进程管理粗暴**：依赖 `taskkill /T /F`、VBS 后台脚本等 hack 方案
- **Docker 依赖沉重**：Docker Desktop 在 Windows 上体积大（~4GB）、配置门槛高
- **两个项目分裂**：SendLike（插件）和 qq-auto-like（脚本）功能分散

### 1.4 目标用户
- Windows 个人用户，希望自动为 QQ 好友点赞
- 技术水平不限，期望"双击即用"的体验
- 通常使用小号运行机器人，主号正常使用 PC 版 QQ

---

## 2. 核心技术决策

| 决策项 | 选型 | 理由 |
|--------|------|------|
| **后端语言** | Rust | 高性能、安全、原生 Windows API 支持、优秀的进程管理 |
| **前端框架** | React + TypeScript | 生态丰富、组件库成熟、开发效率高 |
| **桌面框架** | Tauri 2.0 | 体积小（~3MB）、Rust 原生集成、系统托盘/自启动开箱支持 |
| **QQ 协议层** | NapCat Shell (Windows OneKey) | 内置 QQ 运行环境、无头运行、50~100MB 内存、无需 Docker |
| **OneBot 协议** | OneBot 11 HTTP API | 成熟稳定、NapCat 完整支持 |
| **数据存储** | SQLite (via rusqlite) | 轻量嵌入式、支持历史数据统计、无需额外服务 |

---

## 3. 功能规格

### 3.1 核心功能（从旧项目继承）

| 功能 | 描述 | 优先级 |
|------|------|--------|
| **定时全量点赞** | 每日定时获取好友列表，随机打乱顺序逐个点赞 | P0 |
| **自动回赞** | 接收 OneBot webhook 的 `profile_like` 事件，自动回赞 | P0 |
| **NapCat 进程管理** | 自动下载、启动、监控、重启 NapCat Shell 进程 | P0 |
| **扫码登录** | 首次使用引导扫码登录，自动检测 QQ 号并保存 | P0 |
| **系统托盘** | 最小化到托盘运行，右键菜单（打开面板/暂停/退出） | P0 |
| **Web 管理面板** | 状态查看、手动触发点赞、日志查看、配置管理 | P0 |
| **名额管理** | 每日总点赞上限、回赞预留名额、已赞跳过 | P0 |
| **开机自启** | 可选开机自动启动 | P1 |

### 3.2 新增功能（V1）

| 功能 | 描述 | 优先级 |
|------|------|--------|
| **好友分组/标签** | 对好友设置自定义标签，按分组配置不同点赞策略和优先级 | P1 |
| **数据统计面板** | 历史点赞记录、每日/每周/每月趋势图表、好友互动排行 | P1 |
| **自动回赞（增强）** | 增强回赞逻辑：可配置回赞延迟、回赞次数、仅回赞特定分组 | P1 |

### 3.3 非功能需求

| 需求 | 目标 |
|------|------|
| **安装体积** | 应用本体 < 10MB（不含 NapCat） |
| **内存占用** | 应用自身 < 50MB（NapCat 另计 50~100MB） |
| **启动速度** | 冷启动 < 3 秒（不含 NapCat 启动） |
| **兼容性** | Windows 10 (x64) 及以上 |
| **可靠性** | NapCat 异常时自动重启、状态持久化防丢失 |

---

## 4. 系统架构

```
qq-auto-like-plus.exe (Tauri 2.0 打包)
│
├─ Rust 后端 (Tauri Core)
│   │
│   ├─ napcat_manager        NapCat 进程生命周期管理
│   │   ├─ download()            首次运行自动下载 OneKey 包
│   │   ├─ start()               spawn NapCatWinBootMain.exe
│   │   ├─ monitor()             健康检查 + 异常自动重启
│   │   ├─ stop()                优雅停止进程
│   │   └─ get_status()          运行状态查询
│   │
│   ├─ onebot_client            OneBot 11 HTTP API 封装
│   │   ├─ send_like()           POST /send_like
│   │   ├─ get_friend_list()     POST /get_friend_list
│   │   └─ get_login_info()      POST /get_login_info
│   │
│   ├─ like_engine              点赞业务引擎
│   │   ├─ scheduled_likes()     定时全量点赞（随机顺序、间隔控制）
│   │   ├─ reply_like()          回赞处理
│   │   └─ quota_manager         名额管理（总量/预留/已用）
│   │
│   ├─ friend_manager           好友管理
│   │   ├─ groups/tags           分组与标签 CRUD
│   │   └─ strategy              按分组的点赞策略配置
│   │
│   ├─ stats                    数据统计
│   │   ├─ record_like()         记录每次点赞
│   │   └─ query()               聚合查询（日/周/月）
│   │
│   ├─ db                       SQLite 数据层
│   │   ├─ friends               好友信息 + 分组
│   │   ├─ like_history          点赞历史
│   │   ├─ daily_state           每日状态
│   │   └─ config                配置持久化
│   │
│   ├─ scheduler                定时任务（tokio cron）
│   │
│   ├─ tray                     系统托盘管理
│   │   ├─ icon_state            图标状态（绿/黄/红）
│   │   └─ context_menu          右键菜单
│   │
│   ├─ webhook_server           HTTP Server 接收 NapCat 事件推送
│   │
│   └─ config                   应用配置管理
│
└─ React 前端 (Tauri WebView)
    │
    ├─ pages/
    │   ├─ Dashboard             仪表盘（今日状态、快捷操作）
    │   ├─ Friends               好友管理（列表、分组、标签）
    │   ├─ Statistics            数据统计（图表、趋势、排行）
    │   ├─ Logs                  运行日志
    │   └─ Settings              设置（点赞配置、自启动、NapCat）
    │
    ├─ components/               公共组件
    │
    └─ lib/
        └─ tauri-api             Tauri invoke 封装（前后端通信）
```

---

## 5. 数据模型

### 5.1 SQLite 表设计（初步）

```sql
-- 好友信息
CREATE TABLE friends (
    user_id   TEXT PRIMARY KEY,
    nickname  TEXT,
    remark    TEXT,
    group_tag TEXT,           -- 分组标签
    priority  INTEGER DEFAULT 0,
    updated_at DATETIME
);

-- 点赞历史
CREATE TABLE like_history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    TEXT NOT NULL,
    times      INTEGER,        -- 本次点赞次数
    like_type  TEXT,            -- 'scheduled' | 'reply' | 'manual'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 每日状态
CREATE TABLE daily_state (
    date              TEXT PRIMARY KEY,  -- 'YYYY-MM-DD'
    total_liked       INTEGER DEFAULT 0,
    scheduled_count   INTEGER DEFAULT 0,
    reply_count       INTEGER DEFAULT 0,
    manual_count      INTEGER DEFAULT 0
);

-- 应用配置
CREATE TABLE config (
    key   TEXT PRIMARY KEY,
    value TEXT
);
```

---

## 6. 用户体验流程

### 6.1 首次使用
```
双击 qq-auto-like-plus.exe
  → 检测到首次运行
  → 自动下载 NapCat Shell OneKey（显示进度条）
  → 解压到应用数据目录
  → 启动 NapCat → 弹出扫码登录窗口
  → 扫码成功 → 自动检测 QQ 号 → 保存配置
  → 进入管理面板 → 引导设置定时任务参数
  → 最小化到系统托盘 → 开始运行
```

### 6.2 日常使用
```
开机自启（可选）
  → 系统托盘图标（绿色 = 正常运行）
  → 后台自动执行定时点赞
  → 收到赞 → 自动回赞
  → 双击托盘图标 → 打开管理面板查看状态
```

### 6.3 托盘菜单
```
右键托盘图标：
  ├─ 打开面板
  ├─ 立即点赞
  ├─ 暂停/恢复
  ├─ NapCat 状态: 运行中 ✓
  └─ 退出
```

---

## 7. 技术风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| NapCat 更新导致不兼容 | 点赞功能失效 | 锁定版本 + 支持手动指定 NapCat 路径 |
| QQ 账号风控/封禁 | 无法使用 | 文档提醒使用小号、控制点赞频率和间隔 |
| NapCat Shell OneKey 下载链接失效 | 首次安装失败 | 提供手动下载指引 + 支持本地导入 |
| Tauri WebView 兼容性 | Win10 旧版渲染异常 | 使用 WebView2（Win10/11 自带或自动安装） |
| 30 天强制重新登录 | 服务中断 | 检测掉线状态 → 系统通知提醒用户重新扫码 |

---

## 8. 项目范围与里程碑

### V1.0 — 核心重构（目标版本）
- [x] Tauri 2.0 + Rust 后端搭建
- [x] NapCat Shell 进程管理
- [x] OneBot API 客户端
- [x] 定时全量点赞 + 自动回赞
- [x] 系统托盘 + 开机自启
- [x] React 管理面板（仪表盘、日志、设置）
- [x] 好友分组/标签系统
- [x] 数据统计面板
- [x] SQLite 持久化

### V1.x — 后续迭代（待定）
- [ ] 多账号支持
- [ ] 插件系统（类似旧 SendLike 的"赞我"指令响应）
- [ ] 自动更新机制
- [ ] 数据导出（CSV/Excel）
- [ ] 深色模式

---

## 9. 参考资料

- [NapCat 官方文档](https://napneko.github.io/)
- [NapCat Shell 部署教程](https://napneko.github.io/guide/boot/Shell)
- [NapCat GitHub](https://github.com/NapNeko/NapCatQQ)
- [Tauri 2.0 文档](https://v2.tauri.app/)
- [OneBot 11 协议规范](https://github.com/botuniverse/onebot-11)
- 旧项目参考：`PROJECT_SUMMARY.md`
