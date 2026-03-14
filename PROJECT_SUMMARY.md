# QQ 自动点赞项目总结

> 供新项目 agent 分析用。本文档覆盖两个仓库：`SendLike`（LangBot 插件）和 `qq-auto-like`（独立脚本）。

---

## 一、两个项目的关系

| | SendLike | qq-auto-like |
|---|----------|-------------|
| 定位 | LangBot 插件（被动触发） | 独立 Windows 脚本（主动+被动） |
| 触发方式 | 用户发送"赞我"触发 | 定时全量点赞 + webhook 回赞 + Web 面板手动触发 |
| OneBot 管理 | 不管理，依赖外部 NapCat | 内置 NapCatManager 管理 NapCat 生命周期 |
| 部署形态 | 放入 LangBot 插件目录 | 独立运行，含 bat 脚本和 Web 面板 |
| 核心 API | `POST /send_like` | `POST /send_like` + `POST /get_friend_list` |

**共同点**：都通过 OneBot 11 HTTP API 的 `/send_like` 端点实现 QQ 主页点赞。

---

## 二、SendLike（LangBot 插件）

### 文件结构
```
SendLike/
├── main.py          # 唯一代码文件
├── config.yaml      # onebot_api_url
└── manifest.yaml    # LangBot 插件清单
```

### 核心逻辑
- 继承 `BasePlugin`，通过 `@register` 注册插件元数据
- `@handler(PersonMessageReceived)` + `@handler(GroupMessageReceived)` 监听消息
- 解析"赞我"或"赞我 N"→ 调用 `/send_like` → 回复结果
- 点赞次数范围 1-20，默认 10，超出按 20 处理
- 使用 `ctx.send_message()` 异步回复，`ctx.prevent_default()` 阻止默认行为

### 关键依赖
- LangBot 插件框架（`pkg.plugin.*`, `pkg.platform.types`）
- aiohttp、PyYAML

---

## 三、qq-auto-like（独立脚本）

### 文件结构
```
qq-auto-like/
├── main.py                  # 入口，编排所有组件
├── config.yaml              # 全局配置
├── requirements.txt         # aiohttp, APScheduler, PyYAML
├── core/
│   ├── config.py            # load_config()，读取 config.yaml
│   ├── onebot.py            # OneBotClient：封装 OneBot API 调用
│   ├── like_engine.py       # LikeEngine：定时点赞 + 回赞逻辑
│   ├── state.py             # State：今日状态管理（已赞列表、计数、日志）
│   ├── scheduler.py         # APScheduler 定时任务
│   ├── napcat.py            # NapCatManager：NapCat 进程生命周期管理
│   └── lagrange.py          # LagrangeManager（已废弃，签名服务器关停）
├── web/
│   ├── server.py            # create_app()，aiohttp Web 应用
│   ├── handlers.py          # API 路由 + webhook 处理
│   └── templates.py         # 单文件 HTML 前端（内联 CSS/JS）
├── start.bat                # 启动脚本（含协议选择菜单）
├── stop.bat                 # 停止脚本
├── setup.bat                # 环境安装脚本
├── deploy_docker.bat        # Docker 一键部署脚本
└── run_background.vbs       # 后台静默启动
```

### 架构概览

```
start.bat → python main.py
                │
                ├─ NapCatManager    管理 NapCat 进程（Docker/Local）
                │   ├─ install()          安装/拉取镜像
                │   ├─ generate_config()  生成 onebot11 配置
                │   ├─ start()            启动进程/容器
                │   ├─ wait_ready()       轮询 /get_login_info
                │   └─ stop()             停止进程/容器
                │
                ├─ OneBotClient     调用 OneBot HTTP API
                │   ├─ send_like()        点赞
                │   └─ get_friend_list()  获取好友列表
                │
                ├─ LikeEngine       点赞业务逻辑
                │   ├─ do_scheduled_likes()  全量定时点赞
                │   └─ do_reply_like()       回赞
                │
                ├─ State            今日状态（内存 + JSON 持久化）
                │   ├─ liked_friends     已赞好友 {user_id: times}
                │   ├─ can_like()        检查名额（区分定时/回赞）
                │   └─ save()/load()     data/state.json
                │
                ├─ Scheduler        APScheduler 定时任务
                │   ├─ 每日定时点赞     CronTrigger(hour, minute)
                │   └─ 定期保存状态     interval 5 分钟
                │
                └─ Web Server       aiohttp Web 面板 (:8080)
                    ├─ GET  /              单页面板（状态/设置）
                    ├─ GET  /api/status    今日统计
                    ├─ GET  /api/logs      运行日志
                    ├─ POST /api/trigger   手动触发全量点赞
                    ├─ POST /api/reset     重置今日状态
                    ├─ GET  /api/settings  设置（自启、模式）
                    ├─ POST /api/settings  更新设置
                    └─ POST /webhook       接收 OneBot 事件推送（回赞）
```

### 配置文件 (config.yaml)

