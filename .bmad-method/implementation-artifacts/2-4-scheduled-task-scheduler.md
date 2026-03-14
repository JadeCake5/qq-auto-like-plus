# Story 2.4: 定时任务调度器

Status: Done

## Story

As a 用户,
I want 应用在我配置的时间自动开始点赞,
so that 我不需要手动触发。

## Acceptance Criteria

1. **Given** 批量点赞执行器已就位（Story 2.3）**When** 应用启动 **Then** `engine/scheduler.rs` 使用 `tokio-cron-scheduler` 注册定时任务
2. **Given** config 表有 `schedule_hour` 和 `schedule_minute` **When** 读取定时配置 **Then** 定时时间从 config 表读取（默认 00:05）
3. **Given** 定时任务已注册 **When** 到达定时时间 **Then** 自动触发批量点赞（调用 `run_batch_like`）
4. **Given** 前端修改了定时时间 **When** 收到 `config:updated` 事件（key 为 `schedule_hour` 或 `schedule_minute`）**Then** scheduler 重新注册任务（无需重启应用）
5. **Given** 需要手动触发或暂停 **When** 前端调用命令 **Then** 提供 Tauri commands：`pause_engine`、`resume_engine`
6. **Given** 暂停状态 **When** 应用重启 **Then** 暂停状态从 config 表（`engine_paused`）恢复
7. **Given** 调度器运行中 **When** 前端查询 **Then** `get_next_run_time` 返回下次执行时间的 ISO 8601 字符串
8. **Given** 调度器状态变化 **When** 启动/暂停/恢复/重新调度 **Then** 通过 Tauri event emit(`engine:status-changed`) 通知前端

## Tasks / Subtasks

