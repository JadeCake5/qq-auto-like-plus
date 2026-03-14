# Story 2.2: 名额管理模块

Status: Done

## Story

As a 用户,
I want 应用合理管理每天的点赞数量,
so that 降低风控风险。

## Acceptance Criteria

1. **Given** 数据库 daily_state 表和 config 表已就位 **When** 新的一天开始（零点）**Then** `engine/quota.rs` 自动重置当日名额（创建新的 daily_state 记录）
2. **Given** 名额系统运行中 **When** 计算可用名额 **Then** 可用定时名额 = daily_limit - reserved_for_reply - scheduled_count - manual_count
3. **Given** 点赞引擎执行点赞 **When** 每次点赞消耗名额 **Then** 更新 daily_state 的 liked_count（总人数）和对应类型计数（scheduled_count / reply_count / manual_count）
4. **Given** 收到回赞请求 **When** 消耗回赞名额 **Then** 回赞从预留池扣减（daily_state.reply_count），不影响定时名额
5. **Given** 名额已耗尽 **When** 尝试消耗名额 **Then** 返回 QuotaExhausted 错误，不执行点赞
6. **Given** 前端或引擎需要查询名额 **When** 调用 get_quota_status() **Then** 返回今日已用/剩余/总量的完整状态
7. **Given** 前端仪表盘加载 **When** 调用 invoke("get_daily_stats") **Then** 返回 QuotaStatus 供前端展示
8. **Given** 需要记录点赞历史 **When** 任何类型的点赞发生 **Then** like_history 表记录每次点赞结果（user_id、times、like_type、success、error_msg）

## Tasks / Subtasks

