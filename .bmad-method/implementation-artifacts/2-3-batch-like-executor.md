# Story 2.3: 定时批量点赞执行器

Status: Done

## Story

As a 用户,
I want 应用自动为我的好友点赞,
so that 我不需要手动操作。

## Acceptance Criteria

1. **Given** OneBot 客户端和名额管理模块已就位 **When** 触发批量点赞（定时或手动）**Then** `engine/like_executor.rs` 通过 `/get_friend_list` 获取完整好友列表
2. **Given** 好友列表已获取 **When** 开始批量点赞 **Then** 随机打乱好友顺序
3. **Given** 打乱后的好友列表 **When** 逐个点赞 **Then** 每人之间间隔 `batch_interval` 秒（默认 3 秒）
4. **Given** 点赞一个好友 **When** 调用 `/send_like` **Then** 每人点赞 `times_per_friend` 次（默认 10 次）
5. **Given** 好友列表中的某个好友 **When** 查询 `like_history` 表 **Then** 跳过当天已赞过的好友
6. **Given** 每次点赞前 **When** 检查名额 **Then** 名额耗尽时停止整个批量流程
7. **Given** 每次点赞完成 **When** 记录结果 **Then** 写入 `like_history` 表（user_id、times、like_type='scheduled'、success、error_msg）并更新 `daily_state` 计数
8. **Given** 单个好友点赞失败 **When** 遇到 API 错误 **Then** 记录日志并继续下一个（不中断整体流程）
9. **Given** 批量点赞进行中 **When** 每完成一个好友 **Then** 通过 Tauri event emit("like:progress") 实时推送进度（当前/总数、当前好友昵称）
10. **Given** 批量点赞全部完成 **When** 流程结束 **Then** emit("like:batch-complete") 通知前端
11. **Given** 需要存储好友信息 **When** 首次获取好友列表 **Then** 创建 `friends` 表并缓存好友数据

## Tasks / Subtasks

