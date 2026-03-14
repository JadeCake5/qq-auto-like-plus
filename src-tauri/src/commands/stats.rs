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
