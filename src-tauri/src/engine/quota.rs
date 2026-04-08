use rusqlite::Connection;
use serde::Serialize;

use crate::db::models;
use crate::errors::AppError;

/// 获取当前日期字符串（YYYY-MM-DD）
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaStatus {
    pub date: String,
    pub daily_limit: i32,
    pub reserved_for_reply: i32,
    pub total_liked: i32,
    pub scheduled_count: i32,
    pub reply_count: i32,
    pub manual_count: i32,
    pub available_scheduled: i32,
    pub available_reply: i32,
}

/// 确保今日 daily_state 记录存在
pub fn ensure_today_state(conn: &Connection) -> Result<(), AppError> {
    let date = today();
    models::ensure_daily_state(conn, &date).map_err(AppError::Database)?;
    Ok(())
}

/// 获取名额状态（指定日期版本）
pub fn get_quota_status_for_date(conn: &Connection, date: &str) -> Result<QuotaStatus, AppError> {
    models::ensure_daily_state(conn, date).map_err(AppError::Database)?;

    let daily_limit: i32 = models::get_config_by_key(conn, "daily_limit")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(50)
        .max(10); // 强制最小值 10
    let raw_reserved: i32 = models::get_config_by_key(conn, "reserved_for_reply")
        .ok()
        .and_then(|c| c.value.parse().ok())
        .unwrap_or(10);
    // 确保 reserved_for_reply 不超过 daily_limit 的一半，防止定时名额为零
    let reserved_for_reply = raw_reserved.min(daily_limit / 2);

    let state = models::get_today_state(conn, date)?;

    let (total_liked, scheduled_count, reply_count, manual_count) = match state {
        Some(s) => (s.liked_count, s.scheduled_count, s.reply_count, s.manual_count),
        None => (0, 0, 0, 0),
    };

    let available_scheduled = (daily_limit - reserved_for_reply - scheduled_count - manual_count).max(0);
    let available_reply = (reserved_for_reply - reply_count).max(0);

    Ok(QuotaStatus {
        date: date.to_string(),
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

/// 获取名额状态（便利接口，自动使用今天日期）
pub fn get_quota_status(conn: &Connection) -> Result<QuotaStatus, AppError> {
    get_quota_status_for_date(conn, &today())
}

/// 尝试消耗名额（指定日期版本）
pub fn try_consume_quota_for_date(conn: &Connection, like_type: &str, date: &str) -> Result<(), AppError> {
    let status = get_quota_status_for_date(conn, date)?;

    match like_type {
        "scheduled" | "manual" => {
            if status.available_scheduled <= 0 {
                tracing::warn!(
                    "名额耗尽: 类型={}, 已用={}/{}, 日期={}",
                    like_type,
                    status.scheduled_count + status.manual_count,
                    status.daily_limit - status.reserved_for_reply,
                    date
                );
                return Err(AppError::QuotaExhausted(format!(
                    "定时/手动名额已耗尽（已用 {}/{}）",
                    status.scheduled_count + status.manual_count,
                    status.daily_limit - status.reserved_for_reply
                )));
            }
        }
        "reply" => {
            if status.available_reply <= 0 {
                tracing::warn!(
                    "回赞名额耗尽: 已用={}/{}, 日期={}",
                    status.reply_count,
                    status.reserved_for_reply,
                    date
                );
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

    tracing::debug!("名额消耗: 类型={}, 日期={}, 剩余scheduled={}, 剩余reply={}",
        like_type, date, status.available_scheduled, status.available_reply);

    models::increment_daily_count(conn, date, like_type)
        .map_err(AppError::Database)?;

    Ok(())
}

/// 尝试消耗名额（便利接口，自动使用今天日期）
pub fn try_consume_quota(conn: &Connection, like_type: &str) -> Result<(), AppError> {
    try_consume_quota_for_date(conn, like_type, &today())
}

/// 记录点赞历史
pub fn record_like(
    conn: &Connection,
    user_id: i64,
    times: i32,
    like_type: &str,
    success: bool,
    error_msg: Option<&str>,
) -> Result<(), AppError> {
    models::insert_like_history(conn, user_id, times, like_type, success, error_msg)
        .map_err(AppError::Database)?;
    Ok(())
}

/// 检查用户今日是否已被赞过（指定日期版本）
pub fn has_liked_today_for_date(conn: &Connection, user_id: i64, date: &str) -> Result<bool, AppError> {
    models::has_liked_today(conn, user_id, date)
        .map_err(AppError::Database)
}

/// 检查用户今日是否已被赞过（便利接口）
pub fn has_liked_today(conn: &Connection, user_id: i64) -> Result<bool, AppError> {
    has_liked_today_for_date(conn, user_id, &today())
}