- [x] Task 1: 创建数据库迁移 003 — friends 表 (AC: #11)
  - [x] 1.1 创建 `src-tauri/migrations/003_friends.sql`：friends 表（user_id PK、nickname、remark、updated_at）
  - [x] 1.2 在 `db/migrations.rs` 的 MIGRATIONS 数组中添加 003 条目
  - [x] 1.3 在 `db/models.rs` 中添加 `upsert_friends_batch(conn, friends: &[FriendRow])` 批量更新好友信息
- [x] Task 2: 修复 Story 2.2 QA 遗留问题 (AC: #5, #6)
  - [x] 2.1 修复 `db/models.rs` 的 `has_liked_today` 使用范围查询替代 `DATE()` 函数（QA M1）
  - [x] 2.2 修改 `engine/quota.rs` 的 `get_quota_status` 和 `try_consume_quota` 接受 `date: &str` 参数（QA M2）
  - [x] 2.3 在 `engine/quota.rs` 关键路径添加 tracing 日志（QA L4）
- [x] Task 3: 实现批量点赞执行器 (AC: #1-#10)
  - [x] 3.1 创建 `src-tauri/src/engine/like_executor.rs`：定义 `BatchLikeProgress` 事件 payload 结构体
  - [x] 3.2 定义 `BatchLikeResult` 结构体（总数、成功数、跳过数、失败数）
  - [x] 3.3 实现 `run_batch_like(db, onebot_client, app_handle, like_type)` 异步函数：核心批量逻辑
  - [x] 3.4 在函数内：获取好友列表 → 随机打乱 → 逐个点赞循环（检查已赞→检查名额→调用API→记录结果→emit进度→sleep间隔）
  - [x] 3.5 批量完成后 emit("like:batch-complete") + 返回 BatchLikeResult
- [x] Task 4: 注册 OneBotClient 为 Tauri State (AC: #1)
  - [x] 4.1 在 `onebot/mod.rs` 中定义 `OneBotClientState = Arc<OneBotClient>`
  - [x] 4.2 在 `lib.rs` setup 中从 config 读取 api_port，创建 OneBotClient 并 manage()
- [x] Task 5: 实现 Tauri Commands (AC: #1, #9, #10)
  - [x] 5.1 在 `commands/like.rs` 添加 `start_batch_like` async command：读取 OneBotClientState + DbState + AppHandle，spawn tokio 任务执行批量，返回成功/已在运行
  - [x] 5.2 添加运行状态锁 `BatchLikeRunning = Arc<AtomicBool>` 防止重复触发
  - [x] 5.3 在 `lib.rs` 注册 `start_batch_like` command + manage BatchLikeRunning 状态
- [x] Task 6: 前端类型定义 (AC: #9, #10)
  - [x] 6.1 创建 `src/types/like.ts`：定义 `BatchLikeProgress` 和 `BatchLikeResult` TypeScript 接口
  - [x] 6.2 更新 `src/types/onebot.ts`：确认 `FriendInfo` 类型已有
- [x] Task 7: 构建验证 (AC: #1-#11)
  - [x] 7.1 `cargo check` 编译通过
  - [x] 7.2 `npx tsc --noEmit` TypeScript 类型检查通过

## Dev Notes

### 核心架构挑战：同步 DB + 异步 API + 后台执行

这是本 Story 的核心难点。当前代码库的状态：
- **DbState** = `Arc<Mutex<Connection>>`（同步锁，不能跨 await 持有）
- **OneBotClient** 方法是 `async`（`send_like`、`get_friend_list`）
- **批量点赞** 耗时较长（50人 × 3秒 = 2.5分钟），不能阻塞 Tauri command

**推荐方案：spawn tokio 后台任务**

```rust
// commands/like.rs — start_batch_like
#[tauri::command]
pub async fn start_batch_like(
    db: State<'_, DbState>,
    onebot: State<'_, OneBotClientState>,
    running: State<'_, BatchLikeRunning>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // 防止重复触发
    if running.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("批量点赞正在执行中".to_string());
    }

    let db = db.inner().clone();
    let onebot = onebot.inner().clone();
    let running = running.inner().clone();

    // spawn 后台任务，command 立即返回
    tokio::spawn(async move {
        let result = like_executor::run_batch_like(
            &db, &onebot, &app, "scheduled"
        ).await;

        running.store(false, std::sync::atomic::Ordering::SeqCst);

        match result {
            Ok(r) => tracing::info!("批量点赞完成: {:?}", r),
            Err(e) => {
                tracing::error!("批量点赞异常终止: {}", e);
                let _ = app.emit("like:batch-error", e.to_string());
            }
        }
    });

    Ok("批量点赞已启动".to_string())
}
```

**关键：DB 锁的持有模式**

```rust
// ✅ 正确：锁定 → 操作 → 释放 → 然后 await
{
    let conn = db.lock().map_err(|e| e.to_string())?;
    let already_liked = quota::has_liked_today(&conn, user_id)?;
    // conn 在作用域结束时释放
}
// 现在可以安全 await
onebot_client.send_like(user_id, times).await?;

// ❌ 错误：不能跨 await 持有 Mutex
let conn = db.lock().unwrap();
onebot_client.send_like(user_id, times).await?; // MutexGuard 跨 await！
```

### 数据库迁移 003 — friends 表

```sql
-- 003_friends.sql

-- 好友信息缓存表（每次批量点赞时同步更新）
CREATE TABLE IF NOT EXISTS friends (
    user_id INTEGER PRIMARY KEY,           -- QQ 号
    nickname TEXT NOT NULL DEFAULT '',      -- 昵称
    remark TEXT NOT NULL DEFAULT '',        -- 备注名
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**说明：** friends 表在 Epic 6 (Story 6.1) 会添加标签关联列，当前只需基础字段。

### 迁移注册

```rust
// db/migrations.rs — MIGRATIONS 数组追加
("003_friends", include_str!("../../migrations/003_friends.sql")),
```

### friends 批量更新 — db/models.rs

```rust
/// 好友信息行（用于批量 upsert）
pub struct FriendRow {
    pub user_id: i64,
    pub nickname: String,
    pub remark: String,
}

/// 批量 upsert 好友信息（从 OneBot get_friend_list 同步）
pub fn upsert_friends_batch(conn: &Connection, friends: &[FriendRow]) -> Result<(), rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO friends (user_id, nickname, remark, updated_at)
             VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
             ON CONFLICT(user_id) DO UPDATE SET
                nickname = excluded.nickname,
                remark = excluded.remark,
                updated_at = CURRENT_TIMESTAMP"
        )?;
        for f in friends {
            stmt.execute(params![f.user_id, f.nickname, f.remark])?;
        }
    }
    tx.commit()?;
    Ok(())
}
```

**注意：** 使用 `unchecked_transaction()` 而非 `transaction()`，因为 `Connection` 在 `Arc<Mutex<>>` 中是 `&Connection` 而非 `&mut Connection`。`unchecked_transaction` 在 rusqlite 中支持 `&Connection`。

### QA M1 修复 — has_liked_today 范围查询

**问题：** `DATE(created_at) = ?2` 无法命中 `idx_like_history_user_date` 复合索引。批量点赞时每个好友都会调用此函数。

```rust
// db/models.rs — 修改 has_liked_today
pub fn has_liked_today(
    conn: &Connection,
    user_id: i64,
    date: &str,
) -> Result<bool, rusqlite::Error> {
    // 使用范围查询命中复合索引 idx_like_history_user_date(user_id, created_at)
    let date_start = format!("{} 00:00:00", date);
    let date_end = format!("{} 23:59:59", date);
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM like_history WHERE user_id = ?1 AND success = 1 AND created_at >= ?2 AND created_at <= ?3",
        params![user_id, date_start, date_end],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
```

### QA M2 修复 — quota 函数接受 date 参数

**问题：** `today()` 在 `get_quota_status` 和 `try_consume_quota` 中各自独立调用，午夜边界可能跨天。

```rust
// engine/quota.rs — 公开 today() 供外部统一调用
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

// 所有内部方法改为接受 date 参数版本
pub fn get_quota_status_for_date(conn: &Connection, date: &str) -> Result<QuotaStatus, AppError> {
    // ... 使用传入的 date 替代 today()
}

pub fn try_consume_quota_for_date(conn: &Connection, like_type: &str, date: &str) -> Result<(), AppError> {
    // ... 使用传入的 date
}

// 保留无参版本作为便利接口（向后兼容 get_daily_stats command）
pub fn get_quota_status(conn: &Connection) -> Result<QuotaStatus, AppError> {
    get_quota_status_for_date(conn, &today())
}

pub fn try_consume_quota(conn: &Connection, like_type: &str) -> Result<(), AppError> {
    try_consume_quota_for_date(conn, like_type, &today())
}
```

### OneBotClient 状态注册

当前 `OneBotClient` 未注册为 Tauri State。需要在 `lib.rs` setup 中初始化。

```rust
// onebot/mod.rs — 添加状态类型
pub type OneBotClientState = std::sync::Arc<OneBotClient>;

// lib.rs — setup 中添加（在 db_state 初始化之后）
let conn = db_state.lock().expect("failed to lock db for init");
let api_port: u16 = models::get_config_by_key(&conn, "api_port")
    .ok()
    .and_then(|c| c.value.parse().ok())
    .unwrap_or(3000);
drop(conn); // 释放锁

let onebot_client: onebot::OneBotClientState =
    std::sync::Arc::new(onebot::OneBotClient::new(api_port));
app.manage(onebot_client);
```

**注意：** 当前 config 表可能没有 `api_port` 键。检查 `001_init.sql` 的默认配置。如果没有，需要在 003 迁移中添加：

```sql
INSERT OR IGNORE INTO config (key, value) VALUES ('api_port', '3000');
```

### BatchLikeRunning 状态 — 防止重复触发

```rust
// commands/like.rs 或 engine/mod.rs
pub type BatchLikeRunning = std::sync::Arc<std::sync::atomic::AtomicBool>;

// lib.rs — setup 中
let batch_running: commands::like::BatchLikeRunning =
    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
app.manage(batch_running);
```

### 核心批量点赞逻辑 — engine/like_executor.rs

```rust
use std::sync::Arc;
use serde::Serialize;
use tauri::Emitter;

use crate::db::DbState;
use crate::engine::quota;
use crate::db::models;
use crate::onebot::OneBotClient;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchLikeProgress {
    pub current: i32,          // 当前进度（第几个）
    pub total: i32,            // 总数
    pub user_id: i64,          // 当前好友 QQ
    pub nickname: String,      // 当前好友昵称
    pub success: bool,         // 本次是否成功
    pub skipped: bool,         // 是否跳过（已赞/名额不足）
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchLikeResult {
    pub total: i32,
    pub success_count: i32,
    pub skipped_count: i32,
    pub failed_count: i32,
}

pub async fn run_batch_like(
    db: &DbState,
    onebot: &Arc<OneBotClient>,
    app: &tauri::AppHandle,
    like_type: &str,
) -> Result<BatchLikeResult, AppError> {
    // 1. 统一获取当前日期（防止午夜跨天 — QA M2）
    let date = quota::today();

    // 2. 读取配置
    let (times_per_friend, batch_interval) = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        let tpf: i32 = models::get_config_by_key(&conn, "times_per_friend")
            .ok().and_then(|c| c.value.parse().ok()).unwrap_or(10);
        let bi: u64 = models::get_config_by_key(&conn, "batch_interval")
            .ok().and_then(|c| c.value.parse().ok()).unwrap_or(3);
        (tpf, bi)
    }; // conn 释放

    // 3. 获取好友列表（async）
    tracing::info!("批量点赞开始: 获取好友列表...");
    let friends = onebot.get_friend_list().await
        .map_err(AppError::OneBot)?;
    tracing::info!("获取到 {} 个好友", friends.len());

    // 4. 缓存好友信息到 friends 表
    {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        let rows: Vec<models::FriendRow> = friends.iter().map(|f| models::FriendRow {
            user_id: f.user_id,
            nickname: f.nickname.clone(),
            remark: f.remark.clone(),
        }).collect();
        if let Err(e) = models::upsert_friends_batch(&conn, &rows) {
            tracing::warn!("缓存好友信息失败（不影响点赞）: {}", e);
        }
    }

    // 5. 随机打乱
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    let mut shuffled = friends.clone();
    shuffled.shuffle(&mut rng);

    let total = shuffled.len() as i32;
    let mut success_count = 0i32;
    let mut skipped_count = 0i32;
    let mut failed_count = 0i32;

    // 6. 逐个点赞
    for (i, friend) in shuffled.iter().enumerate() {
        let current = (i + 1) as i32;

        // 6a. 检查是否已赞（DB 短暂加锁）
        let already_liked = {
            let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
            quota::has_liked_today_for_date(&conn, friend.user_id, &date)?
        };
        if already_liked {
            tracing::debug!("跳过已赞好友: {} ({})", friend.nickname, friend.user_id);
            skipped_count += 1;
            let _ = app.emit("like:progress", BatchLikeProgress {
                current, total,
                user_id: friend.user_id,
                nickname: friend.nickname.clone(),
                success: false, skipped: true,
            });
            continue;
        }

        // 6b. 检查名额（DB 短暂加锁）
        let quota_ok = {
            let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
            quota::try_consume_quota_for_date(&conn, like_type, &date)
        };
        if let Err(AppError::QuotaExhausted(msg)) = &quota_ok {
            tracing::info!("名额耗尽，停止批量点赞: {}", msg);
            let _ = app.emit("like:progress", BatchLikeProgress {
                current, total,
                user_id: friend.user_id,
                nickname: friend.nickname.clone(),
                success: false, skipped: true,
            });
            break; // 名额耗尽，终止整个流程
        }
        quota_ok.map_err(|e| {
            tracing::error!("名额检查异常: {}", e);
            e
        })?;

        // 6c. 调用 OneBot API 点赞（async）
        let like_result = onebot.send_like(friend.user_id, times_per_friend).await;

        // 6d. 记录结果（DB 短暂加锁）
        let (success, error_msg) = match &like_result {
            Ok(()) => {
                tracing::info!(
                    "[{}/{}] 点赞成功: {} ({}) ×{}",
                    current, total, friend.nickname, friend.user_id, times_per_friend
                );
                success_count += 1;
                (true, None)
            }
            Err(e) => {
                tracing::warn!(
                    "[{}/{}] 点赞失败: {} ({}) - {}",
                    current, total, friend.nickname, friend.user_id, e
                );
                failed_count += 1;
                (false, Some(e.to_string()))
            }
        };

        {
            let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
            quota::record_like(
                &conn, friend.user_id, times_per_friend,
                like_type, success, error_msg.as_deref(),
            )?;
        }

        // 6e. 推送进度事件
        let _ = app.emit("like:progress", BatchLikeProgress {
            current, total,
            user_id: friend.user_id,
            nickname: friend.nickname.clone(),
            success, skipped: false,
        });

        // 6f. 间隔等待（最后一个不等待）
        if i < shuffled.len() - 1 {
            tokio::time::sleep(std::time::Duration::from_secs(batch_interval)).await;
        }
    }

    // 7. 完成通知
    let result = BatchLikeResult {
        total, success_count, skipped_count, failed_count,
    };
    tracing::info!(
        "批量点赞完成: 总计{} 成功{} 跳过{} 失败{}",
        total, success_count, skipped_count, failed_count
    );
    let _ = app.emit("like:batch-complete", result.clone());

    Ok(result)
}
```

### has_liked_today_for_date 适配

`quota.rs` 中的 `has_liked_today` 当前调用 `today()`。需要添加接受 date 参数的版本：

```rust
// engine/quota.rs — 添加
pub fn has_liked_today_for_date(conn: &Connection, user_id: i64, date: &str) -> Result<bool, AppError> {
    models::has_liked_today(conn, user_id, date)
        .map_err(AppError::Database)
}
```

### rand 依赖

项目当前 **未引入 rand crate**。随机打乱好友顺序需要 rand。

```toml
# Cargo.toml 添加
rand = "0.9"
```

**rand 0.9 API 变化（2025 年发布）：**
- `rand::thread_rng()` 改为 `rand::rng()`
- `use rand::seq::SliceRandom` 仍然有效
- `shuffle(&mut rng)` 用法不变

### engine/mod.rs 更新

```rust
pub mod like_executor;
pub mod quota;

pub use quota::QuotaStatus;
pub use like_executor::{BatchLikeProgress, BatchLikeResult};
```

### commands/like.rs 完整更新

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;

use crate::db::DbState;
use crate::engine::{quota, like_executor};
use crate::onebot::OneBotClientState;

pub type BatchLikeRunning = Arc<AtomicBool>;

#[tauri::command]
pub fn get_daily_stats(db: State<'_, DbState>) -> Result<quota::QuotaStatus, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    quota::get_quota_status(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_batch_like(
    db: State<'_, DbState>,
    onebot: State<'_, OneBotClientState>,
    running: State<'_, BatchLikeRunning>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    if running.swap(true, Ordering::SeqCst) {
        return Err("批量点赞正在执行中".to_string());
    }

    let db = db.inner().clone();
    let onebot = onebot.inner().clone();
    let running_flag = running.inner().clone();

    tokio::spawn(async move {
        let result = like_executor::run_batch_like(
            &db, &onebot, &app, "scheduled"
        ).await;

        running_flag.store(false, Ordering::SeqCst);

        match result {
            Ok(r) => tracing::info!("批量点赞完成: {:?}", r),
            Err(e) => {
                tracing::error!("批量点赞异常终止: {}", e);
                let _ = app.emit("like:batch-error", e.to_string());
            }
        }
    });

    Ok("批量点赞已启动".to_string())
}
```

### lib.rs 修改要点

```rust
// setup 中添加（在 db_state 之后）：
use crate::onebot;
use crate::commands::like::BatchLikeRunning;

// OneBotClient 初始化
let api_port: u16 = {
    let conn = db_state.lock().expect("lock db for onebot init");
    crate::db::models::get_config_by_key(&conn, "api_port")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(3000)
};
let onebot_client: onebot::OneBotClientState =
    std::sync::Arc::new(onebot::OneBotClient::new(api_port));
app.manage(onebot_client);

// BatchLikeRunning 初始化
let batch_running: BatchLikeRunning =
    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
app.manage(batch_running);

// invoke_handler 添加：
commands::like::start_batch_like,
```

### 前端 TypeScript 类型 — src/types/like.ts

```typescript
/** 批量点赞进度事件 payload（对应 like:progress 事件） */
export interface BatchLikeProgress {
  current: number;         // 当前第几个
  total: number;           // 总好友数
  userId: number;          // 当前好友 QQ
  nickname: string;        // 当前好友昵称
  success: boolean;        // 本次是否成功
  skipped: boolean;        // 是否跳过
}

/** 批量点赞完成事件 payload（对应 like:batch-complete 事件） */
export interface BatchLikeResult {
  total: number;
  successCount: number;
  skippedCount: number;
  failedCount: number;
}
```

### 事件命名一览

| 事件名 | 方向 | Payload | 触发时机 |
|--------|------|---------|---------|
| `like:progress` | Rust → 前端 | `BatchLikeProgress` | 每完成一个好友（成功/失败/跳过） |
| `like:batch-complete` | Rust → 前端 | `BatchLikeResult` | 批量点赞全部完成 |
| `like:batch-error` | Rust → 前端 | `String` | 批量点赞异常终止 |

### Tauri Emitter trait

Tauri 2.0 中 `app.emit()` 需要 `use tauri::Emitter;`。确保在 `like_executor.rs` 和 `commands/like.rs` 中导入此 trait。

### api_port 配置默认值

检查 `001_init.sql` 是否有 `api_port` 配置。如果没有，在 `003_friends.sql` 中追加：

```sql
INSERT OR IGNORE INTO config (key, value) VALUES ('api_port', '3000');
```

### 强制规则清单

1. **所有新 Rust 结构体** 必须标注 `#[serde(rename_all = "camelCase")]`
2. **Tauri commands** 返回 `Result<T, String>`，使用 `.map_err(|e| e.to_string())`
3. **async Tauri commands** 使用 `pub async fn` + `#[tauri::command]`
4. **数据库锁** 不得跨 `.await` 持有 — 用 `{}` 作用域限制 MutexGuard 生命周期
5. **禁止** `println!`，使用 `tracing::info!` / `warn!` / `error!`
6. **禁止** `unwrap()` / `expect()` 在非初始化代码中，使用 `?` 操作符
7. **事件命名** 使用 `namespace:action` 格式
8. **日期格式** 使用 chrono `YYYY-MM-DD`
9. **like_type** 值严格为 `"scheduled"` / `"reply"` / `"manual"` 之一
10. **Emitter trait** 使用 `use tauri::Emitter;` 导入后才能调用 `app.emit()`

### Story 2.2 QA 教训直接应用

| 教训 | 来源 | 本 Story 应用 |
|------|------|-------------|
| `has_liked_today` 索引不命中 | 2.2 QA M1 | Task 2.1: 改为范围查询 `created_at >= ? AND created_at <= ?` |
| `today()` 多次独立调用 | 2.2 QA M2 | Task 2.2: quota 函数接受 date 参数 + 批量执行器统一获取一次 date |
| 关键业务路径缺日志 | 2.2 QA L4 | Task 2.3: quota 名额耗尽、点赞失败路径添加 tracing |
| 迁移事务包裹 | 1.2 QA M2 | 已在 2.2 修复，003 迁移继续遵守 |

### 关于 AppError 锁中毒错误

当前代码中 `db.lock().map_err(|e| AppError::NapCat(e.to_string()))` 语义不精确（将 Mutex PoisonError 包装为 NapCat 错误）。如果想修复，可在 `AppError` 中添加 `LockError(String)` 变体。但这是低优先级，不阻塞本 Story。

### 关于 `unchecked_transaction` 的使用

rusqlite 的 `Connection::transaction()` 需要 `&mut self`，但 `Arc<Mutex<Connection>>` 只能获取 `&Connection`。使用 `unchecked_transaction()` 可以绕过这个限制。这在 rusqlite 文档中是推荐的做法，适用于 `Arc<Mutex<Connection>>` 模式。

### 不要做的事

- **不要** 在 like_executor 中直接执行 SQL — 通过 `db::models` 和 `engine::quota`
- **不要** 在 like_executor 中创建新的 reqwest::Client — 使用传入的 OneBotClient
- **不要** 用 `std::thread::sleep` — 使用 `tokio::time::sleep`
- **不要** 在 emit 失败时 panic — 用 `let _ = app.emit(...)`
- **不要** 实现定时调度 — 那是 Story 2.4 的工作

### Project Structure Notes

本 Story 需要创建/修改的文件：

```
src-tauri/
├── Cargo.toml                              ← 修改：添加 rand 依赖
├── migrations/
│   └── 003_friends.sql                     ← 新建：friends 表 + api_port 默认值
├── src/
│   ├── lib.rs                              ← 修改：注册 OneBotClient State + BatchLikeRunning + start_batch_like command
│   ├── db/
│   │   ├── migrations.rs                   ← 修改：添加 003 迁移条目
│   │   └── models.rs                       ← 修改：FriendRow + upsert_friends_batch + 修复 has_liked_today
│   ├── engine/
│   │   ├── mod.rs                          ← 修改：添加 pub mod like_executor + re-exports
│   │   ├── quota.rs                        ← 修改：公开 today() + 添加 _for_date 版本函数 + tracing 日志
│   │   └── like_executor.rs               ← 新建：BatchLikeProgress + BatchLikeResult + run_batch_like
│   ├── onebot/
│   │   └── mod.rs                          ← 修改：添加 OneBotClientState 类型别名
│   └── commands/
│       └── like.rs                         ← 修改：添加 BatchLikeRunning + start_batch_like async command

src/
├── types/
│   └── like.ts                             ← 新建：BatchLikeProgress + BatchLikeResult TypeScript 接口
```

## QA Results

**Reviewer**: Quinn (Test Architect) — Claude Opus 4.6
**Review Date**: 2026-03-13
**Gate Decision**: PASS

### Build Verification
- cargo check: PASS (0 errors, 9 warnings — all unused library code)
- tsc --noEmit: PASS (0 errors)

### AC Coverage: 11/11 PASS
All acceptance criteria verified and met.

### Story 2.2 QA Fixes: 3/3 PASS
- M1 (索引不命中): ✅ 范围查询替代 DATE() 函数
- M2 (today() 跨天竞态): ✅ _for_date 变体 + 统一 date 获取
- L4 (缺少 tracing 日志): ✅ 关键路径日志已添加

### Issues Found

**Medium (1)**:
- M1: 名额在 API 调用前预消耗 — NapCat 崩溃时剩余名额全部浪费。建议添加连续失败检测（3次连续失败后提前终止）或改为 API 成功后再消耗名额。(`engine/like_executor.rs:111-133`)

**Low (4)**:
- L1: AppError::NapCat 用于 Mutex PoisonError 语义不精确 (`engine/like_executor.rs`)
- L2: engine/mod.rs re-exports 产生 unused warnings (`engine/mod.rs:4-5`)
- L3: has_liked_today 边界精度 `<= 23:59:59` vs `< next_date` (`db/models.rs:221`)
- L4: friends.clone() 可优化为 move (`engine/like_executor.rs:76`)

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story2.3] — AC 定义：定时批量点赞执行器
- [Source: .bmad-method/planning-artifacts/architecture.md#engine/] — engine 模块组织、like_executor.rs 定位
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式] — Tauri events emit/listen 模式
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则] — commands/ 唯一前端入口、db/models.rs 唯一 DB 访问
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println、禁止 unwrap、禁止直接 SQL
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri事件命名] — like:progress、like:batch-complete
- [Source: .bmad-method/implementation-artifacts/2-2-quota-management.md] — QuotaStatus、try_consume_quota、record_like、has_liked_today API
- [Source: .bmad-method/implementation-artifacts/2-2-quota-management.md#QA Results] — M1 索引不命中、M2 today() 跨天、L4 缺日志
- [Source: .bmad-method/implementation-artifacts/2-1-onebot-api-client.md] — OneBotClient::send_like/get_friend_list API、FriendInfo 类型
- [Source: src-tauri/src/onebot/client.rs] — OneBotClient 异步实现、重试逻辑
- [Source: src-tauri/src/engine/quota.rs] — 名额管理完整 API
- [Source: src-tauri/src/db/models.rs] — DailyState、LikeHistory CRUD
- [Source: src-tauri/src/lib.rs] — 当前 Tauri State 注册模式
- [Source: src-tauri/src/errors.rs] — AppError 枚举
- [Source: src-tauri/src/db/mod.rs] — DbState = Arc<Mutex<Connection>>

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- cargo check 首次编译失败: `ThreadRng` (rand 0.9) 不实现 `Send`，不能跨 await 持有。修复：将 `rng` 限制在 `{}` 作用域内，shuffle 完立即 drop。

### Completion Notes List

- 003_friends.sql 迁移包含 friends 表 + api_port 默认配置（001_init.sql 中缺少此键）
- has_liked_today 改为范围查询 `created_at >= ? AND created_at <= ?` 命中复合索引
- quota.rs 所有函数新增 `_for_date` 版本，便利接口保持向后兼容
- like_executor.rs 批量执行器统一获取一次 date 防止午夜跨天
- rand::rng() 必须在 {} 作用域内使用，避免 ThreadRng 跨 await
- engine/mod.rs re-exports 产生 unused warnings，不影响功能，后续 story 引用时自动消除

### File List

- `src-tauri/Cargo.toml` — 修改：添加 rand = "0.9" 依赖
- `src-tauri/migrations/003_friends.sql` — 新建：friends 表 + api_port 默认值
- `src-tauri/src/db/migrations.rs` — 修改：添加 003_friends 迁移条目
- `src-tauri/src/db/models.rs` — 修改：FriendRow + upsert_friends_batch + 修复 has_liked_today 范围查询
- `src-tauri/src/engine/mod.rs` — 修改：添加 pub mod like_executor + re-exports
- `src-tauri/src/engine/quota.rs` — 修改：公开 today() + _for_date 版本函数 + tracing 日志
- `src-tauri/src/engine/like_executor.rs` — 新建：BatchLikeProgress + BatchLikeResult + run_batch_like
- `src-tauri/src/onebot/mod.rs` — 修改：添加 OneBotClientState 类型别名
- `src-tauri/src/commands/like.rs` — 修改：BatchLikeRunning + start_batch_like async command
- `src-tauri/src/lib.rs` — 修改：注册 OneBotClient State + BatchLikeRunning + start_batch_like command
- `src/types/like.ts` — 新建：BatchLikeProgress + BatchLikeResult TypeScript 接口