```yaml
qq_account: ""                          # QQ 号（空则扫码，登录后自动回填）
onebot_api_url: "http://127.0.0.1:3000"
napcat:
  enabled: true
  mode: "docker"       # "docker" | "local" | "lagrange"(已废弃)
  dir: "napcat"
  container_name: "napcat-autolike"
  docker_image: "mlikiowa/napcat-docker:latest"
  http_port: 3000
  webui_port: 6099
lagrange:              # 已废弃（签名服务器关停）
  dir: "lagrange"
  protocol: "linux"
  sign_server: "https://sign.lagrangecore.org/api/sign"
web:
  host: "0.0.0.0"
  port: 8080
like:
  daily_limit: 50          # 每日总点赞人数上限
  times_per_friend: 10     # 每人点赞次数
  schedule_hour: 0         # 定时任务小时
  schedule_minute: 5       # 定时任务分钟
  batch_interval: 3        # 批量点赞间隔（秒）
  reserved_for_reply: 10   # 为回赞预留的名额
log:
  level: "INFO"
  file: "logs/auto_like.log"
```

### 核心模块详解

#### OneBotClient (`core/onebot.py`)
- 封装 aiohttp.ClientSession，复用连接
- `_call(endpoint, data)` 通用 POST 调用
- `send_like(user_id, times)` → `POST /send_like`
- `get_friend_list()` → `POST /get_friend_list`

#### LikeEngine (`core/like_engine.py`)
- **定时全量点赞** `do_scheduled_likes()`：获取好友列表 → 随机打乱 → 逐个点赞（跳过已赞、检查名额） → 每人间隔 3 秒
- **回赞** `do_reply_like(user_id)`：收到 webhook 点赞通知后触发，检查是否已赞/名额是否满

#### State (`core/state.py`)
- 跨日自动重置（`_reset_if_new_day`）
- 区分定时点赞和回赞计数
- 名额管理：`daily_limit` 总量，`reserved_for_reply` 为回赞预留
- 持久化到 `data/state.json`

#### NapCatManager (`core/napcat.py`)
- **Docker 模式**：`docker run/stop/rm`，挂载 config 和 QQ 数据目录，转发日志
- **Local 模式**：运行 `NapCatWinBootMain.exe`，`taskkill /T /F` 停止
- 自动生成 onebot11 配置（HTTP Server + HTTP Client webhook）
- 扫码登录后自动检测 QQ 号、回写 config.yaml、重启生效
- `wait_ready()` 轮询 `/get_login_info` 等待 API 就绪

#### Web 面板 (`web/`)
- 单文件 HTML，内联 CSS/JS，无构建工具
- 两个 tab：状态（统计+好友列表+日志）、设置（自启开关+模式显示）
- 轮询刷新：状态 5 秒、日志 3 秒
- webhook 端点接收 OneBot 的 `profile_like` 通知触发回赞

### Bat 脚本

| 脚本 | 功能 |
|------|------|
| `setup.bat` | 安装 Python 依赖 + 下载 NapCat OneKey |
| `start.bat` | 协议选择菜单（Docker/Local/Lagrange）→ 写入 config → 前置检查 → 启动 main.py |
| `stop.bat` | 终止 Python + NapCat + Lagrange + Docker 容器 |
| `deploy_docker.bat` | Docker 一键部署（检查环境 → 拉镜像 → 配置 → 启动） |
| `run_background.vbs` | 用 pythonw.exe 后台静默运行 |

---

## 四、已知问题与经验教训

### Lagrange.OneBot 方案（已废弃）
- Lagrange.Core 于 2025-10 归档，签名服务器 `sign.lagrangecore.org` 已关停
- 下载 URL 需要用 nightly tag，文件名含 `_net9.0_SelfContained`
- .NET 程序不能用 `stdout=PIPE` 启动（`Console.BufferWidth` 需要真实控制台句柄，否则崩溃）
- 需要 `CREATE_NEW_CONSOLE` flag 分配独立窗口
- zip 解压后文件在深层嵌套目录 `Lagrange.OneBot/bin/Release/net9.0/win-x64/publish/`，需要自动提升

### Bat 脚本编码陷阱
- **必须用 CRLF 换行**：LF 会导致 CMD 解析错乱，尤其是 if/else 块
- **必须纯 ASCII**：中文（UTF-8）在 CMD 的 GBK 解析下会被截断，某些字节恰好是 `)` 等控制字符
- **避免 if/else 嵌套含特殊字符**：改用 goto 跳转更安全
- PowerShell 5.x 的 `Set-Content -Encoding UTF8` 会写 BOM，应用 `[System.IO.File]::WriteAllText` + `UTF8Encoding($false)`

### NapCat Docker 模式
- webhook 地址需用 `host.docker.internal` 访问宿主机
- 扫码登录后需要重启容器使 onebot11 配置生效
- 首次无 QQ 号时通过日志正则 `napcat_(\d+)\.json` 检测

---

## 五、新项目重构建议

1. **合并两个项目**：SendLike 的"赞我"指令 + qq-auto-like 的定时/回赞/Web 面板
2. **只保留 NapCat Docker 模式**：最稳定、不冲突、社区活跃；去掉 Local 和 Lagrange
3. **前端改用框架**：当前单文件 HTML 已 234 行，加功能会越来越难维护
4. **配置热更新**：当前改配置需重启，可加 Web API 修改 config.yaml
5. **Windows 服务化**：用 NSSM 或 Windows Service 替代 VBS + pythonw 的后台方案
6. **跨平台**：当前强依赖 Windows（taskkill、bat、vbs），可考虑 Docker Compose 一键部署
