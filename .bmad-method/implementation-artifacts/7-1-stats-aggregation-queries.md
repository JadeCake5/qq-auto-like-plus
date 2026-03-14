# Story 7.1: 统计数据聚合查询

Status: Done

## Story

As a 用户,
I want 应用能汇总我的点赞历史数据,
so that 可以看到直观的统计信息。

## Acceptance Criteria

1. **聚合查询模块**：`stats/queries.rs` 提供完整的统计数据聚合查询函数
2. **日视图**：当天每小时点赞数分布（GROUP BY hour），返回 24 个数据点
3. **周视图**：近 7 天每日点赞数（GROUP BY date）
4. **月视图**：近 30 天每日点赞数（GROUP BY date）
5. **点赞类型占比**：定时(scheduled) / 回赞(reply) / 手动(manual) 各自总数
6. **好友互动排行**：被赞次数 TOP 10（GROUP BY user_id ORDER BY count DESC），含好友昵称
7. **数据保留策略**：定期清理 90 天前的 like_history 记录
8. **Tauri Commands**：提供 get_stats_daily、get_stats_weekly、get_stats_monthly、get_like_type_ratio、get_friend_ranking 五个命令
9. **索引优化**：查询利用现有 SQLite 索引（idx_like_history_created_at、idx_like_history_user_id）
10. **前端 Store**：创建 useStatsStore（Zustand）管理统计数据状态

## Tasks / Subtasks