- [x] Task 1: 创建数据库迁移 002 (AC: #3, #8)
  - [x] 1.1 创建 `src-tauri/migrations/002_quota_and_history.sql`：为 daily_state 表添加 `scheduled_count`、`reply_count`、`manual_count` 列（ALTER TABLE ADD COLUMN）
  - [x] 1.2 在迁移中创建 `like_history` 表（id、user_id、times、like_type、success、error_msg、created_at）
  - [x] 1.3 创建 like_history 索引：`idx_like_history_created_at`、`idx_like_history_user_id`、`idx_like_history_user_date`
  - [x] 1.4 插入新配置默认值：`reserved_for_reply=10`、`batch_interval=3`、`reply_times=10`、`reply_delay_min=0`、`reply_delay_max=0`
  - [x] 1.5 在 `db/migrations.rs` 的 MIGRATIONS 数组中添加 002 迁移条目
- [x] Task 2: 扩展数据模型与 CRUD (AC: #3, #6, #8)
  - [x] 2.1 在 `db/models.rs` 中更新 `DailyState` 结构体：添加 `scheduled_count`、`reply_count`、`manual_count` 字段
  - [x] 2.2 更新 `upsert_today_state` 函数签名以支持新字段
  - [x] 2.3 添加 `increment_daily_count(conn, date, like_type)` 函数：原子递增指定类型计数和 liked_count 总计
  - [x] 2.4 定义 `LikeHistory` 结构体（id、user_id、times、like_type、success、error_msg、created_at）
  - [x] 2.5 实现 `insert_like_history(conn, user_id, times, like_type, success, error_msg)` 函数
  - [x] 2.6 实现 `has_liked_today(conn, user_id, date)` 函数：查询 like_history 判断今日是否已赞过该用户
- [x] Task 3: 实现名额管理模块 (AC: #1, #2, #4, #5, #6)
  - [x] 3.1 创建 `src-tauri/src/engine/quota.rs`：定义 `QuotaStatus` 结构体（date、daily_limit、reserved_for_reply、total_liked、scheduled_count、reply_count、manual_count、available_scheduled、available_reply）
  - [x] 3.2 实现 `ensure_today_state(conn, daily_limit)` 函数：确保今日 daily_state 记录存在（不存在则创建）
  - [x] 3.3 实现 `get_quota_status(conn)` 函数：读取 config 中的 daily_limit 和 reserved_for_reply，结合 daily_state 计算各类剩余名额
  - [x] 3.4 实现 `try_consume_quota(conn, like_type)` 函数：检查对应类型名额是否充足，充足则递增计数并返回 Ok(())，不足则返回 QuotaExhausted 错误
  - [x] 3.5 实现 `record_like(conn, user_id, times, like_type, success, error_msg)` 函数：写入 like_history 并消耗名额（原子操作）
  - [x] 3.6 实现 `has_liked_today(conn, user_id)` 函数：委托 db::models 判断今日是否已赞
- [x] Task 4: 添加错误类型 (AC: #5)
  - [x] 4.1 在 `errors.rs` 的 AppError 枚举中添加 `QuotaExhausted(String)` 变体
- [x] Task 5: 实现 Tauri Command (AC: #7)
  - [x] 5.1 创建 `src-tauri/src/commands/like.rs`：实现 `get_daily_stats` Tauri command
  - [x] 5.2 在 `commands/mod.rs` 中添加 `pub mod like;`
  - [x] 5.3 在 `lib.rs` 的 `invoke_handler` 中注册 `commands::like::get_daily_stats`
- [x] Task 6: 连接模块 (AC: #1-#8)
  - [x] 6.1 编辑 `engine/mod.rs`：添加 `pub mod quota;` 和 re-export
  - [x] 6.2 确保 `lib.rs` 已有 `mod engine;`（当前存在空占位）
- [x] Task 7: 前端类型定义 (AC: #7)
  - [x] 7.1 创建 `src/types/stats.ts`：定义 `QuotaStatus` 和 `DailyStats` TypeScript 接口
  - [x] 7.2 更新 `src/types/config.ts` 的 AppConfig：添加 `reservedForReply`、`batchInterval`、`replyTimes`、`replyDelayMin`、`replyDelayMax` 字段
- [x] Task 8: 构建验证 (AC: #1-#8)
  - [x] 8.1 `cargo check` 编译通过
  - [x] 8.2 `npx tsc --noEmit` TypeScript 类型检查通过

## Dev Notes

### 数据库迁移 002 — SQL 定义

```sql
-- 002_quota_and_history.sql

-- 为 daily_state 表添加分类计数列
-- 注意：SQLite ALTER TABLE 只支持 ADD COLUMN，不支持修改已有列
ALTER TABLE daily_state ADD COLUMN scheduled_count INTEGER DEFAULT 0;
ALTER TABLE daily_state ADD COLUMN reply_count INTEGER DEFAULT 0;
ALTER TABLE daily_state ADD COLUMN manual_count INTEGER DEFAULT 0;

-- 点赞历史记录表
CREATE TABLE IF NOT EXISTS like_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,                    -- QQ 号
    times INTEGER NOT NULL DEFAULT 10,           -- 本次点赞次数
    like_type TEXT NOT NULL CHECK(like_type IN ('scheduled', 'reply', 'manual')),
    success INTEGER NOT NULL DEFAULT 1,          -- 0=失败, 1=成功
    error_msg TEXT,                              -- 失败时的错误信息
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引：按时间查询（日报表、清理）
CREATE INDEX IF NOT EXISTS idx_like_history_created_at ON like_history(created_at);
-- 索引：按用户查询（互动排行）
CREATE INDEX IF NOT EXISTS idx_like_history_user_id ON like_history(user_id);
-- 复合索引：查询用户当日是否已赞（高频操作）
CREATE INDEX IF NOT EXISTS idx_like_history_user_date ON like_history(user_id, created_at);

-- 新增配置默认值
INSERT OR IGNORE INTO config (key, value) VALUES ('reserved_for_reply', '10');
INSERT OR IGNORE INTO config (key, value) VALUES ('batch_interval', '3');
INSERT OR IGNORE INTO config (key, value) VALUES ('reply_times', '10');
INSERT OR IGNORE INTO config (key, value) VALUES ('reply_delay_min', '0');
INSERT OR IGNORE INTO config (key, value) VALUES ('reply_delay_max', '0');
```

### 迁移注册 — migrations.rs 修改

```rust
// 在 MIGRATIONS 数组中追加：
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../../migrations/001_init.sql")),
    ("002_quota_and_history", include_str!("../../migrations/002_quota_and_history.sql")),
];
```

**注意事项（来自 Story 1.2 QA M2）：** 当前迁移执行器 `execute_batch(sql)` 和 `INSERT _migrations` 非原子操作。002 迁移 SQL 全部幂等（ALTER TABLE 如果列已存在会报错但用 execute_batch 整体回滚），如果遇到问题，将 ALTER TABLE 改为逐条执行并用 try 吞掉 "duplicate column" 错误。

**推荐方案：** 在 `execute_batch` 外包裹一个事务：

```rust
// migrations.rs 增强（可选，但推荐）
if !applied {
    conn.execute_batch(&format!("BEGIN; {} COMMIT;", sql))
        .or_else(|e| {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(e)
        })?;
    conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
    tracing::info!("Applied migration: {}", name);
}
```

### DailyState 结构体更新

```rust
// db/models.rs — 更新 DailyState
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyState {
    pub date: String,
    pub liked_count: i32,         // 总已赞人数（所有类型之和）
    pub target_count: i32,        // 每日目标（= daily_limit）
    pub is_completed: bool,       // 是否完成
    pub last_run_at: Option<String>,
    // 新增字段（Story 2.2）
    pub scheduled_count: i32,     // 定时点赞人数
    pub reply_count: i32,         // 回赞人数
    pub manual_count: i32,        // 手动点赞人数
}
```

**重要：** `liked_count` 保持为总已赞人数的含义。恒等式：`liked_count = scheduled_count + reply_count + manual_count`。每次 `increment_daily_count` 应同时递增 `liked_count` 和对应类型计数。

### increment_daily_count 实现

```rust
// db/models.rs — 新增
/// 原子递增每日计数：同时更新 liked_count 和对应类型计数
pub fn increment_daily_count(
    conn: &Connection,
    date: &str,
    like_type: &str,
) -> Result<(), rusqlite::Error> {
    let type_column = match like_type {
        "scheduled" => "scheduled_count",
        "reply" => "reply_count",
        "manual" => "manual_count",
        _ => return Err(rusqlite::Error::InvalidParameterName(
            format!("无效的 like_type: {}", like_type)
        )),
    };

    // 确保今日记录存在
    conn.execute(
        "INSERT OR IGNORE INTO daily_state (date) VALUES (?1)",
        [date],
    )?;

    // 原子递增 liked_count 和对应类型计数
    conn.execute(
        &format!(
            "UPDATE daily_state SET liked_count = liked_count + 1, {} = {} + 1, last_run_at = CURRENT_TIMESTAMP WHERE date = ?1",
            type_column, type_column
        ),
        [date],
    )?;

    Ok(())
}
```

### LikeHistory 模型

```rust
// db/models.rs — 新增
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeHistory {
    pub id: i64,
    pub user_id: i64,
    pub times: i32,
    pub like_type: String,       // "scheduled" | "reply" | "manual"
    pub success: bool,
    pub error_msg: Option<String>,
    pub created_at: String,
}

/// 插入点赞历史记录
pub fn insert_like_history(
    conn: &Connection,
    user_id: i64,
    times: i32,
    like_type: &str,
    success: bool,
    error_msg: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO like_history (user_id, times, like_type, success, error_msg) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![user_id, times, like_type, success as i32, error_msg],
    )?;
    Ok(())
}

/// 查询用户今日是否已被赞过（包含所有类型的成功点赞）
pub fn has_liked_today(
    conn: &Connection,
    user_id: i64,
    date: &str,
) -> Result<bool, rusqlite::Error> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM like_history WHERE user_id = ?1 AND success = 1 AND DATE(created_at) = ?2",
        rusqlite::params![user_id, date],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
```

### QuotaStatus 结构体

```rust
// engine/quota.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaStatus {
    pub date: String,
    pub daily_limit: i32,            // 每日总名额
    pub reserved_for_reply: i32,     // 回赞预留名额
    pub total_liked: i32,            // 今日已赞总人数
    pub scheduled_count: i32,        // 定时点赞人数
    pub reply_count: i32,            // 回赞人数
    pub manual_count: i32,           // 手动点赞人数
    pub available_scheduled: i32,    // 可用定时名额 = daily_limit - reserved_for_reply - scheduled_count - manual_count
    pub available_reply: i32,        // 可用回赞名额 = reserved_for_reply - reply_count
}
```

### 名额管理核心逻辑 — engine/quota.rs

```rust
// engine/quota.rs
use rusqlite::Connection;
use crate::db::models;
use crate::errors::AppError;

/// 获取当前日期字符串（YYYY-MM-DD）
fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
// 注意：项目当前未引入 chrono crate。
// 两个选择：
//   1. 添加 chrono 依赖（推荐，后续 Story 也会用到）
//   2. 手动用 std::time 格式化（复杂且不值得）
// 推荐方案：在 Cargo.toml 添加 chrono = { version = "0.4", features = ["serde"] }

/// 确保今日 daily_state 记录存在
pub fn ensure_today_state(conn: &Connection) -> Result<(), AppError> {
    let date = today();
    conn.execute(
        "INSERT OR IGNORE INTO daily_state (date) VALUES (?1)",
        [&date],
    ).map_err(AppError::Database)?;
    Ok(())
}

/// 获取名额状态（核心查询）
pub fn get_quota_status(conn: &Connection) -> Result<QuotaStatus, AppError> {
    let date = today();
    ensure_today_state(conn)?;

    // 从 config 读取限制参数
    let daily_limit: i32 = models::get_config_by_key(conn, "daily_limit")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(50);
    let reserved_for_reply: i32 = models::get_config_by_key(conn, "reserved_for_reply")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(10);

    // 从 daily_state 读取今日计数
    let state = models::get_today_state(conn, &date)
        .map_err(AppError::Database)?;

    let (total_liked, scheduled_count, reply_count, manual_count) = match state {
        Some(s) => (s.liked_count, s.scheduled_count, s.reply_count, s.manual_count),
        None => (0, 0, 0, 0),
    };

    let available_scheduled = (daily_limit - reserved_for_reply - scheduled_count - manual_count).max(0);
    let available_reply = (reserved_for_reply - reply_count).max(0);

    Ok(QuotaStatus {
        date,
        daily_limit,
        reserved_for_reply,
        total_liked,
        scheduled_count,
        reply_count,
        manual_count,
        available_scheduled,
        available_reply,
    })
}

/// 尝试消耗名额（核心决策函数）
/// 成功返回 Ok(())，名额不足返回 QuotaExhausted
pub fn try_consume_quota(conn: &Connection, like_type: &str) -> Result<(), AppError> {
    let status = get_quota_status(conn)?;

    match like_type {
        "scheduled" | "manual" => {
            let available = if like_type == "scheduled" {
                status.available_scheduled
            } else {
                // 手动点赞也从定时池消耗
                status.available_scheduled
            };
            if available <= 0 {
                return Err(AppError::QuotaExhausted(format!(
                    "定时/手动名额已耗尽（已用 {}/{}）",
                    status.scheduled_count + status.manual_count,
                    status.daily_limit - status.reserved_for_reply
                )));
            }
        }
        "reply" => {
            if status.available_reply <= 0 {
                return Err(AppError::QuotaExhausted(format!(
                    "回赞名额已耗尽（已用 {}/{}）",
                    status.reply_count,
                    status.reserved_for_reply
                )));
            }
        }
        _ => {
            return Err(AppError::QuotaExhausted(format!(
                "无效的点赞类型: {}", like_type
            )));
        }
    }

    // 递增计数
    let date = today();
    models::increment_daily_count(conn, &date, like_type)
        .map_err(AppError::Database)?;

    Ok(())
}

/// 记录点赞并消耗名额（组合操作，供 like_executor 和 reply_handler 调用）
pub fn record_like(
    conn: &Connection,
    user_id: i64,
    times: i32,
    like_type: &str,
    success: bool,
    error_msg: Option<&str>,
) -> Result<(), AppError> {
    // 写入历史记录
    models::insert_like_history(conn, user_id, times, like_type, success, error_msg)
        .map_err(AppError::Database)?;
    Ok(())
}

/// 检查用户今日是否已被赞过
pub fn has_liked_today(conn: &Connection, user_id: i64) -> Result<bool, AppError> {
    let date = today();
    models::has_liked_today(conn, user_id, &date)
        .map_err(AppError::Database)
}
```

### 错误类型扩展

```rust
// errors.rs — 添加变体
#[derive(Error, Debug)]
pub enum AppError {
    // ... 已有变体 ...

    #[error("名额已耗尽: {0}")]
    QuotaExhausted(String),
}
```

### Tauri Command — commands/like.rs

```rust
// commands/like.rs
use tauri::State;
use crate::db::DbState;
use crate::engine::quota;

#[tauri::command]
pub fn get_daily_stats(db: State<'_, DbState>) -> Result<quota::QuotaStatus, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    quota::get_quota_status(&conn).map_err(|e| e.to_string())
}
```

### engine/mod.rs 模块声明

```rust
// engine/mod.rs
pub mod quota;

pub use quota::QuotaStatus;
```

### 前端 TypeScript 类型 — src/types/stats.ts

```typescript
// src/types/stats.ts

/** 每日名额状态（对应 Rust QuotaStatus） */
export interface QuotaStatus {
  date: string;
  dailyLimit: number;          // 每日总名额
  reservedForReply: number;    // 回赞预留名额
  totalLiked: number;          // 今日已赞总人数
  scheduledCount: number;      // 定时点赞人数
  replyCount: number;          // 回赞人数
  manualCount: number;         // 手动点赞人数
  availableScheduled: number;  // 可用定时名额
  availableReply: number;      // 可用回赞名额
}
```

### AppConfig 类型扩展 — src/types/config.ts

在现有 AppConfig 接口中添加：

```typescript
export interface AppConfig {
  // ... 已有字段 ...
  reservedForReply: number;    // 回赞预留名额（默认 10）
  batchInterval: number;       // 批次间隔秒数（默认 3）
  replyTimes: number;          // 回赞次数（默认 10）
  replyDelayMin: number;       // 回赞最小延迟秒数
  replyDelayMax: number;       // 回赞最大延迟秒数
}
```

同时更新 `parseConfigEntries()` 函数以解析新字段。

### 日期工具 — chrono 依赖

项目当前 **未引入 chrono crate**。名额管理模块需要获取当前日期（YYYY-MM-DD 格式）来查询 daily_state 和 like_history。

**推荐方案：** 在 `src-tauri/Cargo.toml` 添加 chrono：

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**替代方案（不添加新依赖）：** 使用 `time` crate（Rust 标准库的 SystemTime 无法直接格式化为 YYYY-MM-DD，需要额外计算，不推荐）。

**注意：** 如果选择不引入 chrono，可以在 SQL 中使用 `DATE('now', 'localtime')` 获取当前日期，但这样 Rust 代码中需要传递 SQL 函数而非参数，降低灵活性。建议引入 chrono。

### try_consume_quota 和 record_like 的调用时序

后续 Story 2.3（批量点赞执行器）和 Story 5.2（回赞处理）将按以下流程调用本模块：

```
1. has_liked_today(user_id) → 已赞则跳过
2. try_consume_quota(like_type) → 名额不足则停止/跳过
3. onebot_client.send_like(user_id, times) → 实际调用 API
4. record_like(user_id, times, like_type, success, error_msg) → 记录结果
```

**关键设计决策：** `try_consume_quota` 先消耗名额再执行点赞。如果点赞失败，名额已消耗但 `record_like` 记录 `success=false`。这是保守策略——宁可少赞几个人也不要超额触发风控。后续如果需要"失败退还名额"机制，可在 quota.rs 添加 `refund_quota()` 函数。

### DailyState 查询的 get_today_state 修改

当前 `get_today_state` 函数签名：

```rust
pub fn get_today_state(conn: &Connection, date: &str) -> Result<Option<DailyState>, rusqlite::Error>
```

需要修改 SELECT 语句以包含新列：

```rust
pub fn get_today_state(conn: &Connection, date: &str) -> Result<Option<DailyState>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT date, liked_count, target_count, is_completed, last_run_at, scheduled_count, reply_count, manual_count FROM daily_state WHERE date = ?1"
    )?;
    let result = stmt.query_row([date], |row| {
        Ok(DailyState {
            date: row.get(0)?,
            liked_count: row.get(1)?,
            target_count: row.get(2)?,
            is_completed: row.get(3)?,
            last_run_at: row.get(4)?,
            scheduled_count: row.get(5)?,
            reply_count: row.get(6)?,
            manual_count: row.get(7)?,
        })
    });
    match result {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}
```

### upsert_today_state 函数更新

当前签名：

```rust
pub fn upsert_today_state(
    conn: &Connection,
    date: &str,
    liked_count: i32,
    target_count: i32,
    is_completed: bool,
) -> Result<(), rusqlite::Error>
```

添加新参数或改为接受 DailyState 引用。推荐保持简单——本 Story 不修改 upsert 签名，因为 quota.rs 使用 `increment_daily_count` 进行原子递增，而非全量覆盖。如果确实需要覆盖式更新，可在后续 Story 补充。

### 强制规则清单

1. **所有新 Rust 结构体** 必须标注 `#[serde(rename_all = "camelCase")]`
2. **Tauri commands** 返回 `Result<T, String>`，使用 `.map_err(|e| e.to_string())`
3. **数据库表名** snake_case（`like_history`）、列名 snake_case（`user_id`, `like_type`）
4. **索引命名** `idx_{表}_{列}`（`idx_like_history_user_id`）
5. **禁止** `println!`，使用 `tracing::info!` / `warn!` / `error!`
6. **禁止** `unwrap()` / `expect()` 在非初始化代码中，使用 `?` 操作符
7. **事件命名** 使用 `namespace:action` 格式（如 `like:quota-updated`）
8. **日期格式** 数据库使用 SQLite DATETIME，Rust 代码使用 chrono 格式化为 `YYYY-MM-DD`
9. **布尔值** SQLite 中 `0`/`1`，Rust 中 `bool`（rusqlite 自动转换）
10. **like_type 值** 严格为 `"scheduled"` / `"reply"` / `"manual"` 之一，使用 CHECK 约束

### Story 2.1 QA 教训应用

| 教训 | 来源 | 本 Story 应用 |
|------|------|-------------|
| HTTP 状态码检查 | Story 1.3 M1 | N/A — 本 Story 无 HTTP 调用 |
| OneBot status 字段检查 | Story 1.4 M2 | N/A — 本 Story 不调用 OneBot API |
| 迁移非原子操作 | Story 1.2 M2 | 注意：002 迁移中 ALTER TABLE 如列已存在会报错，建议事务包裹或逐条 try |
| config/mod.rs 被绕过 | Story 1.2 M1 | commands/like.rs 直接调用 engine/quota → db/models，不经过 config/mod.rs（与现有模式一致） |
| OneBotClient 未实现 Clone | Story 2.1 L2 | N/A — 本 Story 不直接使用 OneBotClient |
| call_api 4xx/Deserialize 缺日志 | Story 2.1 M1 | N/A — 本 Story 不调用 call_api |

### 架构边界遵守

- `engine/quota.rs` 通过 `db::models` 访问数据库 — **不直接执行 SQL**（除 ensure_today_state 中的 INSERT OR IGNORE）
- `commands/like.rs` 通过 `engine::quota` 获取数据 — **不直接访问 db 模块**
- 本模块不注册任何 Tauri State — 使用已有的 `DbState`
- 本模块不调用 OneBot API — 纯粹的名额管理逻辑

### 关于 ensure_today_state 中直接执行 SQL

`ensure_today_state` 包含一条 `INSERT OR IGNORE INTO daily_state`。严格来说应该通过 `db::models` 封装。两个方案：

1. **在 models.rs 中添加 `ensure_daily_state(conn, date)` 函数**（推荐，保持边界清晰）
2. **在 quota.rs 中直接执行**（简单但违反边界规则）

推荐方案 1：在 models.rs 中添加：

```rust
/// 确保指定日期的 daily_state 记录存在（不存在则创建默认值）
pub fn ensure_daily_state(conn: &Connection, date: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO daily_state (date) VALUES (?1)",
        [date],
    )?;
    Ok(())
}
```

然后 `quota.rs` 调用 `models::ensure_daily_state(&conn, &date)?;`

### Project Structure Notes

本 Story 需要创建/修改的文件：

```
src-tauri/
├── Cargo.toml                              ← 修改：添加 chrono 依赖
├── migrations/
│   └── 002_quota_and_history.sql            ← 新建：daily_state 扩展 + like_history 表
├── src/
│   ├── lib.rs                               ← 修改：invoke_handler 注册 get_daily_stats
│   ├── errors.rs                            ← 修改：添加 QuotaExhausted 变体
│   ├── db/
│   │   ├── migrations.rs                    ← 修改：MIGRATIONS 添加 002 条目
│   │   └── models.rs                        ← 修改：DailyState 新字段 + LikeHistory 模型 + 新 CRUD 函数
│   ├── engine/
│   │   ├── mod.rs                           ← 修改：添加 pub mod quota + re-export
│   │   └── quota.rs                         ← 新建：QuotaStatus + 名额管理逻辑
│   └── commands/
│       ├── mod.rs                           ← 修改：添加 pub mod like
│       └── like.rs                          ← 新建：get_daily_stats Tauri command

src/
├── types/
│   ├── stats.ts                             ← 新建：QuotaStatus TypeScript 接口
│   └── config.ts                            ← 修改：AppConfig 添加新配置字段
```

## QA Results

### Gate Decision: PASS

**Reviewer:** Quinn (Test Architect) — Claude Opus 4.6
**Date:** 2026-03-13

**AC Coverage:** 8/8 PASS
**Build:** cargo check PASS (0 errors, 21 warnings — 9 from 2.2 library code), tsc --noEmit PASS (0 errors)

### Issues Summary

| Severity | Count |
|----------|-------|
| High | 0 |
| Medium | 2 |
| Low | 5 |

**M1:** `has_liked_today` 使用 `DATE(created_at)` 函数查询，无法利用 `idx_like_history_user_date` 复合索引。Story 2.3 中此函数将被 200+ 好友逐个调用，建议改为范围查询 `created_at >= ? AND created_at < ?`。(models.rs:191)

**M2:** `try_consume_quota` 调用链中 `today()` 被独立调用 3 次（get_quota_status 内 2 次 + 自身 1 次），午夜边界可能跨天。建议 get_quota_status/try_consume_quota 接受 `date` 参数，调用者统一获取。(quota.rs:72,34-36,100)

**L1:** `record_like` Dev Notes 注释为"组合操作（记录+消耗名额）"，实际仅记录。实现正确但文档误导。
**L2:** 无效 like_type 使用 QuotaExhausted 错误类型（语义不精确），应为 InvalidParameter 类。
**L3:** Story 2.2 新增 models 函数返回 `rusqlite::Error` 而非 `AppError`，与 Story 1.2 不一致。
**L4:** quota.rs 关键业务路径（名额耗尽、点赞失败）缺少 tracing 日志。
**L5:** 21 个 dead_code warnings（9 来自 2.2），Story 2.3 接入后消除。

**Recommendation:** PROCEED to Story 2.3。M1 建议在 Story 2.3 实现批量点赞时一并修复（直接影响批量性能）。M2 概率极低但建议在 Story 2.3 传入日期参数时顺带重构。

### References

- [Source: .bmad-method/planning-artifacts/architecture.md#engine/] — engine/quota.rs 名额管理算法
- [Source: .bmad-method/planning-artifacts/architecture.md#数据架构] — SQLite WAL、rusqlite 原生访问、嵌入式迁移
- [Source: .bmad-method/planning-artifacts/architecture.md#实现模式与一致性规则] — 命名规范、错误处理分层（thiserror→anyhow→String）
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则] — db/models.rs 唯一数据库访问点、commands/ 唯一前端入口
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println、禁止 unwrap、禁止绕过 db 模块
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri事件命名] — namespace:action 格式
- [Source: .bmad-method/planning-artifacts/architecture.md#数据格式规范] — camelCase JSON、SQLite 布尔 0/1
- [Source: .bmad-method/planning-artifacts/epics.md#Story2.2] — AC 定义：名额管理模块
- [Source: .bmad-method/implementation-artifacts/1-2-sqlite-database-and-config.md] — DailyState 模型、迁移系统、CRUD 模式
- [Source: .bmad-method/implementation-artifacts/1-2-sqlite-database-and-config.md#QA Results M2] — 迁移非原子操作教训
- [Source: .bmad-method/implementation-artifacts/2-1-onebot-api-client.md] — OneBotClient 实现、serde 双策略、错误类型
- [Source: .bmad-method/implementation-artifacts/2-1-onebot-api-client.md#QA Results] — 前序教训
- [Source: src-tauri/src/db/models.rs] — 现有 DailyState、ConfigEntry CRUD
- [Source: src-tauri/src/errors.rs] — 现有 AppError 枚举
- [Source: src-tauri/src/engine/mod.rs] — 空占位文件
- [Source: src-tauri/migrations/001_init.sql] — 现有数据库 schema

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

无调试问题。

### Completion Notes List

- 迁移执行器增强：为 `run_migrations` 添加了事务包裹（BEGIN/COMMIT/ROLLBACK），应用 Story 1.2 QA M2 教训
- `ensure_daily_state` 按推荐方案 1 放在 `db/models.rs` 中，保持架构边界清晰
- `upsert_today_state` 签名未修改（按 Dev Notes 建议，quota.rs 使用 `increment_daily_count` 进行原子递增）
- 添加 `chrono` 依赖用于日期格式化
- 所有 Rust 结构体标注 `#[serde(rename_all = "camelCase")]`
- 所有新函数使用 `?` 操作符，无 `unwrap()`/`expect()`
- cargo check 通过（0 errors，warnings 均为后续 Story 将使用的函数）
- npx tsc --noEmit 通过

### File List

- `src-tauri/Cargo.toml` — 修改：添加 chrono 依赖
- `src-tauri/migrations/002_quota_and_history.sql` — 新建：daily_state 扩展 + like_history 表 + 索引 + 配置默认值
- `src-tauri/src/db/migrations.rs` — 修改：添加 002 迁移条目 + 事务包裹增强
- `src-tauri/src/db/models.rs` — 修改：DailyState 新字段 + ensure_daily_state + increment_daily_count + LikeHistory 模型 + insert_like_history + has_liked_today
- `src-tauri/src/errors.rs` — 修改：添加 QuotaExhausted 变体
- `src-tauri/src/engine/mod.rs` — 修改：添加 pub mod quota + re-export QuotaStatus
- `src-tauri/src/engine/quota.rs` — 新建：QuotaStatus + ensure_today_state + get_quota_status + try_consume_quota + record_like + has_liked_today
- `src-tauri/src/commands/like.rs` — 新建：get_daily_stats Tauri command
- `src-tauri/src/commands/mod.rs` — 修改：添加 pub mod like
- `src-tauri/src/lib.rs` — 修改：添加 mod engine + 注册 get_daily_stats command
- `src/types/stats.ts` — 新建：QuotaStatus TypeScript 接口
- `src/types/config.ts` — 修改：AppConfig 添加 5 个新配置字段 + parseConfigEntries 解析新字段
