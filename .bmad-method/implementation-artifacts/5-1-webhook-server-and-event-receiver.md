# Story 5.1: Webhook 服务器与事件接收

Status: Done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 用户,
I want 应用能接收到别人给我点赞的通知,
so that 可以自动赞回去。

## Acceptance Criteria

1. **axum HTTP 服务器**：`webhook/mod.rs` 使用 axum 启动 HTTP 服务器监听 `webhook_port`（默认 8080），与 Tauri 共享同一 Tokio 异步运行时
2. **仅本地监听**：服务器绑定 `127.0.0.1:webhook_port`，不暴露到外部网络
3. **POST /webhook 端点**：接收 NapCat 推送的 POST 请求，Content-Type 为 application/json
4. **事件解析**：解析 OneBot 11 事件 JSON，识别 `post_type=notice` + `notice_type=notify` + `sub_type=profile_like` 事件
5. **提取点赞者**：从 profile_like 事件中提取 `operator_id`（点赞者 QQ 号）
6. **静默忽略**：非 profile_like 事件（消息、请求、其他通知类型）静默忽略，返回 200 OK
7. **事件转发**：收到 profile_like 事件后通过 Tauri event `emit("webhook:profile-like", { userId })` 通知前端和后续回赞处理（Story 5.2 消费）
8. **端口可配置**：webhook 端口通过 config 表的 `webhook_port` 键读取，默认值 8080
9. **端口冲突处理**：端口被占用时记录 `error!` 日志并通过 `emit("webhook:error", { message })` 通知前端
10. **tracing 日志**：所有事件接收、解析、忽略操作记录 tracing 日志（info 级别收到事件、debug 级别忽略事件、error 级别解析失败）
11. **NapCat 配置更新**：`napcat/config.rs` 生成的 OneBot 配置文件中启用 httpServers Webhook 回调（`http://127.0.0.1:{webhook_port}/webhook`）
12. **优雅停止**：Webhook 服务器支持 graceful shutdown，应用退出时正确释放端口

## Tasks / Subtasks

