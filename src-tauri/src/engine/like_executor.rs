use std::sync::Arc;

use serde::Serialize;
use tauri::Emitter;

use crate::db::DbState;
use crate::db::models;
use crate::engine::quota;
use crate::errors::AppError;
use crate::friends::strategy;
use crate::onebot::OneBotClient;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchLikeProgress {
    pub current: i32,
    pub total: i32,
    pub user_id: i64,
    pub nickname: String,
    pub success: bool,
    pub skipped: bool,
    pub error_msg: Option<String>,
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
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(10);
        let bi: u64 = models::get_config_by_key(&conn, "batch_interval")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(5);
        (tpf, bi)
    };

    // 3. 检查 DB 缓存的好友数量；只有缓存为空才调用 OneBot API
    let cached_count: i64 = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        models::get_friend_count(&conn)?
    };

    if cached_count == 0 {
        tracing::info!("好友缓存为空，从 OneBot 拉取好友列表...");
        let friends = onebot.get_friend_list().await
            .map_err(AppError::OneBot)?;
        tracing::info!("获取到 {} 个好友，写入缓存", friends.len());

        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        let rows: Vec<models::FriendRow> = friends.iter().map(|f| models::FriendRow {
            user_id: f.user_id,
            nickname: f.nickname.clone(),
            remark: f.remark.clone(),
        }).collect();
        if let Err(e) = models::upsert_friends_batch(&conn, &rows) {
            tracing::warn!("缓存好友信息失败（不影响点赞）: {}", e);
        }
    } else {
        tracing::info!("批量点赞开始: 使用 DB 缓存的 {} 个好友（跳过 get_friend_list API）", cached_count);
    }

    // 5. 构建标签策略排序的点赞队列
    let like_queue = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        let friends_with_tags = models::get_all_friends_with_tags(&conn, &date)?;
        strategy::build_like_queue(friends_with_tags, times_per_friend)
    };

    let total = like_queue.len() as i32;
    let mut success_count = 0i32;
    let mut skipped_count = 0i32;
    let mut failed_count = 0i32;
    let mut consecutive_failures = 0i32;
    const MAX_CONSECUTIVE_FAILURES: i32 = 5;

    // 6. 逐个点赞（按标签优先级排序）
    for (i, strat) in like_queue.iter().enumerate() {
        let current = (i + 1) as i32;

        // 6a. 检查是否已赞
        let already_liked = {
            let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
            quota::has_liked_today_for_date(&conn, strat.user_id, &date)?
        };
        if already_liked {
            tracing::debug!("跳过已赞好友: {} ({})", strat.nickname, strat.user_id);
            skipped_count += 1;
            let _ = app.emit("like:progress", BatchLikeProgress {
                current,
                total,
                user_id: strat.user_id,
                nickname: strat.nickname.clone(),
                success: false,
                skipped: true,
                error_msg: None,
            });
            continue;
        }

        // 6b. 检查名额
        let quota_ok = {
            let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
            quota::try_consume_quota_for_date(&conn, like_type, &date)
        };
        if let Err(AppError::QuotaExhausted(msg)) = &quota_ok {
            tracing::info!("名额耗尽，停止批量点赞: {}", msg);
            let _ = app.emit("like:progress", BatchLikeProgress {
                current,
                total,
                user_id: strat.user_id,
                nickname: strat.nickname.clone(),
                success: false,
                skipped: true,
                error_msg: Some(msg.clone()),
            });
            break;
        }
        quota_ok.map_err(|e| {
            tracing::error!("名额检查异常: {}", e);
            e
        })?;

        // 6c. 调用 OneBot API 点赞（使用标签策略的 like_times）
        let like_result = onebot.send_like(strat.user_id, strat.like_times).await;

        // 6d. 记录结果
        let (success, error_msg) = match &like_result {
            Ok(()) => {
                tracing::info!(
                    "[{}/{}] 点赞成功: {} ({}) ×{}",
                    current, total, strat.nickname, strat.user_id, strat.like_times
                );
                success_count += 1;
                consecutive_failures = 0;
                (true, None)
            }
            Err(e) => {
                tracing::warn!(
                    "[{}/{}] 点赞失败: {} ({}) - {}",
                    current, total, strat.nickname, strat.user_id, e
                );
                failed_count += 1;
                consecutive_failures += 1;
                (false, Some(e.to_string()))
            }
        };

        {
            let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
            quota::record_like(
                &conn, strat.user_id, strat.like_times,
                like_type, success, error_msg.as_deref(),
            )?;
        }

        // 6e. 推送进度事件
        let _ = app.emit("like:progress", BatchLikeProgress {
            current,
            total,
            user_id: strat.user_id,
            nickname: strat.nickname.clone(),
            success,
            skipped: false,
            error_msg: error_msg.clone(),
        });

        // 6f. 连续失败过多则提前终止
        if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
            tracing::error!(
                "连续 {} 次点赞失败，提前终止批量点赞",
                consecutive_failures
            );
            let _ = app.emit("like:batch-error",
                format!("连续 {} 次点赞失败，已停止", consecutive_failures));
            break;
        }

        // 6g. 间隔等待（最后一个不等待）
        if i < like_queue.len() - 1 {
            tokio::time::sleep(std::time::Duration::from_secs(batch_interval)).await;
        }
    }

    // 7. 完成通知
    let result = BatchLikeResult {
        total,
        success_count,
        skipped_count,
        failed_count,
    };
    tracing::info!(
        "批量点赞完成: 总计{} 成功{} 跳过{} 失败{}",
        total, success_count, skipped_count, failed_count
    );
    let _ = app.emit("like:batch-complete", result.clone());

    Ok(result)
}
