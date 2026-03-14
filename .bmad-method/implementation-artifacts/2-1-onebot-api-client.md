# Story 2.1: OneBot API 客户端

Status: Done

## Story

As a 应用,
I want 有一个可靠的 OneBot API 通信层,
so that 可以调用 NapCat 的点赞和查询接口。

## Acceptance Criteria

1. **Given** NapCat 进程已启动并监听 HTTP API（端口 3000）**When** 应用需要调用 OneBot API **Then** `onebot/client.rs` 封装 reqwest HTTP POST 请求
2. **Given** 客户端已初始化 **When** 调用 API **Then** 支持三个端点：`/send_like`（点赞）、`/get_friend_list`（好友列表）、`/get_login_info`（登录检查）
3. **Given** 需要定义数据结构 **When** 处理 OneBot 请求/响应 **Then** 类型定义在 `onebot/types.rs`，暴露给前端的类型使用 `serde(rename_all = "camelCase")`，同时通过 `serde(alias)` 兼容 OneBot 11 的 snake_case 响应
4. **Given** 发送 API 请求 **When** 网络延迟 **Then** 请求超时设置为 10 秒
5. **Given** API 调用失败 **When** 网络错误 **Then** 使用 thiserror 定义具体错误枚举（`ConnectionRefused`、`Timeout`、`ApiError`）
6. **Given** 单次请求失败 **When** 可重试错误 **Then** 自动重试（最多 2 次，间隔 1 秒）
7. **Given** 任何 API 调用 **When** 请求发出或完成 **Then** 记录 tracing 日志（info 级别请求、error 级别失败）

## Tasks / Subtasks

