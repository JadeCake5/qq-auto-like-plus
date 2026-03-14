use rusqlite::{params, Connection};
use serde::Serialize;
use crate::errors::AppError;

/// 每小时统计数据点
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyStats {
    pub hour: i32,
    pub count: i32,
}

/// 每日统计数据点
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStatsPoint {
    pub date: String,
    pub count: i32,
    pub scheduled: i32,
    pub reply: i32,
    pub manual: i32,
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
    pub total_likes: i32,
}

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

    let result: Vec<HourlyStats> = (0..24)
        .map(|h| HourlyStats {
            hour: h,
            count: *hour_map.get(&h).unwrap_or(&0),
        })
        .collect();

    Ok(result)
}

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