- [x] Task 1: 新增 webhook_port 配置项 (AC: #8)
  - [x] 1.1 创建 migration `005_webhook_config.sql`：`INSERT OR IGNORE INTO config (key, value) VALUES ('webhook_port', '8080');`
  - [x] 1.2 在 `db/migrations.rs` 的 MIGRATIONS 数组添加 `("005_webhook_config", include_str!(...))`

- [x] Task 2: 定义 OneBot Webhook 事件类型 (AC: #3, #4, #5)
  - [x] 2.1 在 `onebot/types.rs` 添加 Webhook 事件相关结构体：
    - `OneBotEvent`：顶层事件（`post_type`, `notice_type`, `sub_type` 等字段，使用 `Option` 兼容不同事件类型）
    - `ProfileLikeEvent`：包含 `operator_id: i64`（点赞者）、`user_id: i64`（被赞者）
  - [x] 2.2 所有请求体接收使用 `serde(default)` 保持向前兼容，防止 NapCat 版本差异导致反序列化失败

- [x] Task 3: 实现 axum Webhook 服务器 (AC: #1, #2, #3, #4, #5, #6, #7, #9, #10, #12)
  - [x] 3.1 在 `webhook/mod.rs` 定义 `WebhookServer` 结构体和 `WebhookState`（包含 `AppHandle`）
  - [x] 3.2 实现 `POST /webhook` handler：
    - 解析 JSON body 为 `OneBotEvent`
    - 检查 `post_type == "notice"` && `notice_type == "notify"` && `sub_type == "profile_like"`
    - 匹配 → 提取 `operator_id`，`emit("webhook:profile-like", payload)`
    - 不匹配 → `tracing::debug!` 记录并返回 200 OK
    - 解析失败 → `tracing::warn!` 记录原始 body 片段，返回 200 OK（不返回错误，防止 NapCat 重试）
  - [x] 3.3 实现 `start()` 函数：
    - 从 config 读取 `webhook_port`
    - `TcpListener::bind(("127.0.0.1", port))`
    - 绑定失败 → `error!` 日志 + `emit("webhook:error", ...)` + 返回 Err
    - 使用 `axum::serve(listener, app).with_graceful_shutdown(shutdown_signal)` 启动
  - [x] 3.4 使用 `tokio::sync::watch` channel 实现 graceful shutdown signal
  - [x] 3.5 导出 `WebhookServerHandle`（包含 shutdown sender + JoinHandle）供 lib.rs 管理生命周期

- [x] Task 4: 更新 NapCat 配置生成器 (AC: #11)
  - [x] 4.1 修改 `napcat/config.rs` 的 `generate_napcat_config()`：从 config 表读取 `webhook_port`
  - [x] 4.2 在生成的 `onebot11.json` 的 `httpServers` 数组中添加 Webhook 回调配置：
    ```json
    {
      "name": "auto-like-webhook",
      "enable": true,
      "url": "http://127.0.0.1:{webhook_port}/webhook",
      "secret": "",
      "reportSelfMessage": false
    }
    ```

- [x] Task 5: 集成 Webhook 服务器到应用生命周期 (AC: #1, #12)
  - [x] 5.1 在 `lib.rs` 的 `setup()` 中，OneBotClient 初始化之后启动 Webhook 服务器
  - [x] 5.2 将 `WebhookServerHandle` 通过 `app.manage()` 注入 Tauri State
  - [x] 5.3 在应用退出流程中（`on_window_event` 或 shutdown hook）触发 graceful shutdown

- [x] Task 6: 添加 Webhook 状态 Tauri command (AC: #9)
  - [x] 6.1 在 `commands/` 下添加 `webhook.rs`（或扩展已有模块）
  - [x] 6.2 添加 `get_webhook_status` command：返回 Webhook 服务器运行状态（running/stopped/error）
  - [x] 6.3 在 `commands/mod.rs` 注册新模块
  - [x] 6.4 在 `lib.rs` 的 `invoke_handler` 中注册新 command

## Dev Notes

### 已有基础设施（直接复用！）

**Rust 依赖（已在 Cargo.toml，无需新增）：**
- `axum = "0.8"` — HTTP 服务器框架
- `tokio = { version = "1", features = ["full"] }` — 异步运行时（含 `net::TcpListener`）
- `serde = { version = "1", features = ["derive"] }` + `serde_json = "1"` — JSON 序列化
- `tracing = "0.1"` — 结构化日志

**已存在的模块（已在 lib.rs 声明）：**
- `mod webhook;` — 已在 `lib.rs` line 9 声明，但 `webhook/mod.rs` 当前为空
- `mod onebot;` — `onebot/types.rs` 已有 `OneBotError`、`SendLikeRequest`、`FriendInfo`、`OneBotLoginInfo`
- `mod engine;` — `engine/quota.rs` 已有名额管理（`try_consume_quota("reply")`、`has_liked_today()`）
- `mod db;` — `db/models.rs` 已有 `get_config_by_key()`、`insert_like_history()`

**Tauri AppHandle 模式（参考 lib.rs 现有代码）：**
```rust
// lib.rs:79 — 获取 app_handle 用于 emit
let app_handle = app.handle().clone();
// lib.rs:101 — 监听事件
app.listen("napcat:status-changed", move |event| { ... });
// 已有事件模式：emit("napcat:status-changed", payload)、emit("engine:status-changed", payload)
```

**OneBotClient State 模式（参考 lib.rs:54-56）：**
```rust
pub type OneBotClientState = Arc<OneBotClient>;
// app.manage(onebot_client.clone());
```

**Config 读取模式（参考 lib.rs:47-53）：**
```rust
let port: u16 = {
    let conn = db_state.lock().expect("lock db");
    db::models::get_config_by_key(&conn, "api_port")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(3000)
};
```

**错误处理模式（参考 errors.rs）：**
- `AppError` 已有 `NapCat(String)` 变体 — Webhook 错误可复用或新增 `Webhook(String)` 变体
- Tauri command 层：`Result<T, String>` + `.map_err(|e| e.to_string())`

### axum 0.8 关键 API（与 0.7 的差异！）

**axum 0.8 Breaking Changes（必须遵守！）：**
- `axum::serve()` 替代 `Server::bind()`，签名为 `axum::serve(listener, app)`
- `TcpListener` 使用 `tokio::net::TcpListener`（不是 `std::net::TcpListener`）
- Router 创建：`Router::new().route("/webhook", post(handler))`
- State 注入使用 `Router::with_state(state)` + handler 参数 `State(state): State<AppState>`
- `with_graceful_shutdown()` 接受一个 `Future<Output = ()>`

**正确的 axum 0.8 启动模式：**
```rust
use axum::{Router, routing::post, extract::State, Json};
use tokio::net::TcpListener;

let app = Router::new()
    .route("/webhook", post(handle_webhook))
    .with_state(webhook_state);

let listener = TcpListener::bind(("127.0.0.1", port)).await?;
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal)
    .await?;
```

### OneBot 11 Webhook 事件格式

NapCat 推送的 profile_like 事件 JSON 格式：
```json
{
  "time": 1710000000,
  "self_id": 123456789,
  "post_type": "notice",
  "notice_type": "notify",
  "sub_type": "profile_like",
  "user_id": 123456789,
  "operator_id": 987654321
}
```

**关键字段说明：**
- `post_type`: 事件大类 — `"message"` | `"notice"` | `"request"` | `"meta_event"`
- `notice_type`: 通知子类 — `"notify"` 表示提示类通知
- `sub_type`: 具体类型 — `"profile_like"` 表示主页点赞
- `operator_id`: 点赞操作者的 QQ 号（我们需要回赞的目标）
- `user_id`: 被点赞者（通常是自己）

**反序列化策略：**
- 使用宽松反序列化：所有非必须字段标 `#[serde(default)]`
- NapCat 不同版本可能有额外字段 — 使用 `#[serde(deny_unknown_fields)]` 会导致解析失败，**禁止使用**
- profile_like 事件中 `operator_id` 有时也通过 `sender_id` 字段传递 — 用 `#[serde(alias = "sender_id")]` 做兼容

### NapCat httpServers 配置格式

当前 `napcat/config.rs:19-47` 生成的 onebot11.json 中 `"httpServers": []` 为空数组。需要改为：
```json
"httpServers": [
  {
    "name": "auto-like-webhook",
    "enable": true,
    "url": "http://127.0.0.1:8080/webhook",
    "secret": "",
    "reportSelfMessage": false
  }
]
```

### Webhook 事件的 Tauri Emit Payload 类型

为 Story 5.2 准备的事件格式（Story 5.1 只负责 emit，5.2 负责 listen 和处理）：
```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileLikePayload {
    pub operator_id: i64,  // 点赞者 QQ 号
    pub timestamp: i64,    // 事件时间戳
}
// emit("webhook:profile-like", ProfileLikePayload { ... })
```

前端 TypeScript 对应类型（Story 5.2 前端消费时添加）：
```typescript
interface ProfileLikePayload {
  operatorId: number;
  timestamp: number;
}
```

### Graceful Shutdown 模式

使用 `tokio::sync::watch` channel：
```rust
let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

// 启动时传入 shutdown signal
let shutdown_signal = async move {
    shutdown_rx.changed().await.ok();
};

// 停止时发送信号
shutdown_tx.send(true).unwrap();
```

导出 Handle 让 lib.rs 可以在退出时触发：
```rust
pub struct WebhookServerHandle {
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    join_handle: tokio::task::JoinHandle<()>,
}
```

### 端口冲突处理

`TcpListener::bind()` 失败时（`AddrInUse`）：
1. `tracing::error!("Webhook 端口 {} 被占用", port)`
2. `app_handle.emit("webhook:error", json!({ "message": "端口被占用..." }))`
3. 不 panic — 应用继续运行，但 Webhook 功能不可用
4. get_webhook_status 命令返回 error 状态

### 前几个 Story 的经验教训（必须遵守！）

1. **serde rename_all = "camelCase"**：所有对外暴露的结构体必须加此注解（Architecture 强制规则 #1）
2. **Tauri command 返回 Result<T, String>**：不直接暴露 AppError（Architecture 强制规则 #2）
3. **事件命名 namespace:action**：webhook 域使用 `webhook:profile-like`、`webhook:error`（Architecture 事件命名规范）
4. **不用 println!/unwrap()/expect()**：使用 tracing 宏 + `?` 操作符（Architecture 反模式清单）
5. **webhook/ 收到事件后只通过 emit 转发**：不直接操作数据库或 OneBot（Architecture 组件边界规则）
6. **config 读取使用 db::models::get_config_by_key()**：不绕过 db 模块（Architecture 边界规则）
7. **Tauri invoke 一致性**：新增的 Tauri command 如果需要前端调用，wrapper 统一放 `src/lib/tauri.ts`（Story 4.2 P3-F1 教训）
8. **shadcn base-ui Tooltip 不支持 asChild**：如果涉及前端 tooltip，使用 `render` prop（Story 4.2 教训）

### 不要做的事情

- **不要实现回赞逻辑** — 那是 Story 5.2 的 `engine/reply_handler.rs` 职责
- **不要在 webhook handler 中直接调用 OneBotClient** — 只 emit 事件，让 5.2 的 listener 处理
- **不要在 webhook handler 中直接操作数据库** — 遵守 architecture boundary
- **不要用 `std::net::TcpListener`** — axum 0.8 使用 `tokio::net::TcpListener`
- **不要给非 profile_like 事件返回非 200 状态码** — NapCat 可能会重试
- **不要引入新的 Rust crate** — axum + tokio + serde 已经足够
- **不要修改任何前端文件** — Story 5.1 是纯后端
- **不要修改 `engine/mod.rs` 或创建 `engine/reply_handler.rs`** — Story 5.2 范围
- **不要修改已有的 Tauri event listeners（napcat:status-changed、engine:status-changed）**
- **不要修改 `tray/mod.rs`**
- **不要修改 `onebot/client.rs`** — 只修改 `onebot/types.rs` 添加事件类型

### Project Structure Notes

新增文件：
```
src-tauri/
├── migrations/
│   └── 005_webhook_config.sql        # NEW — webhook_port 配置默认值
└── src/
    ├── webhook/
    │   └── mod.rs                     # MODIFY（当前为空）— axum Webhook 服务器完整实现
    ├── onebot/
    │   └── types.rs                   # MODIFY — 添加 OneBot Webhook 事件类型
    ├── commands/
    │   ├── mod.rs                     # MODIFY — 注册 webhook 模块
    │   └── webhook.rs                 # NEW — get_webhook_status command
    └── napcat/
        └── config.rs                  # MODIFY — 添加 httpServers webhook 配置
```

修改文件：
```
src-tauri/src/lib.rs                   # MODIFY — 启动 Webhook 服务器 + 注册 command + state
src-tauri/src/db/migrations.rs         # MODIFY — 添加 005 migration
src-tauri/src/errors.rs                # MODIFY — 添加 Webhook 错误变体（可选）
```

**路径与架构对齐验证：**
- `webhook/mod.rs` — 与 architecture.md 项目结构定义一致 ✅
- `commands/webhook.rs` — 遵循 commands/ 按功能域分文件模式 ✅
- OneBot 事件类型放 `onebot/types.rs` — 遵循类型集中定义模式 ✅
- 新 migration 编号 005 — 顺承现有 001-004 ✅

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 5.1: Webhook 服务器与事件接收]
- [Source: .bmad-method/planning-artifacts/epics.md#Epic 5: 自动回赞 — FR12, FR13, FR14, FR15, AR4]
- [Source: .bmad-method/planning-artifacts/architecture.md#API 与通信 — Webhook 服务器: axum]
- [Source: .bmad-method/planning-artifacts/architecture.md#项目结构 — webhook/mod.rs]
- [Source: .bmad-method/planning-artifacts/architecture.md#架构边界 — NapCat → Rust via axum HTTP POST]
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则 — webhook/ 收到事件后只调用 engine/]
- [Source: .bmad-method/planning-artifacts/architecture.md#数据流 — NapCat 推送 → webhook/ → engine/]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — Rust snake_case, serde camelCase]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri 事件命名 — namespace:action 格式]
- [Source: .bmad-method/planning-artifacts/architecture.md#错误处理模式 — thiserror 库层 + anyhow 应用层]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单]
- [Source: .bmad-method/planning-artifacts/architecture.md#强制规则 7 条]
- [Source: src-tauri/Cargo.toml — axum 0.8, tokio full 已安装]
- [Source: src-tauri/src/lib.rs — mod webhook 已声明, setup() 模式参考]
- [Source: src-tauri/src/webhook/mod.rs — 当前为空]
- [Source: src-tauri/src/onebot/types.rs — 现有类型定义模式]
- [Source: src-tauri/src/onebot/client.rs — OneBotClient 实现参考]
- [Source: src-tauri/src/napcat/config.rs — generate_napcat_config() 需修改]
- [Source: src-tauri/src/engine/quota.rs — try_consume_quota("reply") 回赞名额消耗（Story 5.2 用）]
- [Source: src-tauri/src/db/models.rs — get_config_by_key(), has_liked_today() 已存在]
- [Source: src-tauri/src/db/migrations.rs — migration 注册模式]
- [Source: src-tauri/src/errors.rs — AppError 枚举定义]
- [Source: src-tauri/src/commands/mod.rs — 模块注册模式]
- [Source: .bmad-method/implementation-artifacts/4-4-log-viewer-page.md — 前置 Story 经验教训]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

无调试问题。唯一编译错误为 `state` 生命周期问题（lib.rs Exit handler），通过在 `if let Ok(guard)` 块后添加 `;` 解决。

### Completion Notes List

- 使用 `axum::body::Bytes` 接收 raw body 后再反序列化，便于在解析失败时记录原始 body
- `OneBotEvent` 所有字段使用 `#[serde(default)]`，`operator_id` 添加 `#[serde(alias = "sender_id")]` 兼容
- Graceful shutdown 通过 `Builder::build()` + `App::run()` 模式在 `RunEvent::Exit` 中触发
- 端口冲突时不 panic，返回 `None` 使应用继续运行但 Webhook 不可用
- `get_webhook_status` command 返回 "running"/"stopped"/"error" 三种状态

### File List

- `src-tauri/migrations/005_webhook_config.sql` — NEW — webhook_port 配置默认值
- `src-tauri/src/db/migrations.rs` — MODIFIED — 添加 005 migration
- `src-tauri/src/onebot/types.rs` — MODIFIED — 添加 OneBotEvent, ProfileLikePayload 类型
- `src-tauri/src/webhook/mod.rs` — MODIFIED（原为空）— axum Webhook 服务器完整实现
- `src-tauri/src/napcat/config.rs` — MODIFIED — httpServers 添加 webhook 回调配置
- `src-tauri/src/lib.rs` — MODIFIED — Webhook 服务器启动/State 注册/graceful shutdown/command 注册
- `src-tauri/src/commands/webhook.rs` — NEW — get_webhook_status command
- `src-tauri/src/commands/mod.rs` — MODIFIED — 注册 webhook 模块

### Change Log

- 2026-03-14: Story 5.1 实现完成，所有 6 个 Task 已完成，cargo check 通过

## QA Results

### Review Type: Code Review
### Reviewer: Quinn (Test Architect)
### Date: 2026-03-14
### Gate Decision: PASS

---

### AC 验证矩阵 (12/12 PASS)

| AC# | 验证项 | 状态 | 验证位置 |
|-----|--------|------|----------|
| 1 | axum HTTP 服务器 + Tokio 共享运行时 | PASS | `webhook/mod.rs:57-65` — `tokio::spawn` 在 Tauri Tokio runtime 内 |
| 2 | 仅本地监听 127.0.0.1 | PASS | `webhook/mod.rs:37` — `TcpListener::bind(("127.0.0.1", port))` |
| 3 | POST /webhook 端点 | PASS | `webhook/mod.rs:34` — `route("/webhook", post(handle_webhook))` |
| 4 | OneBot 11 事件 JSON 解析 | PASS | `webhook/mod.rs:79` — `serde_json::from_slice::<OneBotEvent>` |
| 5 | 提取 operator_id | PASS | `webhook/mod.rs:100-103` — `event.operator_id` + alias `sender_id` |
| 6 | 非 profile_like 静默忽略 + 200 OK | PASS | `webhook/mod.rs:108-117` — debug! 日志 + 返回 `{ "status": "ok" }` |
| 7 | Tauri emit webhook:profile-like | PASS | `webhook/mod.rs:105` — `emit("webhook:profile-like", &payload)` |
| 8 | webhook_port 可配置 + 默认 8080 | PASS | `lib.rs:98-104` + `005_webhook_config.sql` |
| 9 | 端口冲突 error! 日志 + emit | PASS | `webhook/mod.rs:40-46` — error! + `emit("webhook:error", ...)` |
| 10 | tracing 日志（info/debug/error 分级） | PASS | info:55,96 / debug:109 / warn:82 / error:62,106 |
| 11 | NapCat httpServers 配置更新 | PASS | `napcat/config.rs:36-44` — 动态 webhook_port URL |
| 12 | Graceful shutdown | PASS | `webhook/mod.rs:49-53` — watch channel + `lib.rs:252-261` RunEvent::Exit 触发 |

---

### 架构合规检查 (7/7 PASS)

| 规则# | 强制规则 | 状态 | 备注 |
|--------|---------|------|------|
| 1 | 对外结构体 `rename_all = "camelCase"` | PASS | `ProfileLikePayload` ✅ / `OneBotEvent` 为内部反序列化无需 ✅ |
| 2 | Tauri command 返回 `Result<T, String>` | PASS | `get_webhook_status` ✅ |
| 3 | 事件命名 `namespace:action` | PASS | `webhook:profile-like`, `webhook:error` ✅ |
| 4 | 数据库命名规范 | PASS | `config` 表 snake_case ✅ |
| 5 | — (前端无修改) | N/A | |
| 6 | — (无 store 变更) | N/A | |
| 7 | 使用 tracing 宏，禁止 println! | PASS | 全文零 println!/unwrap()/expect() ✅ |

### 反模式检查: PASS
- 无 `println!` / `unwrap()` / `expect()` / `static mut`
- webhook/ 未直接操作数据库或 OneBot（组件边界合规）
- 未引入新 crate

---

### 发现项

#### ADVISORY — 字节边界截断 (Low Severity)

- **位置**: `webhook/mod.rs:85`
- **代码**: `&body_str[..body_str.len().min(200)]`
- **风险**: 若 body 包含多字节 UTF-8 字符且恰好在第 200 字节处截断，会触发 `str` 索引 panic
- **概率**: 极低（NapCat JSON 主体为 ASCII），但非零
- **建议**: 可在后续迭代中改为 `body_str.get(..200).unwrap_or(&body_str)` 或 `&body[..body.len().min(200)]`（直接截原始字节记录）
- **不阻塞**: 此为 advisory，不影响 PASS 判定

#### INFO — 两次 mutex lock (napcat/config.rs)

- **位置**: `napcat/config.rs:12-25`
- **描述**: `napcat_api_port` 和 `webhook_port` 分别获取 mutex lock，可合并为一次
- **影响**: 无功能影响，仅微小性能差异
- **不阻塞**: 信息项

---

### 总评

实现质量优秀。8 个文件全部符合架构规范，12 项 AC 全部满足，代码简洁清晰。Webhook 模块正确遵守组件边界（只 emit 不操作 DB/OneBot），graceful shutdown 机制完整，错误处理路径合理（端口冲突不 panic，应用继续运行）。唯一 advisory 项为日志截断的极低概率 panic 风险，不影响通过。