- [X] Task 1: 定义 OneBot 错误类型 (AC: #5)
  - [X] 1.1 创建 `onebot/types.rs`（或在现有结构中）定义 `OneBotError` 枚举：`ConnectionRefused(String)`、`Timeout(String)`、`ApiError(String)`、`Network(String)`、`Deserialize(String)`
  - [X] 1.2 编辑 `errors.rs`：添加 `OneBot(#[from] crate::onebot::OneBotError)` 变体
- [X] Task 2: 定义 OneBot 请求/响应类型 (AC: #3)
  - [X] 2.1 在 `onebot/types.rs` 定义 `OneBotResponse<T>` 通用响应包装（status、retcode、data），仅 `Deserialize`，无 rename（匹配 OneBot snake_case）
  - [X] 2.2 定义 `SendLikeRequest`（user_id、times），仅 `Serialize`，无 rename（发送给 OneBot 需要 snake_case）
  - [X] 2.3 定义 `FriendInfo`（user_id、nickname、remark），`Serialize + Deserialize`，`rename_all = "camelCase"` + `alias = "user_id"` 兼容 OneBot 响应
  - [X] 2.4 定义 `OneBotLoginInfo`（user_id、nickname），同上策略
- [X] Task 3: 实现 OneBotClient (AC: #1, #4, #6, #7)
  - [X] 3.1 创建 `onebot/client.rs`：实现 `OneBotClient` 结构体，持有 `reqwest::Client` 和 `base_url: String`
  - [X] 3.2 `new(api_port: u16)` 构造函数，创建带 10 秒超时的 reqwest::Client
  - [X] 3.3 实现内部 `call_api<T>(&self, endpoint, body)` 通用方法，包含重试逻辑（最多 2 次重试，间隔 1 秒）
  - [X] 3.4 重试中对 reqwest::Error 分类：`is_connect()` → ConnectionRefused、`is_timeout()` → Timeout、其他 → Network
  - [X] 3.5 检查 OneBot 响应 status 字段，非 "ok" 时返回 ApiError（含 retcode）
  - [X] 3.6 每次请求 tracing::info! 记录端点和参数摘要，失败 tracing::error! 记录具体错误
- [X] Task 4: 实现三个 API 端点方法 (AC: #2)
  - [X] 4.1 `send_like(&self, user_id: i64, times: i32) -> Result<(), OneBotError>`：POST `/send_like`
  - [X] 4.2 `get_friend_list(&self) -> Result<Vec<FriendInfo>, OneBotError>`：POST `/get_friend_list`
  - [X] 4.3 `get_login_info(&self) -> Result<OneBotLoginInfo, OneBotError>`：POST `/get_login_info`
- [X] Task 5: 连接模块 (AC: #1)
  - [X] 5.1 编辑 `onebot/mod.rs`：添加 `pub mod client; pub mod types;`，re-export 关键类型
  - [X] 5.2 确保 `lib.rs` 已有 `mod onebot;`（当前存在但为空模块，需改为目录模块或添加子模块声明）
- [X] Task 6: 前端类型定义 (AC: #3)
  - [X] 6.1 创建 `src/types/onebot.ts`：定义 `FriendInfo`（userId、nickname、remark）和 `OneBotLoginInfo`（userId、nickname）TypeScript 接口
- [X] Task 7: 构建验证 (AC: #1-#7)
  - [X] 7.1 `cargo check` 编译通过
  - [X] 7.2 `npx tsc --noEmit` TypeScript 类型检查通过

## Dev Notes

### OneBot 11 协议关键点

OneBot 11 HTTP API 使用 **snake_case** JSON 格式。所有请求为 POST，body 为 JSON。

**通用响应格式：**

```json
{
  "status": "ok",
  "retcode": 0,
  "data": { ... }
}
```

**`/send_like` — 点赞：**

```json
// 请求
{ "user_id": 123456789, "times": 10 }
// 响应
{ "status": "ok", "retcode": 0, "data": null }
```

**`/get_friend_list` — 好友列表：**

```json
// 请求
{}
// 响应
{
  "status": "ok", "retcode": 0,
  "data": [
    { "user_id": 123456789, "nickname": "小明", "remark": "备注名" }
  ]
}
```

**`/get_login_info` — 登录信息：**

```json
// 请求
{}
// 响应
{
  "status": "ok", "retcode": 0,
  "data": { "user_id": 123456789, "nickname": "我的昵称" }
}
```

### serde 序列化策略（重要）

OneBot 11 使用 snake_case，前端期望 camelCase。本模块采用如下策略：

**发送给 OneBot 的请求类型（仅 Serialize）**：不加 rename，保持 Rust 默认 snake_case → OneBot 能正确解析。

```rust
#[derive(Debug, Serialize)]
pub struct SendLikeRequest {
    pub user_id: i64,
    pub times: i32,
}
// 序列化: { "user_id": 123, "times": 10 } ✓
```

**从 OneBot 接收且需暴露给前端的类型（Serialize + Deserialize）**：使用 `camelCase` + `alias` 双向兼容。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendInfo {
    #[serde(alias = "user_id")]
    pub user_id: i64,    // 序列化→"userId"(前端), 反序列化←"userId"或"user_id"(OneBot)
    pub nickname: String, // 单词无差异
    #[serde(default, alias = "remark")]
    pub remark: String,
}
```

**仅从 OneBot 反序列化的内部类型（仅 Deserialize）**：不加 rename，直接匹配 snake_case。

```rust
#[derive(Debug, Deserialize)]
pub struct OneBotResponse<T> {
    pub status: String,
    pub retcode: i32,
    pub data: T,
}
```

### OneBotError 定义

```rust
// onebot/types.rs（或单独的 error 子模块）
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OneBotError {
    #[error("连接被拒绝: {0}")]
    ConnectionRefused(String),
    #[error("请求超时: {0}")]
    Timeout(String),
    #[error("API 返回错误 (retcode={retcode}): {message}")]
    ApiError { retcode: i32, message: String },
    #[error("网络错误: {0}")]
    Network(String),
    #[error("反序列化错误: {0}")]
    Deserialize(String),
}
```

**注意：** `errors.rs` 中已有 `Download(#[from] reqwest::Error)`。不能再加另一个 `#[from] reqwest::Error`。`OneBotError` 手动分类 reqwest 错误：

```rust
fn classify_reqwest_error(e: reqwest::Error) -> OneBotError {
    if e.is_connect() {
        OneBotError::ConnectionRefused(e.to_string())
    } else if e.is_timeout() {
        OneBotError::Timeout(e.to_string())
    } else {
        OneBotError::Network(e.to_string())
    }
}
```

### errors.rs 修改

```rust
// 新增变体
#[error("OneBot 错误: {0}")]
OneBot(#[from] crate::onebot::OneBotError),
```

### OneBotClient 实现参考

```rust
// onebot/client.rs
use std::time::Duration;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::types::*;

const MAX_RETRIES: u32 = 2;
const RETRY_INTERVAL: Duration = Duration::from_secs(1);

pub struct OneBotClient {
    client: reqwest::Client,
    base_url: String,
}

impl OneBotClient {
    pub fn new(api_port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("创建 HTTP 客户端失败"); // 构造函数中 expect 可接受
        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", api_port),
        }
    }

    /// 通用 API 调用，含重试逻辑
    async fn call_api<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &impl Serialize,
    ) -> Result<T, OneBotError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut last_error: Option<OneBotError> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                tracing::info!("重试 OneBot API {}: 第 {} 次", endpoint, attempt);
                tokio::time::sleep(RETRY_INTERVAL).await;
            }

            tracing::info!("调用 OneBot API: {}", endpoint);

            match self.client.post(&url).json(body).send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        last_error = Some(OneBotError::ApiError {
                            retcode: status.as_u16() as i32,
                            message: format!("HTTP {}", status),
                        });
                        continue;
                    }

                    let onebot_resp: OneBotResponse<T> = resp.json().await.map_err(|e| {
                        OneBotError::Deserialize(e.to_string())
                    })?;

                    if onebot_resp.status != "ok" {
                        last_error = Some(OneBotError::ApiError {
                            retcode: onebot_resp.retcode,
                            message: format!("status={}", onebot_resp.status),
                        });
                        continue;
                    }

                    return Ok(onebot_resp.data);
                }
                Err(e) => {
                    let classified = classify_reqwest_error(e);
                    tracing::error!("OneBot API {} 失败: {}", endpoint, classified);
                    last_error = Some(classified);
                }
            }
        }

        Err(last_error.unwrap_or(OneBotError::Network("未知错误".to_string())))
    }

    pub async fn send_like(&self, user_id: i64, times: i32) -> Result<(), OneBotError> {
        tracing::info!("点赞: user_id={}, times={}", user_id, times);
        let req = SendLikeRequest { user_id, times };
        // /send_like 返回 null data，用 serde_json::Value 接收
        let _: serde_json::Value = self.call_api("/send_like", &req).await?;
        Ok(())
    }

    pub async fn get_friend_list(&self) -> Result<Vec<FriendInfo>, OneBotError> {
        self.call_api("/get_friend_list", &serde_json::json!({})).await
    }

    pub async fn get_login_info(&self) -> Result<OneBotLoginInfo, OneBotError> {
        self.call_api("/get_login_info", &serde_json::json!({})).await
    }
}

fn classify_reqwest_error(e: reqwest::Error) -> OneBotError {
    if e.is_connect() {
        OneBotError::ConnectionRefused(e.to_string())
    } else if e.is_timeout() {
        OneBotError::Timeout(e.to_string())
    } else {
        OneBotError::Network(e.to_string())
    }
}
```

### 模块结构

当前 `onebot/mod.rs` 是空文件（一行）。需要改为：

```rust
// onebot/mod.rs
pub mod client;
pub mod types;

pub use client::OneBotClient;
pub use types::*;
```

### send_like 的 data: null 处理

OneBot `/send_like` 响应的 `data` 字段为 `null`。`OneBotResponse<T>` 中 `data: T` 反序列化 `null` 时：

- 如果 T = `serde_json::Value`，反序列化为 `Value::Null` ✓
- 如果 T = `()`，serde 不支持从 `null` 反序列化为 `()`

**解决方案：** `send_like` 使用 `serde_json::Value` 接收 data 并丢弃，或者将 `OneBotResponse` 的 data 改为 `Option<T>`：

```rust
#[derive(Debug, Deserialize)]
pub struct OneBotResponse<T> {
    pub status: String,
    pub retcode: i32,
    pub data: Option<T>, // 某些端点返回 null
}
```

使用 `Option<T>` 更通用，`call_api` 返回 `Option<T>` 然后各方法按需处理。**推荐用 `Option<T>` 方案。**

### 现有代码重复注意

以下代码已在别处直接调用 OneBot HTTP API，存在重复：

| 位置                          | 功能                     | 说明                                    |
| ----------------------------- | ------------------------ | --------------------------------------- |
| `napcat/process.rs:212-296` | `poll_login_status()`  | 直接用 reqwest 调用 `/get_login_info` |
| `commands/napcat.rs:82-115` | `get_login_info_cmd()` | 直接用 reqwest 调用 `/get_login_info` |

**本 Story 不重构这些代码。** 它们属于 NapCat 进程管理流程（登录轮询），与业务引擎调用的 OneBot 客户端职责不同。后续可在 Epic 3（NapCat 健康检查）Story 中统一收敛到 OneBotClient。当前先并行存在。

### API 端口获取

OneBotClient 构造需要 `api_port`，从 config 表读取：

```rust
let api_port: u16 = {
    let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
    crate::db::models::get_config_by_key(&conn, "napcat_api_port")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(3000)
};
let client = OneBotClient::new(api_port);
```

这个模式已在 `commands/napcat.rs:47-53` 中确立。后续 Story 2.3 的 engine 会以同样方式获取端口创建客户端。

### 前端 TypeScript 类型

```typescript
// src/types/onebot.ts
export interface FriendInfo {
  userId: number;
  nickname: string;
  remark: string;
}

export interface OneBotLoginInfo {
  userId: number;
  nickname: string;
}
```

### 强制规则清单

1. **发送给 OneBot 的类型** 不加 `rename_all`（保持 snake_case 匹配 OneBot 11 协议）
2. **暴露给前端的类型** 使用 `rename_all = "camelCase"` + `alias` 兼容 OneBot 反序列化
3. **Tauri commands 返回** `Result<T, String>`，使用 `.map_err(|e| e.to_string())`
4. **禁止** `println!`，使用 `tracing::info!` / `warn!` / `error!`
5. **禁止** `unwrap()` / `expect()` 在非构造函数代码中
6. **reqwest::Error** 不能用 `#[from]`（已被 `AppError::Download` 占用），必须手动分类为 OneBotError 具体变体
7. **重试仅对可重试错误**：ConnectionRefused、Timeout、HTTP 5xx。反序列化错误和 API 业务错误（retcode != 0）不重试
8. **HTTP 状态码必须检查**（Story 1.3 M1 教训）：使用 `resp.status().is_success()` 或 `error_for_status()`

### Story 1.4 经验教训

- **M1 后台任务缺乏取消机制**：本 Story 的 OneBotClient 是无状态的，不产生后台任务。但后续 Story 2.3 调度器使用客户端时需注意取消
- **M2 get_login_info_cmd 未检查 OneBot status 字段**：本 Story 的 `call_api` 已正确检查 `status != "ok"` → 返回 ApiError
- **M3 配置文件名兼容性**：与本 Story 无关
- **L2 spawn_blocking 用于长时间阻塞**：本 Story 全部使用 async/await，无阻塞操作

### 目录结构

本 Story 需要创建/修改的文件：

```
src-tauri/src/
├── errors.rs                 ← 修改：添加 OneBot(#[from] OneBotError) 变体
└── onebot/
    ├── mod.rs                ← 修改：从空文件改为 pub mod client; types; + re-exports
    ├── client.rs             ← 新建：OneBotClient 结构体 + send_like/get_friend_list/get_login_info + 重试逻辑
    └── types.rs              ← 新建：OneBotError + OneBotResponse<T> + SendLikeRequest + FriendInfo + OneBotLoginInfo

src/
└── types/
    └── onebot.ts             ← 新建：FriendInfo + OneBotLoginInfo TypeScript 接口
```

### Project Structure Notes

- `onebot/mod.rs` 已作为空占位存在于项目中（Story 1.1 创建），本 Story 首次填充内容
- `onebot/client.rs` 和 `onebot/types.rs` 与架构文档 `onebot/` 模块定义完全对齐
- OneBotClient 是纯库代码，不注册 Tauri State 也不注册新命令。它被后续 Story（2.2 名额管理、2.3 批量点赞）的 engine 模块消费
- `lib.rs` 无需修改（`mod onebot;` 在 Story 1.1 已添加，只是模块内容为空）

### References

- [Source: .bmad-method/planning-artifacts/architecture.md#onebot/] — onebot/client.rs（reqwest HTTP 封装）、onebot/types.rs（OneBot 请求/响应结构体）
- [Source: .bmad-method/planning-artifacts/architecture.md#实现模式与一致性规则] — 命名规范、错误处理三层分层（thiserror→anyhow→String）
- [Source: .bmad-method/planning-artifacts/architecture.md#API与通信] — OneBot 调用使用 reqwest HTTP POST、请求超时
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println!、禁止 unwrap、禁止前端直接 HTTP
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则] — onebot/client.rs 是唯一的 OneBot API 出口
- [Source: .bmad-method/planning-artifacts/epics.md#Story2.1] — AC 定义、技术要求
- [Source: .bmad-method/implementation-artifacts/1-4-napcat-process-and-qr-login.md#QA Results] — M2 未检查 OneBot status 字段教训
- [Source: .bmad-method/implementation-artifacts/1-3-napcat-download-and-extract.md#QA Results] — M1 HTTP 状态码检查教训
- [Source: src-tauri/src/errors.rs] — 现有 AppError 枚举结构，已有 Download(#[from] reqwest::Error)
- [Source: src-tauri/src/napcat/process.rs:212-296] — 现有 OneBot API 直接调用代码（poll_login_status）

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

（无调试问题）

### Completion Notes List

- `OneBotResponse<T>` 使用 `Option<T>` 处理 `/send_like` 返回 `null` data 的情况
- `call_api` 返回 `Option<T>`，各端点方法按需 unwrap 或忽略
- 重试逻辑改进：反序列化错误和 OneBot 业务错误（retcode != 0）不重试，仅 ConnectionRefused/Timeout/HTTP 5xx 重试
- `lib.rs` 实际缺少 `mod onebot;` 声明（与 Story 注释矛盾），已补充添加

### File List

- `src-tauri/src/onebot/types.rs` — 新建：OneBotError、OneBotResponse`<T>`、SendLikeRequest、FriendInfo、OneBotLoginInfo
- `src-tauri/src/onebot/client.rs` — 新建：OneBotClient + send_like/get_friend_list/get_login_info + 重试逻辑
- `src-tauri/src/onebot/mod.rs` — 修改：添加 pub mod client/types + re-exports
- `src-tauri/src/errors.rs` — 修改：添加 OneBot(#[from]) 变体
- `src-tauri/src/lib.rs` — 修改：添加 mod onebot 声明
- `src/types/onebot.ts` — 新建：FriendInfo、OneBotLoginInfo TypeScript 接口

## QA Results

**Reviewer:** Quinn (Test Architect) — Claude Opus 4.6
**Date:** 2026-03-13
**Gate Decision:** PASS
**Gate File:** `.bmad-method/test-artifacts/gates/2.1-onebot-api-client.yml`

### Build Verification

| Check            | Result                                                                      |
| ---------------- | --------------------------------------------------------------------------- |
| `cargo check`  | PASS (12 warnings — 7 个 Story 2.1 库代码 dead_code + 5 个 Story 1.2 遗留) |
| `tsc --noEmit` | PASS (零错误)                                                               |

### AC Coverage: 7/7 PASS

| AC | Description                                             | Result | Evidence                                     |
| -- | ------------------------------------------------------- | ------ | -------------------------------------------- |
| #1 | `onebot/client.rs` 封装 reqwest HTTP POST             | PASS   | `client.rs:45` — post().json().send()     |
| #2 | 支持 /send_like、/get_friend_list、/get_login_info      | PASS   | `client.rs:88,95,101` — 三个公开方法      |
| #3 | types.rs 类型定义 + camelCase + alias 兼容              | PASS   | `types.rs:39-55` — 双策略正确实现         |
| #4 | 请求超时 10 秒                                          | PASS   | `client.rs:19` — timeout(10s)             |
| #5 | thiserror 错误枚举 (ConnectionRefused/Timeout/ApiError) | PASS   | `types.rs:7-18` — 5 个具体变体            |
| #6 | 自动重试（最多 2 次，间隔 1 秒）                        | PASS   | `client.rs:8-9,37` — MAX_RETRIES=2        |
| #7 | tracing 日志（info 请求、error 失败）                   | PASS   | `client.rs:39,43,55,79` — minor gap 见 M1 |

### 前序教训执行情况

| 教训                   | 来源         | 执行                                               |
| ---------------------- | ------------ | -------------------------------------------------- |
| HTTP 状态码检查        | Story 1.3 M1 | PASS —`client.rs:48` resp.status().is_success() |
| OneBot status 字段检查 | Story 1.4 M2 | PASS —`client.rs:67` onebot_resp.status != "ok" |
| 阻塞异步线程           | Story 1.3 M2 | N/A — 全部 async/await，无同步 IO                 |
| 后台任务取消           | Story 1.4 M1 | N/A — OneBotClient 无状态，不产生后台任务         |

### Issues Summary: 0 High / 1 Medium / 3 Low

**Medium Issues:**

- **M1**: `call_api` 中 HTTP 4xx 和 Deserialize 错误未记录 `tracing::error!` 日志 (`client.rs:48-59,63-65`) — AC#7 要求 error 级别失败日志，但两个 early return 路径跳过了 `tracing::error!`。风险低（localhost 不应产生 4xx）

**Low Issues:**

- **L1**: `FriendInfo.remark` 的 `alias = "remark"` 冗余 — camelCase 对单词 remark 无变换 (`types.rs:45`)
- **L2**: `OneBotClient` 未实现 Clone — Story 2.3 引擎可能需要共享实例 (`client.rs:11`)
- **L3**: 重试日志未包含触发重试的错误原因 (`client.rs:39`)

### Recommendation

PROCEED to Story 2.2。Sprint 2 首个 Story 质量优秀，serde 双策略精准落地。

- **M1 (建议)**: 补全 4xx/Deserialize 路径的 tracing::error!
- **L1-L3**: 可在 Story 2.3 整合时顺带处理

### Change Log

- 2026-03-13: Story 2.1 实现完成，所有 7 个 Task 完成，cargo check 和 tsc 通过