- [x] Task 1: 创建数据库迁移 004 — 新增调度器相关配置 (AC: #6)
  - [x] 1.1 创建 `src-tauri/migrations/004_scheduler_config.sql`：添加 `engine_paused` 默认值 `'false'`
  - [x] 1.2 在 `db/migrations.rs` 的 MIGRATIONS 数组追加 004 条目
- [x] Task 2: 实现 scheduler 模块 (AC: #1, #2, #3, #4)
  - [x] 2.1 创建 `src-tauri/src/engine/scheduler.rs`
  - [x] 2.2 定义 `LikeScheduler` 结构体（持有 `JobScheduler`、当前 job uuid、配置引用）
  - [x] 2.3 实现 `LikeScheduler::new()` 初始化调度器
  - [x] 2.4 实现 `start(db, onebot, app, running)` — 读取配置、创建 cron job、启动调度器
  - [x] 2.5 实现 `reschedule(hour, minute)` — 移除旧 job、创建新 job
  - [x] 2.6 实现 `pause()` / `resume()` — 暂停/恢复调度
  - [x] 2.7 实现 `get_next_run_time()` — 返回下次执行时间
- [x] Task 3: 实现 Tauri commands (AC: #5, #7, #8)
  - [x] 3.1 创建 `src-tauri/src/commands/engine.rs`：`pause_engine`、`resume_engine`、`get_next_run_time`、`get_engine_status`
  - [x] 3.2 更新 `commands/mod.rs` 添加 `pub mod engine`
  - [x] 3.3 在 `lib.rs` 注册新 commands + 初始化 scheduler
- [x] Task 4: 配置热更新监听 (AC: #4)
  - [x] 4.1 在 scheduler 启动时监听 `config:updated` 事件
  - [x] 4.2 收到 `schedule_hour` 或 `schedule_minute` 变更时调用 `reschedule`
- [x] Task 5: 前端类型定义 (AC: #7, #8)
  - [x] 5.1 创建 `src/types/engine.ts`：`EngineStatus` 接口
- [x] Task 6: 构建验证 (AC: #1-#8)
  - [x] 6.1 `cargo check` 编译通过
  - [x] 6.2 `npx tsc --noEmit` TypeScript 类型检查通过

## Dev Notes

### 核心挑战：JobScheduler 生命周期 + 配置热更新

tokio-cron-scheduler 的 `JobScheduler` 是 `Send + Sync + Clone`（内部 Arc 包装），可以安全跨线程共享。关键难点是**动态重新调度**：用户在设置页修改定时时间后，必须移除旧任务、注册新任务，且不能重启整个调度器。

### tokio-cron-scheduler 0.13 API 要点

**Crate 已在 Cargo.toml 中声明：** `tokio-cron-scheduler = "0.13"`

**核心 API：**
- `JobScheduler::new().await?` — 创建调度器（使用默认内存存储）
- `sched.add(job).await?` — 注册任务，返回 `Uuid`
- `sched.remove(&uuid).await?` — 移除任务
- `sched.start().await?` — 启动（spawn tokio 任务，每 500ms tick 一次）
- `sched.shutdown().await?` — 停止调度器
- `sched.next_tick_for_job(uuid).await?` — 返回 `Option<DateTime<Utc>>`，下次执行时间

**Cron 表达式格式（6 字段）：** `sec min hour day_of_month month day_of_week`
- 默认 00:05 → `"0 5 0 * * *"`（秒=0 分=5 时=0）
- **注意：默认是 UTC 时间！** 必须使用 `Job::new_async_tz` 或 `JobBuilder` + `with_timezone` 指定本地时区

**时区处理（关键）：**
用户配置的 `schedule_hour`/`schedule_minute` 是本地时间。必须使用 `chrono-tz` 或 `chrono::Local` 转换。推荐方案：计算 UTC 偏移后生成 UTC cron 表达式，避免引入 `chrono-tz` 额外依赖。

```rust
// 本地时间转 UTC cron 表达式
fn build_cron_expr(local_hour: u32, local_minute: u32) -> String {
    use chrono::Local;
    let now = Local::now();
    let offset_secs = now.offset().local_minus_utc();
    let offset_hours = offset_secs / 3600;

    // 简化处理：只处理整小时偏移（覆盖 UTC+8 等常见情况）
    let utc_hour = ((local_hour as i32 - offset_hours) % 24 + 24) % 24;
    format!("0 {} {} * * *", local_minute, utc_hour)
}
```

**替代方案（更健壮）：** 使用 `JobBuilder::new().with_timezone(chrono_tz::Asia::Shanghai)` 直接指定时区。需要 `chrono-tz` 依赖。但 `chrono-tz` 是一个较大的 crate（包含所有时区数据），对包大小有影响。

**推荐方案：UTC 偏移计算**，因为本应用只需支持单一用户本地时区。

### LikeScheduler 结构体设计

```rust
// engine/scheduler.rs
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio_cron_scheduler::{JobScheduler, Job};
use uuid::Uuid;
use serde::Serialize;

use crate::db::DbState;
use crate::onebot::OneBotClientState;
use crate::commands::like::BatchLikeRunning;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub is_paused: bool,
    pub is_running_batch: bool,   // 当前是否在执行批量点赞
    pub next_run_time: Option<String>,  // ISO 8601
    pub schedule_hour: u32,
    pub schedule_minute: u32,
}

/// 调度器内部状态（需要跨 await 共享所以用 TokioMutex）
struct SchedulerInner {
    scheduler: JobScheduler,
    current_job_id: Option<Uuid>,
    is_paused: bool,
    schedule_hour: u32,
    schedule_minute: u32,
}

/// 公开的调度器句柄（Clone + Send + Sync）
#[derive(Clone)]
pub struct LikeScheduler {
    inner: Arc<TokioMutex<SchedulerInner>>,
}

pub type LikeSchedulerState = Arc<LikeScheduler>;
```

**为什么用 `tokio::sync::Mutex` 而不是 `std::sync::Mutex`：**
- `JobScheduler` 的方法全是 async 的（`add().await`、`remove().await`）
- 需要在持有锁的同时调用 `.await`
- `std::sync::Mutex` 不能跨 `.await` 持有（会 panic 或死锁）
- `tokio::sync::Mutex` 专门设计用于 async 上下文

**但不要把 `DbState`（`std::sync::Mutex`）和这里搞混！** DB 锁仍然是同步的、短暂持有的。Scheduler 锁是异步的。

### LikeScheduler 核心实现

```rust
impl LikeScheduler {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            inner: Arc::new(TokioMutex::new(SchedulerInner {
                scheduler,
                current_job_id: None,
                is_paused: false,
                schedule_hour: 0,
                schedule_minute: 5,
            })),
        })
    }

    /// 初始化并启动调度器
    pub async fn start(
        &self,
        db: DbState,
        onebot: OneBotClientState,
        app: tauri::AppHandle,
        running: BatchLikeRunning,
    ) -> Result<(), anyhow::Error> {
        // 1. 从 config 读取定时时间和暂停状态
        let (hour, minute, paused) = {
            let conn = db.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
            let h: u32 = crate::db::models::get_config_by_key(&conn, "schedule_hour")
                .ok().and_then(|c| c.value.parse().ok()).unwrap_or(0);
            let m: u32 = crate::db::models::get_config_by_key(&conn, "schedule_minute")
                .ok().and_then(|c| c.value.parse().ok()).unwrap_or(5);
            let p: bool = crate::db::models::get_config_by_key(&conn, "engine_paused")
                .ok().map(|c| c.value == "true").unwrap_or(false);
            (h, m, p)
        }; // conn 释放

        let mut inner = self.inner.lock().await;
        inner.schedule_hour = hour;
        inner.schedule_minute = minute;
        inner.is_paused = paused;

        // 2. 如果不是暂停状态，注册 cron job
        if !paused {
            let cron_expr = build_cron_expr(hour, minute);
            let job_id = self.add_cron_job(
                &mut inner, &cron_expr, db.clone(), onebot.clone(),
                app.clone(), running.clone(),
            ).await?;
            inner.current_job_id = Some(job_id);
        }

        // 3. 启动调度器
        inner.scheduler.start().await?;
        tracing::info!(
            "调度器已启动: {}:{:02} (UTC cron), paused={}",
            hour, minute, paused
        );

        // 4. 通知前端
        let status = self.build_status(&inner).await;
        let _ = app.emit("engine:status-changed", &status);

        Ok(())
    }
}
```

### Cron Job 创建（关键）

```rust
/// 创建并添加 cron job
async fn add_cron_job(
    &self,
    inner: &mut SchedulerInner,
    cron_expr: &str,
    db: DbState,
    onebot: OneBotClientState,
    app: tauri::AppHandle,
    running: BatchLikeRunning,
) -> Result<Uuid, anyhow::Error> {
    let job = Job::new_async(cron_expr, move |_uuid, _lock| {
        let db = db.clone();
        let onebot = onebot.clone();
        let app = app.clone();
        let running = running.clone();
        Box::pin(async move {
            // 防止重复执行
            if running.swap(true, std::sync::atomic::Ordering::SeqCst) {
                tracing::warn!("定时触发时批量点赞仍在执行，跳过本次");
                return;
            }
            tracing::info!("定时任务触发: 开始批量点赞");
            let result = crate::engine::like_executor::run_batch_like(
                &db, &onebot, &app, "scheduled"
            ).await;
            running.store(false, std::sync::atomic::Ordering::SeqCst);
            match result {
                Ok(r) => tracing::info!("定时批量点赞完成: {:?}", r),
                Err(e) => {
                    tracing::error!("定时批量点赞异常: {}", e);
                    let _ = app.emit("like:batch-error", e.to_string());
                }
            }
        })
    })?;
    let job_id = inner.scheduler.add(job).await?;
    tracing::info!("Cron job 已注册: {} (expr: {})", job_id, cron_expr);
    Ok(job_id)
}
```

**关键：`Job::new_async` 闭包签名**
- 闭包参数：`|uuid: Uuid, lock: JobToRunLock|`（第二个参数名可自选）
- 返回值：`Box<Pin<Future<Output = ()>>>`，用 `Box::pin(async move { ... })` 包装
- 闭包内 clone 所有需要的 Arc 资源，因为闭包是 `'static`

### 配置热更新实现

```rust
/// 重新调度（用户修改定时时间后调用）
pub async fn reschedule(
    &self,
    hour: u32,
    minute: u32,
    db: DbState,
    onebot: OneBotClientState,
    app: tauri::AppHandle,
    running: BatchLikeRunning,
) -> Result<(), anyhow::Error> {
    let mut inner = self.inner.lock().await;
    inner.schedule_hour = hour;
    inner.schedule_minute = minute;

    // 移除旧 job
    if let Some(old_id) = inner.current_job_id.take() {
        inner.scheduler.remove(&old_id).await?;
        tracing::info!("旧 cron job 已移除: {}", old_id);
    }

    // 如果不在暂停状态，注册新 job
    if !inner.is_paused {
        let cron_expr = build_cron_expr(hour, minute);
        let job_id = self.add_cron_job(
            &mut inner, &cron_expr, db, onebot,
            app.clone(), running,
        ).await?;
        inner.current_job_id = Some(job_id);
    }

    let status = self.build_status(&inner).await;
    let _ = app.emit("engine:status-changed", &status);
    tracing::info!("调度器已重新配置: {}:{:02}", hour, minute);
    Ok(())
}
```

### 暂停/恢复实现

```rust
pub async fn pause(
    &self,
    db: &DbState,
    app: &tauri::AppHandle,
) -> Result<(), anyhow::Error> {
    let mut inner = self.inner.lock().await;
    if inner.is_paused {
        return Ok(()); // 已暂停
    }

    // 移除 cron job（暂停期间不触发）
    if let Some(old_id) = inner.current_job_id.take() {
        inner.scheduler.remove(&old_id).await?;
    }
    inner.is_paused = true;

    // 持久化暂停状态
    {
        let conn = db.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        crate::db::models::upsert_config(&conn, "engine_paused", "true")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    let status = self.build_status(&inner).await;
    let _ = app.emit("engine:status-changed", &status);
    tracing::info!("调度器已暂停");
    Ok(())
}

pub async fn resume(
    &self,
    db: DbState,
    onebot: OneBotClientState,
    app: tauri::AppHandle,
    running: BatchLikeRunning,
) -> Result<(), anyhow::Error> {
    let mut inner = self.inner.lock().await;
    if !inner.is_paused {
        return Ok(()); // 未暂停
    }
    inner.is_paused = false;

    // 持久化恢复状态
    {
        let conn = db.lock().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        crate::db::models::upsert_config(&conn, "engine_paused", "false")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    // 重新注册 cron job
    let cron_expr = build_cron_expr(inner.schedule_hour, inner.schedule_minute);
    let job_id = self.add_cron_job(
        &mut inner, &cron_expr, db, onebot,
        app.clone(), running,
    ).await?;
    inner.current_job_id = Some(job_id);

    let status = self.build_status(&inner).await;
    let _ = app.emit("engine:status-changed", &status);
    tracing::info!("调度器已恢复");
    Ok(())
}
```

### get_next_run_time 实现

```rust
pub async fn get_next_run_time(&self) -> Option<String> {
    let inner = self.inner.lock().await;
    if let Some(job_id) = inner.current_job_id {
        match inner.scheduler.next_tick_for_job(job_id).await {
            Ok(Some(dt)) => Some(dt.to_rfc3339()),
            _ => None,
        }
    } else {
        None
    }
}

async fn build_status(&self, inner: &SchedulerInner) -> EngineStatus {
    let next_run = if let Some(job_id) = inner.current_job_id {
        inner.scheduler.next_tick_for_job(job_id).await
            .ok().flatten().map(|dt| dt.to_rfc3339())
    } else {
        None
    };

    EngineStatus {
        is_paused: inner.is_paused,
        is_running_batch: false, // 由 BatchLikeRunning AtomicBool 判断
        next_run_time: next_run,
        schedule_hour: inner.schedule_hour,
        schedule_minute: inner.schedule_minute,
    }
}
```

### Tauri Commands — commands/engine.rs

```rust
use tauri::{Emitter, State};
use crate::db::DbState;
use crate::engine::scheduler::{LikeSchedulerState, EngineStatus};
use crate::onebot::OneBotClientState;
use crate::commands::like::BatchLikeRunning;

#[tauri::command]
pub async fn pause_engine(
    scheduler: State<'_, LikeSchedulerState>,
    db: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    scheduler.pause(db.inner(), &app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_engine(
    scheduler: State<'_, LikeSchedulerState>,
    db: State<'_, DbState>,
    onebot: State<'_, OneBotClientState>,
    running: State<'_, BatchLikeRunning>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    scheduler.resume(
        db.inner().clone(),
        onebot.inner().clone(),
        app.clone(),
        running.inner().clone(),
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_next_run_time(
    scheduler: State<'_, LikeSchedulerState>,
) -> Result<Option<String>, String> {
    Ok(scheduler.get_next_run_time().await)
}

#[tauri::command]
pub async fn get_engine_status(
    scheduler: State<'_, LikeSchedulerState>,
    running: State<'_, BatchLikeRunning>,
) -> Result<EngineStatus, String> {
    let mut status = scheduler.get_status().await;
    status.is_running_batch = running.load(std::sync::atomic::Ordering::SeqCst);
    Ok(status)
}
```

### lib.rs 修改要点

```rust
// setup 中添加（在 batch_running 之后，napcat_state 之前）：
use crate::engine::scheduler;

// 初始化 scheduler（async 操作需要在 spawn 中执行）
let like_scheduler = {
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        scheduler::LikeScheduler::new().await
            .expect("failed to create scheduler")
    })
};
let scheduler_state: scheduler::LikeSchedulerState =
    std::sync::Arc::new(like_scheduler);
app.manage(scheduler_state.clone());

// 在 setup 最后、Ok(()) 之前启动 scheduler
let db_for_sched = db_state.clone();
let onebot_for_sched = onebot_client.clone();
let running_for_sched = batch_running.clone();
let app_handle = app.handle().clone();
let sched = scheduler_state.clone();

tokio::spawn(async move {
    if let Err(e) = sched.start(
        db_for_sched,
        onebot_for_sched,
        app_handle.clone(),
        running_for_sched,
    ).await {
        tracing::error!("调度器启动失败: {}", e);
    }

    // 监听配置更新事件进行热更新
    // 注意：Tauri 后端监听事件使用 app.listen()
});
```

**⚠️ 关键：setup 闭包中的 async 操作**

Tauri `setup` 闭包不是 async 的。两种处理方式：

1. **`tokio::runtime::Handle::current().block_on()`** — 用于必须同步完成的初始化（如 `JobScheduler::new()`）
2. **`tokio::spawn()`** — 用于后台启动调度器（不阻塞 setup）

推荐在 setup 中用 `block_on` 创建 scheduler，然后 `spawn` 启动它。

### 配置热更新监听

**方案 A：在 commands/settings.rs 的 `update_config` 中直接触发 reschedule**

```rust
// commands/settings.rs — 修改 update_config
#[tauri::command]
pub async fn update_config(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    scheduler: State<'_, LikeSchedulerState>,
    onebot: State<'_, OneBotClientState>,
    running: State<'_, BatchLikeRunning>,
    key: String,
    value: String,
) -> Result<(), String> {
    {
        let conn = db.lock().map_err(|e| e.to_string())?;
        models::upsert_config(&conn, &key, &value).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("config:updated", &key);

    // 定时配置变更时重新调度
    if key == "schedule_hour" || key == "schedule_minute" {
        let (hour, minute) = {
            let conn = db.lock().map_err(|e| e.to_string())?;
            let h: u32 = models::get_config_by_key(&conn, "schedule_hour")
                .ok().and_then(|c| c.value.parse().ok()).unwrap_or(0);
            let m: u32 = models::get_config_by_key(&conn, "schedule_minute")
                .ok().and_then(|c| c.value.parse().ok()).unwrap_or(5);
            (h, m)
        };
        scheduler.reschedule(
            hour, minute,
            db.inner().clone(),
            onebot.inner().clone(),
            app.clone(),
            running.inner().clone(),
        ).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

**方案 A 的注意事项：**
- `update_config` 从 `pub fn` 变为 `pub async fn`（因为 reschedule 是 async）
- 需要额外注入 `scheduler`、`onebot`、`running` State
- 前端 invoke 签名不变（Tauri 透明处理 sync → async）

**方案 B：在后台监听事件** — 更解耦但实现更复杂，需要 `app.listen()` + channel 转发

**推荐方案 A** — 更简单直接，且 `update_config` 已经知道哪个 key 被改了。

### engine/mod.rs 更新

```rust
pub mod like_executor;
pub mod quota;
pub mod scheduler;

pub use quota::QuotaStatus;
pub use like_executor::{BatchLikeProgress, BatchLikeResult};
pub use scheduler::{LikeScheduler, LikeSchedulerState, EngineStatus};
```

### commands/mod.rs 更新

```rust
pub mod engine;
pub mod like;
pub mod napcat;
pub mod settings;
```

### invoke_handler 更新

```rust
.invoke_handler(tauri::generate_handler![
    commands::settings::get_config,
    commands::settings::update_config,
    commands::napcat::download_napcat,
    commands::napcat::import_napcat,
    commands::napcat::get_napcat_status,
    commands::napcat::start_napcat,
    commands::napcat::stop_napcat,
    commands::napcat::get_login_info_cmd,
    commands::like::get_daily_stats,
    commands::like::start_batch_like,
    commands::engine::pause_engine,
    commands::engine::resume_engine,
    commands::engine::get_next_run_time,
    commands::engine::get_engine_status,
])
```

### 数据库迁移 004

```sql
-- 004_scheduler_config.sql
INSERT OR IGNORE INTO config (key, value) VALUES ('engine_paused', 'false');
```

**说明：** `schedule_hour` 和 `schedule_minute` 已在 001_init.sql 中定义（默认 0 和 5）。只需添加 `engine_paused` 键。

### 前端 TypeScript 类型 — src/types/engine.ts

```typescript
/** 引擎状态（对应 engine:status-changed 事件 + get_engine_status 命令） */
export interface EngineStatus {
  isPaused: boolean;
  isRunningBatch: boolean;
  nextRunTime: string | null;  // ISO 8601
  scheduleHour: number;
  scheduleMinute: number;
}
```

### 事件命名一览

| 事件名 | 方向 | Payload | 触发时机 |
|--------|------|---------|---------|
| `engine:status-changed` | Rust → 前端 | `EngineStatus` | 调度器启动/暂停/恢复/重新调度 |
| `config:updated` | Rust → 前端 | `String` (key) | 配置更新（已有，Story 1.2） |

### uuid 依赖

`tokio-cron-scheduler` 的 `add()` 返回 `uuid::Uuid`。检查 `uuid` crate 是否已作为传递依赖可用。如果编译报 `Uuid` 找不到，在 Cargo.toml 添加：

```toml
uuid = { version = "1", features = ["v4"] }
```

通常 `tokio-cron-scheduler` 会 re-export 或引入 uuid 作为依赖，但如果需要在自己的代码中直接引用 `Uuid` 类型，可能需要显式声明。

### anyhow 使用

scheduler 模块内部使用 `anyhow::Error` 作为错误类型（应用层），在 Tauri commands 层用 `.map_err(|e| e.to_string())` 转换。这符合架构的三层错误处理模式。

### Story 2.3 教训应用

| 教训 | 本 Story 应用 |
|------|-------------|
| DB 锁不跨 await | scheduler 中 DB 读取用 `{}` 作用域限制，scheduler 自身用 `tokio::sync::Mutex` |
| BatchLikeRunning 防重复触发 | cron job 内复用同一个 `BatchLikeRunning` AtomicBool |
| `use tauri::Emitter` 导入 | 所有需要 emit 的文件都要导入 |
| rand rng 不跨 await | 本 Story 不涉及 rand，但记住模式 |

### 不要做的事

- **不要** 创建新的 `OneBotClient` — 复用 Tauri State 中已注册的实例
- **不要** 在 cron job 闭包中直接访问 Tauri State — 在 setup 时 clone 出 Arc 传入
- **不要** 用 `std::sync::Mutex` 包装 `JobScheduler` — 它的方法是 async 的，必须用 `tokio::sync::Mutex`
- **不要** 在 `setup` 闭包中直接 `.await` — setup 不是 async，用 `block_on` 或 `spawn`
- **不要** 硬编码时区偏移 — 通过 `chrono::Local` 动态获取
- **不要** 在暂停状态下保留 cron job — 暂停时移除 job，恢复时重新创建
- **不要** 忘记在 `resume` 时检查 `engine_paused` 配置 — 持久化状态必须与内存一致

### Project Structure Notes

本 Story 需要创建/修改的文件：

```
src-tauri/
├── migrations/
│   └── 004_scheduler_config.sql              ← 新建：engine_paused 默认值
├── src/
│   ├── lib.rs                                ← 修改：初始化 + 管理 LikeSchedulerState、注册新 commands
│   ├── db/
│   │   └── migrations.rs                     ← 修改：添加 004 迁移条目
│   ├── engine/
│   │   ├── mod.rs                            ← 修改：添加 pub mod scheduler + re-exports
│   │   └── scheduler.rs                      ← 新建：LikeScheduler + EngineStatus + build_cron_expr
│   └── commands/
│       ├── mod.rs                            ← 修改：添加 pub mod engine
│       ├── engine.rs                         ← 新建：pause_engine、resume_engine、get_next_run_time、get_engine_status
│       └── settings.rs                       ← 修改：update_config 变 async + 定时配置变更触发 reschedule

src/
├── types/
│   └── engine.ts                             ← 新建：EngineStatus TypeScript 接口
```

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story2.4] — AC 定义：定时任务调度器
- [Source: .bmad-method/planning-artifacts/architecture.md#engine/] — engine/scheduler.rs 定位
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式] — Tauri events emit/listen 模式
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则] — commands/ 唯一前端入口
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println、禁止 unwrap
- [Source: .bmad-method/implementation-artifacts/2-3-batch-like-executor.md] — run_batch_like API、BatchLikeRunning 模式、DB 锁教训
- [Source: src-tauri/src/engine/like_executor.rs] — run_batch_like 签名和依赖
- [Source: src-tauri/src/commands/like.rs] — BatchLikeRunning 类型定义、start_batch_like 模式
- [Source: src-tauri/src/commands/settings.rs] — update_config 当前实现
- [Source: src-tauri/src/lib.rs] — 当前 setup 模式和 State 注册
- [Source: src-tauri/src/errors.rs] — AppError 枚举
- [Source: src-tauri/Cargo.toml] — tokio-cron-scheduler = "0.13" 已声明
- [Source: src-tauri/migrations/001_init.sql] — schedule_hour/schedule_minute 默认值已有

## QA Results

**Reviewer:** Quinn (Test Architect) — Claude Opus 4.6
**Date:** 2026-03-13
**Gate Decision:** PASS
**Gate File:** `.bmad-method/test-artifacts/gates/2.4-scheduled-task-scheduler.yml`

### AC 验证矩阵

| AC | 描述 | 结果 |
|----|------|------|
| #1 | tokio-cron-scheduler 注册定时任务 | PASS |
| #2 | 从 config 表读取定时配置（默认 00:05）| PASS |
| #3 | 到达定时时间自动触发 run_batch_like | PASS |
| #4 | 前端修改配置后 scheduler 重新注册（无需重启）| PASS |
| #5 | pause_engine / resume_engine 命令 | PASS |
| #6 | 暂停状态持久化到 config 表，重启后恢复 | PASS |
| #7 | get_next_run_time 返回 ISO 8601 字符串 | PASS |
| #8 | engine:status-changed 事件通知前端 | PASS |

### Issues: 0 High, 0 Medium, 5 Low

- **L1**: `build_cron_expr` 仅处理整小时时区偏移（目标用户 UTC+8 不受影响）
- **L2**: `build_status_from` 硬编码 `is_running_batch: false`（命令查询正确）
- **L3**: `schedule_hour`/`schedule_minute` 无显式输入验证（cron 解析器兜底）
- **L4**: 窗口关闭时未显式 shutdown 调度器（进程退出自动清理）
- **L5**: `start_batch_like` 的 `like_type` 为 `"scheduled"` 而非 `"manual"`（Story 2.3 遗留）

### 亮点

- tokio::sync::Mutex 正确用于 async 上下文
- DB 锁 `{}` 作用域限制，完美应用 Story 2.3 教训
- BatchLikeRunning AtomicBool 复用防止并发执行
- 幂等 pause/resume 设计
- 架构合规性满分

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- cargo check 失败 #1: `uuid` crate 未显式声明 — tokio-cron-scheduler 不 re-export uuid::Uuid。修复：Cargo.toml 添加 `uuid = { version = "1", features = ["v4"] }`
- cargo check 失败 #2: `JobScheduler::next_tick_for_job` 需要 `&mut self`。修复：`build_status_from` 参数从 `&SchedulerInner` 改为 `&mut SchedulerInner`，所有 lock 改为 `let mut inner`

### Completion Notes List

- 配置热更新采用方案 A（直接在 update_config 中触发 reschedule），更简单直接
- update_config 从 sync fn 变为 async fn（因 reschedule 是 async），前端 invoke 签名不变
- 本地时间转 UTC cron 表达式使用 chrono::Local 偏移计算，避免引入 chrono-tz
- LikeScheduler 使用 tokio::sync::Mutex（async mutex），因为 JobScheduler 方法都是 async
- scheduler 在 setup 中用 block_on 创建，然后 tokio::spawn 后台启动
- 暂停/恢复会持久化 engine_paused 到 config 表，重启后恢复状态

### File List

- `src-tauri/Cargo.toml` — 修改：添加 uuid 依赖
- `src-tauri/migrations/004_scheduler_config.sql` — 新建：engine_paused 默认配置
- `src-tauri/src/db/migrations.rs` — 修改：添加 004 迁移条目
- `src-tauri/src/engine/mod.rs` — 修改：添加 pub mod scheduler + re-exports
- `src-tauri/src/engine/scheduler.rs` — 新建：LikeScheduler + EngineStatus + build_cron_expr + 完整调度逻辑
- `src-tauri/src/commands/engine.rs` — 新建：pause_engine、resume_engine、get_next_run_time、get_engine_status
- `src-tauri/src/commands/mod.rs` — 修改：添加 pub mod engine
- `src-tauri/src/commands/settings.rs` — 修改：update_config 变 async + 定时配置变更触发 reschedule
- `src-tauri/src/lib.rs` — 修改：初始化 LikeSchedulerState、注册新 commands、后台 spawn 启动调度器
- `src/types/engine.ts` — 新建：EngineStatus TypeScript 接口