- [x] Task 1: 实现 stats/queries.rs — 聚合查询函数 (AC: #1, #2, #3, #4, #5, #6, #9)
  - [x] 1.1 定义返回结构体：HourlyStats、DailyStats、LikeTypeRatio、FriendRanking
  - [x] 1.2 实现 get_hourly_stats(conn, date) — 当天每小时点赞数（24 个数据点，无数据小时补 0）
  - [x] 1.3 实现 get_daily_stats_range(conn, start_date, end_date) — 日期范围内每日点赞数
  - [x] 1.4 实现 get_weekly_stats(conn) — 近 7 天每日点赞数（复用 get_daily_stats_range）
  - [x] 1.5 实现 get_monthly_stats(conn) — 近 30 天每日点赞数（复用 get_daily_stats_range）
  - [x] 1.6 实现 get_like_type_ratio(conn, start_date, end_date) — 各类型点赞占比
  - [x] 1.7 实现 get_friend_ranking(conn, start_date, end_date, limit) — 好友互动排行 TOP N
  - [x] 1.8 在 stats/mod.rs 导出 queries 模块

- [x] Task 2: 实现数据保留清理 (AC: #7)
  - [x] 2.1 在 stats/queries.rs 新增 cleanup_old_history(conn, days) — 删除超过指定天数的 like_history 记录
  - [x] 2.2 在 lib.rs setup 中添加启动时清理调用（spawn 异步任务，清理 90 天前数据）

- [x] Task 3: 创建 commands/stats.rs — Tauri 命令 (AC: #8)
  - [x] 3.1 新增 get_stats_daily 命令 — 调用 get_hourly_stats，参数 date（可选，默认今日）
  - [x] 3.2 新增 get_stats_weekly 命令 — 调用 get_weekly_stats
  - [x] 3.3 新增 get_stats_monthly 命令 — 调用 get_monthly_stats
  - [x] 3.4 新增 get_like_type_ratio 命令 — 参数 period（"day"/"week"/"month"），计算对应日期范围
  - [x] 3.5 新增 get_friend_ranking 命令 — 参数 period（"day"/"week"/"month"），默认 TOP 10
  - [x] 3.6 新增 cleanup_history 命令 — 手动触发数据清理（可选）
  - [x] 3.7 在 commands/mod.rs 注册 `pub mod stats;`
  - [x] 3.8 在 lib.rs invoke_handler 注册所有新命令

- [x] Task 4: 注册 stats 模块到 lib.rs (AC: #1)
  - [x] 4.1 在 lib.rs 添加 `mod stats;`（当前未注册但目录已存在）

- [x] Task 5: 定义前端类型 (AC: #10)
  - [x] 5.1 扩展 src/types/stats.ts — 新增 HourlyStats、DailyStatsPoint、LikeTypeRatio、FriendRanking 接口

- [x] Task 6: 创建 src/lib/tauri.ts invoke wrappers (AC: #8)
  - [x] 6.1 新增 getStatsDaily(date?) wrapper
  - [x] 6.2 新增 getStatsWeekly() wrapper
  - [x] 6.3 新增 getStatsMonthly() wrapper
  - [x] 6.4 新增 getLikeTypeRatio(period) wrapper
  - [x] 6.5 新增 getFriendRanking(period) wrapper

- [x] Task 7: 创建 useStatsStore (AC: #10)
  - [x] 7.1 创建 src/stores/useStatsStore.ts
  - [x] 7.2 定义状态字段：hourlyData、weeklyData、monthlyData、typeRatio、friendRanking、currentPeriod、isLoading
  - [x] 7.3 实现 fetchDailyStats、fetchWeeklyStats、fetchMonthlyStats actions
  - [x] 7.4 实现 fetchLikeTypeRatio、fetchFriendRanking actions
  - [x] 7.5 实现 setPeriod action 切换视图并自动刷新对应数据

## Dev Notes

### 已有基础设施（直接复用！）

**数据库 — 无需新建 migration：**
- `like_history` 表（002_quota_and_history.sql）已有完整结构：id, user_id, times, like_type, success, error_msg, created_at
- 已有索引：`idx_like_history_created_at`、`idx_like_history_user_id`、`idx_like_history_user_date`（复合索引 user_id + created_at）
- `daily_state` 表（001_init.sql + 002 扩展）已有：date, liked_count, target_count, is_completed, last_run_at, scheduled_count, reply_count, manual_count
- `friends` 表（003_friends.sql）已有：user_id, nickname, remark, updated_at
- 最后 migration 编号：008_tag_strategy.sql — **本 Story 无需新增 migration**

**Rust 已有 — 必须复用：**

| 模块 | 函数/结构体 | 位置 | 用途 |
|------|------------|------|------|
| `db/models.rs` | `LikeHistory` | L158-168 | 复用结构体定义参考 |
| `db/models.rs` | `DailyState` | L61-72 | 复用结构体参考 |
| `db/models.rs` | `FriendRow` | L188-192 | 好友信息参考（排行需要昵称） |
| `db/models.rs` | `has_liked_today()` | L217-230 | 日期范围查询模式参考 |
| `db/mod.rs` | `DbState` | — | Tauri State 类型，`Arc<Mutex<Connection>>` |
| `errors.rs` | `AppError` | L1-21 | 复用，无需新增变体（Database variant 足够） |
| `stats/mod.rs` | 空文件 | — | **已存在但未导出**，需添加 `pub mod queries;` |
| `lib.rs` | 模块声明 | L3-12 | **未声明 `mod stats`**，需添加 |
| `commands/mod.rs` | 模块列表 | L1-7 | **未声明 `pub mod stats`**，需添加 |

**前端已有 — 必须复用：**

| 文件 | 状态 |
|------|------|
| `src/types/stats.ts` | 已有 QuotaStatus 接口 — **需扩展**新增统计相关类型 |
| `src/lib/tauri.ts` | 已有其他 invoke wrappers — **需扩展**新增 5 个统计 wrappers |
| `src/stores/` | 已有 5 个 store — **需新建** useStatsStore.ts |
| `src/pages/Statistics.tsx` | 已有占位页面（仅标题 "数据统计"）— **本 Story 不修改**（Story 7.2 处理） |

### 架构合规要点

**Rust 代码：**
- 所有对外结构体必须 `#[serde(rename_all = "camelCase")]`
- Tauri commands 返回 `Result<T, String>`，错误转换 `.map_err(|e| e.to_string())`
- 聚合查询放 `stats/queries.rs`（architecture.md 明确指定 `stats/queries.rs # 聚合查询`）
- DB 访问放 `stats/queries.rs` 本身（统计查询为只读聚合，不需要经过 `db/models.rs` — 但 **必须通过 DbState 获取连接**）
- 命令放 `commands/stats.rs`（architecture.md 指定 `commands/stats.rs # 统计查询`）
- 使用 `tracing::info!` / `tracing::error!`，禁止 `println!`
- 使用 `?` 操作符，禁止 `unwrap()` / `expect()` 在生产代码
- DB lock 不跨 await — 先锁取数据、释放、再返回

**前端代码：**
- Store 使用 Zustand `create<T>()` 模式，与 useLikeStore、useFriendsStore 一致
- Invoke wrapper 放 `src/lib/tauri.ts`
- 类型定义放 `src/types/stats.ts`

### 关键实现细节

**stats/queries.rs — 返回结构体定义：**

```rust
use rusqlite::{params, Connection};
use serde::Serialize;
use crate::errors::AppError;

/// 每小时统计数据点
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyStats {
    pub hour: i32,        // 0-23
    pub count: i32,       // 该小时的点赞数
}

/// 每日统计数据点
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStatsPoint {
    pub date: String,     // "2026-03-14"
    pub count: i32,       // 当日总点赞数
    pub scheduled: i32,   // 定时点赞数
    pub reply: i32,       // 回赞数
    pub manual: i32,      // 手动数
}

/// 点赞类型占比
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeTypeRatio {
    pub scheduled: i32,
    pub reply: i32,
    pub manual: i32,
    pub total: i32,
}

/// 好友互动排行
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendRanking {
    pub user_id: i64,
    pub nickname: String,
    pub total_likes: i32,    // 被赞总次数
}
```

**stats/queries.rs — 每小时统计（日视图）：**

```rust
/// 获取指定日期每小时点赞数分布（24 个数据点，无数据小时补 0）
pub fn get_hourly_stats(conn: &Connection, date: &str) -> Result<Vec<HourlyStats>, AppError> {
    let date_start = format!("{} 00:00:00", date);
    let date_end = format!("{} 23:59:59", date);

    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', created_at) AS INTEGER) AS hour, COUNT(*) AS count
         FROM like_history
         WHERE success = 1 AND created_at >= ?1 AND created_at <= ?2
         GROUP BY hour
         ORDER BY hour"
    )?;

    let mut hour_map = std::collections::HashMap::new();
    let rows = stmt.query_map(params![date_start, date_end], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
    })?;
    for row in rows {
        let (hour, count) = row?;
        hour_map.insert(hour, count);
    }

    // 补齐 24 小时
    let result: Vec<HourlyStats> = (0..24)
        .map(|h| HourlyStats {
            hour: h,
            count: *hour_map.get(&h).unwrap_or(&0),
        })
        .collect();

    Ok(result)
}
```

**stats/queries.rs — 日期范围统计（周/月视图通用）：**

```rust
/// 获取日期范围内每日点赞数（含类型分类）
pub fn get_daily_stats_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<DailyStatsPoint>, AppError> {
    let start = format!("{} 00:00:00", start_date);
    let end = format!("{} 23:59:59", end_date);

    let mut stmt = conn.prepare(
        "SELECT DATE(created_at) AS day,
                COUNT(*) AS total,
                SUM(CASE WHEN like_type = 'scheduled' THEN 1 ELSE 0 END) AS scheduled,
                SUM(CASE WHEN like_type = 'reply' THEN 1 ELSE 0 END) AS reply,
                SUM(CASE WHEN like_type = 'manual' THEN 1 ELSE 0 END) AS manual
         FROM like_history
         WHERE success = 1 AND created_at >= ?1 AND created_at <= ?2
         GROUP BY day
         ORDER BY day"
    )?;

    let rows = stmt.query_map(params![start, end], |row| {
        Ok(DailyStatsPoint {
            date: row.get(0)?,
            count: row.get(1)?,
            scheduled: row.get(2)?,
            reply: row.get(3)?,
            manual: row.get(4)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::Database)
}

/// 近 7 天每日点赞统计
pub fn get_weekly_stats(conn: &Connection) -> Result<Vec<DailyStatsPoint>, AppError> {
    let end = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start = (chrono::Local::now() - chrono::Duration::days(6))
        .format("%Y-%m-%d")
        .to_string();
    get_daily_stats_range(conn, &start, &end)
}

/// 近 30 天每日点赞统计
pub fn get_monthly_stats(conn: &Connection) -> Result<Vec<DailyStatsPoint>, AppError> {
    let end = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start = (chrono::Local::now() - chrono::Duration::days(29))
        .format("%Y-%m-%d")
        .to_string();
    get_daily_stats_range(conn, &start, &end)
}
```

**stats/queries.rs — 点赞类型占比：**

```rust
/// 获取指定日期范围内各类型点赞占比
pub fn get_like_type_ratio(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<LikeTypeRatio, AppError> {
    let start = format!("{} 00:00:00", start_date);
    let end = format!("{} 23:59:59", end_date);

    let result = conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN like_type = 'scheduled' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN like_type = 'reply' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN like_type = 'manual' THEN 1 ELSE 0 END), 0),
            COUNT(*)
         FROM like_history
         WHERE success = 1 AND created_at >= ?1 AND created_at <= ?2",
        params![start, end],
        |row| {
            Ok(LikeTypeRatio {
                scheduled: row.get(0)?,
                reply: row.get(1)?,
                manual: row.get(2)?,
                total: row.get(3)?,
            })
        },
    )?;

    Ok(result)
}
```

**stats/queries.rs — 好友互动排行 TOP N：**

```rust
/// 获取好友互动排行（被赞次数 TOP N）
pub fn get_friend_ranking(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
    limit: i32,
) -> Result<Vec<FriendRanking>, AppError> {
    let start = format!("{} 00:00:00", start_date);
    let end = format!("{} 23:59:59", end_date);

    let mut stmt = conn.prepare(
        "SELECT lh.user_id,
                COALESCE(f.nickname, CAST(lh.user_id AS TEXT)) AS nickname,
                COUNT(*) AS total_likes
         FROM like_history lh
         LEFT JOIN friends f ON lh.user_id = f.user_id
         WHERE lh.success = 1 AND lh.created_at >= ?1 AND lh.created_at <= ?2
         GROUP BY lh.user_id
         ORDER BY total_likes DESC
         LIMIT ?3"
    )?;

    let rows = stmt.query_map(params![start, end, limit], |row| {
        Ok(FriendRanking {
            user_id: row.get(0)?,
            nickname: row.get(1)?,
            total_likes: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::Database)
}
```

**stats/queries.rs — 数据保留清理：**

```rust
/// 清理超过指定天数的 like_history 记录
pub fn cleanup_old_history(conn: &Connection, retention_days: i32) -> Result<i64, AppError> {
    let cutoff = (chrono::Local::now() - chrono::Duration::days(retention_days as i64))
        .format("%Y-%m-%d 00:00:00")
        .to_string();

    let deleted = conn.execute(
        "DELETE FROM like_history WHERE created_at < ?1",
        [&cutoff],
    )?;

    if deleted > 0 {
        tracing::info!("已清理 {} 条超过 {} 天的点赞历史记录", deleted, retention_days);
    }

    Ok(deleted as i64)
}
```

**chrono 依赖 — 需确认或添加：**

检查 `Cargo.toml` 是否已有 `chrono` 依赖。`tokio-cron-scheduler` 依赖 `chrono`，因此 chrono 应已在依赖树中。但需确认是否在 `[dependencies]` 直接声明。若未声明，需添加：

```toml
chrono = { version = "0.4", features = ["serde"] }
```

**替代方案（不用 chrono）：** 若不想添加新的直接依赖，可用 SQLite 的 `date('now', '-6 days')` 在 SQL 中计算日期范围，避免 Rust 侧日期计算。示例：

```rust
// 无需 chrono，SQL 中计算日期
pub fn get_weekly_stats(conn: &Connection) -> Result<Vec<DailyStatsPoint>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT DATE(created_at) AS day,
                COUNT(*) AS total,
                SUM(CASE WHEN like_type = 'scheduled' THEN 1 ELSE 0 END),
                SUM(CASE WHEN like_type = 'reply' THEN 1 ELSE 0 END),
                SUM(CASE WHEN like_type = 'manual' THEN 1 ELSE 0 END)
         FROM like_history
         WHERE success = 1 AND created_at >= datetime('now', '-6 days', 'start of day')
         GROUP BY day ORDER BY day"
    )?;
    // ... map rows
}
```

**推荐**：若 chrono 已在依赖树中（通过 tokio-cron-scheduler），直接声明为直接依赖更清晰。若 dev 希望避免新依赖，使用 SQLite 内置日期函数。

**stats/mod.rs — 模块导出：**

```rust
pub mod queries;
```

**commands/stats.rs — 日期范围辅助函数 + Tauri 命令：**

```rust
use tauri::State;
use crate::db::DbState;
use crate::stats::queries;

/// 根据 period 字符串计算日期范围
fn date_range_for_period(period: &str) -> (String, String) {
    let now = chrono::Local::now();
    let end = now.format("%Y-%m-%d").to_string();
    let start = match period {
        "day" => now.format("%Y-%m-%d").to_string(),
        "week" => (now - chrono::Duration::days(6)).format("%Y-%m-%d").to_string(),
        "month" => (now - chrono::Duration::days(29)).format("%Y-%m-%d").to_string(),
        _ => (now - chrono::Duration::days(6)).format("%Y-%m-%d").to_string(),
    };
    (start, end)
}

#[tauri::command]
pub fn get_stats_daily(
    db: State<'_, DbState>,
    date: Option<String>,
) -> Result<Vec<queries::HourlyStats>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let target_date = date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });
    queries::get_hourly_stats(&conn, &target_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stats_weekly(
    db: State<'_, DbState>,
) -> Result<Vec<queries::DailyStatsPoint>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    queries::get_weekly_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stats_monthly(
    db: State<'_, DbState>,
) -> Result<Vec<queries::DailyStatsPoint>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    queries::get_monthly_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_like_type_ratio(
    db: State<'_, DbState>,
    period: String,
) -> Result<queries::LikeTypeRatio, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let (start, end) = date_range_for_period(&period);
    queries::get_like_type_ratio(&conn, &start, &end).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_friend_ranking(
    db: State<'_, DbState>,
    period: String,
) -> Result<Vec<queries::FriendRanking>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    let (start, end) = date_range_for_period(&period);
    queries::get_friend_ranking(&conn, &start, &end, 10).map_err(|e| e.to_string())
}
```

**lib.rs — 需要添加的修改：**

```rust
// 在 mod 声明区添加（L3-12 区域）：
mod stats;

// 在 invoke_handler 注册区追加（L255-283 区域）：
commands::stats::get_stats_daily,
commands::stats::get_stats_weekly,
commands::stats::get_stats_monthly,
commands::stats::get_like_type_ratio,
commands::stats::get_friend_ranking,

// 在 setup 块中 tokio::spawn 调度器之后添加数据清理：
let db_for_cleanup = db_state.clone();
tokio::spawn(async move {
    // 延迟 10 秒再执行，避免启动时阻塞
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    if let Ok(conn) = db_for_cleanup.lock() {
        match stats::queries::cleanup_old_history(&conn, 90) {
            Ok(deleted) => {
                if deleted > 0 {
                    tracing::info!("启动清理完成: 删除 {} 条过期记录", deleted);
                }
            }
            Err(e) => tracing::error!("启动清理失败: {}", e),
        }
    }
});
```

**commands/mod.rs — 添加：**

```rust
pub mod stats;
```

**前端 src/types/stats.ts — 扩展（保留已有 QuotaStatus）：**

```typescript
/** 每日名额状态（已有，保持不变） */
export interface QuotaStatus {
  date: string;
  dailyLimit: number;
  reservedForReply: number;
  totalLiked: number;
  scheduledCount: number;
  replyCount: number;
  manualCount: number;
  availableScheduled: number;
  availableReply: number;
}

/** 每小时统计数据点（日视图） */
export interface HourlyStats {
  hour: number;   // 0-23
  count: number;
}

/** 每日统计数据点（周/月视图） */
export interface DailyStatsPoint {
  date: string;       // "2026-03-14"
  count: number;      // 当日总点赞数
  scheduled: number;  // 定时点赞数
  reply: number;      // 回赞数
  manual: number;     // 手动数
}

/** 点赞类型占比 */
export interface LikeTypeRatio {
  scheduled: number;
  reply: number;
  manual: number;
  total: number;
}

/** 好友互动排行 */
export interface FriendRanking {
  userId: number;
  nickname: string;
  totalLikes: number;
}

/** 统计时间范围 */
export type StatsPeriod = "day" | "week" | "month";
```

**src/lib/tauri.ts — 新增 invoke wrappers（追加到文件末尾）：**

```typescript
import type { HourlyStats, DailyStatsPoint, LikeTypeRatio, FriendRanking, StatsPeriod } from "@/types/stats";

export async function getStatsDaily(date?: string): Promise<HourlyStats[]> {
  return invoke<HourlyStats[]>("get_stats_daily", { date: date ?? null });
}

export async function getStatsWeekly(): Promise<DailyStatsPoint[]> {
  return invoke<DailyStatsPoint[]>("get_stats_weekly");
}

export async function getStatsMonthly(): Promise<DailyStatsPoint[]> {
  return invoke<DailyStatsPoint[]>("get_stats_monthly");
}

export async function getLikeTypeRatio(period: StatsPeriod): Promise<LikeTypeRatio> {
  return invoke<LikeTypeRatio>("get_like_type_ratio", { period });
}

export async function getFriendRanking(period: StatsPeriod): Promise<FriendRanking[]> {
  return invoke<FriendRanking[]>("get_friend_ranking", { period });
}
```

**src/stores/useStatsStore.ts：**

```typescript
import { create } from "zustand";
import type { HourlyStats, DailyStatsPoint, LikeTypeRatio, FriendRanking, StatsPeriod } from "@/types/stats";
import { getStatsDaily, getStatsWeekly, getStatsMonthly, getLikeTypeRatio, getFriendRanking } from "@/lib/tauri";

interface StatsStore {
  // 状态
  hourlyData: HourlyStats[];
  weeklyData: DailyStatsPoint[];
  monthlyData: DailyStatsPoint[];
  typeRatio: LikeTypeRatio | null;
  friendRanking: FriendRanking[];
  currentPeriod: StatsPeriod;
  isLoading: boolean;

  // 操作
  fetchDailyStats: (date?: string) => Promise<void>;
  fetchWeeklyStats: () => Promise<void>;
  fetchMonthlyStats: () => Promise<void>;
  fetchLikeTypeRatio: (period?: StatsPeriod) => Promise<void>;
  fetchFriendRanking: (period?: StatsPeriod) => Promise<void>;
  setPeriod: (period: StatsPeriod) => Promise<void>;
}

export const useStatsStore = create<StatsStore>((set, get) => ({
  hourlyData: [],
  weeklyData: [],
  monthlyData: [],
  typeRatio: null,
  friendRanking: [],
  currentPeriod: "week",
  isLoading: false,

  fetchDailyStats: async (date) => {
    try {
      set({ isLoading: true });
      const data = await getStatsDaily(date);
      set({ hourlyData: data });
    } catch {
      // 静默处理，UI 层 toast
    } finally {
      set({ isLoading: false });
    }
  },

  fetchWeeklyStats: async () => {
    try {
      set({ isLoading: true });
      const data = await getStatsWeekly();
      set({ weeklyData: data });
    } catch {
      // 静默处理
    } finally {
      set({ isLoading: false });
    }
  },

  fetchMonthlyStats: async () => {
    try {
      set({ isLoading: true });
      const data = await getStatsMonthly();
      set({ monthlyData: data });
    } catch {
      // 静默处理
    } finally {
      set({ isLoading: false });
    }
  },

  fetchLikeTypeRatio: async (period) => {
    try {
      const p = period ?? get().currentPeriod;
      const data = await getLikeTypeRatio(p);
      set({ typeRatio: data });
    } catch {
      // 静默处理
    }
  },

  fetchFriendRanking: async (period) => {
    try {
      const p = period ?? get().currentPeriod;
      const data = await getFriendRanking(p);
      set({ friendRanking: data });
    } catch {
      // 静默处理
    }
  },

  setPeriod: async (period) => {
    set({ currentPeriod: period });
    const store = get();
    // 根据 period 刷新对应数据
    if (period === "day") {
      await store.fetchDailyStats();
    } else if (period === "week") {
      await store.fetchWeeklyStats();
    } else {
      await store.fetchMonthlyStats();
    }
    // 同时刷新类型占比和排行
    await Promise.all([
      store.fetchLikeTypeRatio(period),
      store.fetchFriendRanking(period),
    ]);
  },
}));
```

### 索引使用分析 (AC: #9)

所有查询均设计为命中现有索引：

| 查询 | 使用索引 | 说明 |
|------|---------|------|
| 每小时统计 | `idx_like_history_created_at` | WHERE created_at 范围过滤 + GROUP BY |
| 日期范围统计 | `idx_like_history_created_at` | WHERE created_at 范围过滤 |
| 类型占比 | `idx_like_history_created_at` | WHERE created_at 范围过滤 + CASE 聚合 |
| 好友排行 | `idx_like_history_user_id` + `idx_like_history_created_at` | JOIN friends + WHERE + GROUP BY user_id |
| 数据清理 | `idx_like_history_created_at` | WHERE created_at < cutoff DELETE |

### Story 6.3 QA 延续问题

- **P3-F1** `friends/strategy.rs:43` `.unwrap()` — 不影响本 Story
- **P4-I1** `AppError::NapCat` 滥用 — 本 Story 不新增变体，统计查询错误走 `AppError::Database`

### 不要做的事情

- **不要修改 `src/pages/Statistics.tsx`** — 图表可视化是 Story 7.2 的内容
- **不要新建数据库 migration** — 所有需要的表和索引已存在
- **不要修改 `engine/` 下任何文件** — 统计查询是只读操作，不影响引擎
- **不要修改 `db/models.rs`** — 聚合查询放 `stats/queries.rs`，不在 models 层
- **不要添加 recharts 相关代码** — 图表库使用在 Story 7.2
- **不要修改 Dashboard.tsx** — 仪表盘不受统计功能影响
- **不要在 useStatsStore 中添加 Tauri event 监听** — 统计数据不需要实时推送，按需拉取即可
- **不要修改 `webhook/mod.rs`** — 不相关
- **不要修改 `tray/mod.rs`** — 不相关
- **不要修改 `friends/` 模块** — 好友排行通过 LEFT JOIN 查询，不依赖 friends 模块函数
- **不要创建 `src/components/ChartPanel.tsx`** — 图表组件在 Story 7.2

### Project Structure Notes

新增文件：
```
src-tauri/src/
└── stats/
    └── queries.rs               # NEW — 聚合查询函数（5 个查询 + 1 个清理）

src-tauri/src/commands/
└── stats.rs                     # NEW — 5 个 Tauri commands + 日期范围辅助

src/stores/
└── useStatsStore.ts             # NEW — Zustand 统计数据 store
```

修改文件：
```
src-tauri/src/stats/mod.rs       # MODIFY — 添加 pub mod queries
src-tauri/src/commands/mod.rs    # MODIFY — 添加 pub mod stats
src-tauri/src/lib.rs             # MODIFY — 添加 mod stats + invoke_handler 注册 + 启动清理
src/types/stats.ts               # MODIFY — 新增 HourlyStats、DailyStatsPoint、LikeTypeRatio、FriendRanking、StatsPeriod
src/lib/tauri.ts                 # MODIFY — 新增 5 个 stats invoke wrappers
```

**Cargo.toml 可能修改：**
```
src-tauri/Cargo.toml             # MAYBE — 添加 chrono 直接依赖（若尚未声明）
```

**路径与架构对齐验证：**
- `stats/queries.rs` — architecture.md 明确指定 `stats/queries.rs # 聚合查询` ✅
- `commands/stats.rs` — architecture.md 指定 `commands/stats.rs # 统计查询: daily, weekly, monthly` ✅
- `useStatsStore.ts` — architecture.md 指定 `stores/useStatsStore.ts # 统计数据状态` ✅
- Tauri commands 返回 `Result<T, String>` ✅
- 所有结构体 `serde(rename_all = "camelCase")` ✅
- DB 查询利用现有索引 ✅
- 错误走 `AppError::Database` variant ✅
- 不污染其他模块 ✅

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story 7.1: 统计数据聚合查询]
- [Source: .bmad-method/planning-artifacts/epics.md#Epic 7: 数据统计与可视化 — FR47, FR48, FR49, FR50]
- [Source: .bmad-method/planning-artifacts/architecture.md#项目结构 — stats/queries.rs # 聚合查询, commands/stats.rs # 统计查询]
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则 — commands/ 唯一前端入口]
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范 — Rust snake_case, serde camelCase]
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri IPC 命令模式 — invoke + Result<T, String>]
- [Source: .bmad-method/planning-artifacts/architecture.md#Zustand Store 模式 — 每个域一个 store]
- [Source: .bmad-method/planning-artifacts/architecture.md#强制规则 7 条]
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单]
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#统计页面 — 渐变色组合、图表配色]
- [Source: src-tauri/migrations/002_quota_and_history.sql — like_history 表结构 + 3 个索引]
- [Source: src-tauri/migrations/001_init.sql — daily_state 表结构]
- [Source: src-tauri/migrations/003_friends.sql — friends 表结构]
- [Source: src-tauri/src/db/models.rs:158-168 — LikeHistory 结构体]
- [Source: src-tauri/src/db/models.rs:61-72 — DailyState 结构体]
- [Source: src-tauri/src/db/models.rs:217-230 — has_liked_today 日期范围查询模式]
- [Source: src-tauri/src/db/migrations.rs — 最后 migration 008]
- [Source: src-tauri/src/lib.rs:3-12 — 模块声明（无 stats）]
- [Source: src-tauri/src/lib.rs:255-283 — invoke_handler 注册位置]
- [Source: src-tauri/src/commands/mod.rs — 模块列表（无 stats）]
- [Source: src-tauri/src/stats/mod.rs — 空文件]
- [Source: src-tauri/src/errors.rs — AppError 枚举]
- [Source: src/types/stats.ts — 已有 QuotaStatus]
- [Source: src/lib/tauri.ts — 当前 invoke wrappers（无 stats）]
- [Source: src/stores/ — 已有 5 个 store（无 useStatsStore）]
- [Source: src/pages/Statistics.tsx — 占位页面]
- [Source: .bmad-method/implementation-artifacts/6-3-tag-based-like-strategy.md — 最近 Story 实现参考]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

无阻塞问题，一次性通过。

### Completion Notes List

- chrono 已在 Cargo.toml 中存在，无需添加新依赖
- Task 3.6（cleanup_history 手动命令）标记为可选，清理已通过启动时自动 spawn 实现，未暴露前端手动命令（不影响 AC）
- 所有 Rust 结构体均使用 `#[serde(rename_all = "camelCase")]`
- 所有 Tauri commands 返回 `Result<T, String>`
- DB lock 不跨 await — 同步获取数据后立即释放
- 所有查询利用现有 SQLite 索引
- 0 个 unwrap/expect 在新增生产代码中
- cargo check 通过（9 个 warning 均为既有代码）
- tsc --noEmit 通过
- eslint 通过

### File List

| 文件 | 操作 |
|------|------|
| src-tauri/src/stats/queries.rs | NEW — 聚合查询函数（6 个函数 + 4 个结构体） |
| src-tauri/src/stats/mod.rs | MODIFY — 添加 `pub mod queries;` |
| src-tauri/src/commands/stats.rs | NEW — 5 个 Tauri commands + date_range_for_period 辅助 |
| src-tauri/src/commands/mod.rs | MODIFY — 添加 `pub mod stats;` |
| src-tauri/src/lib.rs | MODIFY — 添加 `mod stats;` + invoke_handler 注册 5 命令 + 启动清理 spawn |
| src/types/stats.ts | MODIFY — 新增 HourlyStats、DailyStatsPoint、LikeTypeRatio、FriendRanking、StatsPeriod |
| src/lib/tauri.ts | MODIFY — 新增 5 个 stats invoke wrappers + 新 import |
| src/stores/useStatsStore.ts | NEW — Zustand 统计数据 store |

### Change Log

- 2026-03-14: Story 7.1 实现完成，所有 7 个 Task 及子任务全部完成
- 2026-03-14: QA Code Review — PASS

## QA Results

### Gate Decision: PASS

- **Reviewer:** Quinn (Test Architect)
- **Date:** 2026-03-14
- **Gate File:** `.bmad-method/test-artifacts/gates/7.1-stats-aggregation-queries.yml`
- **Confidence:** High

### AC Coverage: 10/10 (100%)

| AC | 描述 | 状态 |
|----|------|------|
| #1 | 聚合查询模块 stats/queries.rs | ✅ PASS |
| #2 | 日视图 24 小时分布 | ✅ PASS |
| #3 | 周视图近 7 天 | ✅ PASS |
| #4 | 月视图近 30 天 | ✅ PASS |
| #5 | 点赞类型占比 | ✅ PASS |
| #6 | 好友互动排行 TOP 10 | ✅ PASS |
| #7 | 数据保留策略 90 天 | ✅ PASS |
| #8 | 5 个 Tauri Commands | ✅ PASS |
| #9 | 索引优化 | ✅ PASS |
| #10 | useStatsStore (Zustand) | ✅ PASS |

### Architecture Compliance: 7/7 PASS

- serde(rename_all = "camelCase"): 4/4 structs ✅
- Result<T, String> return: 5/5 commands ✅
- tracing logging (no println): ✅
- No unwrap/expect in production code: ✅
- DB lock not held across await: ✅
- DbState injection via State<'_, DbState>: ✅
- Error via AppError::Database: ✅

### Findings Summary

| 级别 | 数量 | 阻塞 |
|------|------|------|
| P1 (Blocker) | 0 | — |
| P2 (Concern) | 0 | — |
| P3 (Advisory) | 2 | No |
| P4 (Info) | 2 | No |

### P3 Advisory Items

**P3-F1: Weekly/Monthly 日期缺失不补零**
- 位置: `stats/queries.rs:82-97`
- `get_hourly_stats` 正确为 0-23 小时补零，但 `get_daily_stats_range` 仅返回有数据的日期。7 天范围内若有 3 天无记录，仅返回 3 个数据点。
- 建议: Story 7.2 前端图表组件处理日期缺口，或在 Rust 侧补零。

**P3-F2: useStatsStore isLoading 竞态**
- 位置: `stores/useStatsStore.ts:40-67,86-95`
- `setPeriod()` 中多个 fetch 各自操作 isLoading，快速切换 period 时无取消机制。
- 建议: 使用请求计数器或 AbortController 模式。

### P4 Info Items

**P4-I1:** Store catch 块为空，未暴露 error 状态字段供 UI 消费 (`useStatsStore.ts`)
**P4-I2:** `date_range_for_period` 的 `_` 分支静默回退为 "week"，无 warning 日志 (`commands/stats.rs:11`)
