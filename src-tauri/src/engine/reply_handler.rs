use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::db;
use crate::errors::AppError;
use crate::onebot;

pub type DbState = Arc<std::sync::Mutex<rusqlite::Connection>>;
pub type OneBotClientState = Arc<onebot::OneBotClient>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyLikeResult {
    pub operator_id: i64,
    pub times: i32,
    pub success: bool,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

pub async fn handle_reply_like(
    operator_id: i64,
    db: &DbState,
    onebot: &OneBotClientState,
    app: &AppHandle,
) -> Result<(), AppError> {
    // 检查 reply_enabled 开关
    let reply_enabled: bool = {
        let conn = db.lock().expect("lock db");
        db::models::get_config_by_key(&conn, "reply_enabled")
            .ok()
            .map(|c| c.value == "true")
            .unwrap_or(true)
    };
    if !reply_enabled {
        tracing::debug!("回赞已关闭，跳过 QQ {}", operator_id);
        let _ = app.emit("like:reply-complete", ReplyLikeResult {
            operator_id,
            times: 0,
            success: false,
            skipped: true,
            skip_reason: Some("回赞功能已关闭".to_string()),
        });
        return Ok(());
    }

    // 检查好友标签的回赞策略
    let (tag_auto_reply, tag_like_times) = {
        let conn = db.lock().expect("lock db");
        let tags = db::models::get_friend_tags(&conn, operator_id);
        match tags {
            Ok(tags) if !tags.is_empty() => {
                let strategy = crate::friends::strategy::resolve_friend_strategy(
                    &db::models::FriendWithTags {
                        user_id: operator_id,
                        nickname: String::new(),
                        remark: String::new(),
                        tags,
                        liked_today: false,
                    },
                    0, // placeholder, will use tag_like_times below
                );
                (strategy.auto_reply, strategy.like_times)
            }
            _ => (true, 0), // 无标签时默认允许回赞
        }
    };
    if !tag_auto_reply {
        tracing::debug!("标签设置不允许回赞，跳过 QQ {}", operator_id);
        let _ = app.emit("like:reply-complete", ReplyLikeResult {
            operator_id,
            times: 0,
            success: false,
            skipped: true,
            skip_reason: Some("标签设置不允许回赞".to_string()),
        });
        return Ok(());
    }

    // 检查今日是否已赞过该用户
    let already_liked = {
        let conn = db.lock().expect("lock db");
        super::quota::has_liked_today(&conn, operator_id)?
    };
    if already_liked {
        tracing::debug!("今日已赞过 QQ {}，跳过回赞", operator_id);
        let _ = app.emit("like:reply-complete", ReplyLikeResult {
            operator_id,
            times: 0,
            success: false,
            skipped: true,
            skip_reason: Some("今日已赞过该用户".to_string()),
        });
        return Ok(());
    }

    // 检查回赞名额
    let quota_result = {
        let conn = db.lock().expect("lock db");
        super::quota::ensure_today_state(&conn)?;
        super::quota::try_consume_quota(&conn, "reply")
    };
    if let Err(AppError::QuotaExhausted(msg)) = quota_result {
        tracing::warn!("回赞名额不足，跳过 QQ {}: {}", operator_id, msg);
        let _ = app.emit("like:reply-complete", ReplyLikeResult {
            operator_id,
            times: 0,
            success: false,
            skipped: true,
            skip_reason: Some("回赞名额已耗尽".to_string()),
        });
        return Ok(());
    }
    quota_result?;

    // 读取随机延迟配置并 sleep
    let (delay_min, delay_max) = {
        let conn = db.lock().expect("lock db");
        let min: u64 = db::models::get_config_by_key(&conn, "reply_delay_min")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(0);
        let max: u64 = db::models::get_config_by_key(&conn, "reply_delay_max")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(0);
        (min, max)
    };
    if delay_max > 0 {
        let delay = if delay_min >= delay_max {
            delay_min
        } else {
            use rand::Rng;
            rand::rng().random_range(delay_min..=delay_max)
        };
        if delay > 0 {
            tracing::debug!("回赞延迟 {} 秒: QQ {}", delay, operator_id);
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        }
    }

    // 读取回赞次数（标签覆盖优先）
    let reply_times: i32 = if tag_like_times > 0 {
        tag_like_times
    } else {
        let conn = db.lock().expect("lock db");
        db::models::get_config_by_key(&conn, "reply_times")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(10)
    };

    // 执行回赞
    match onebot.send_like(operator_id, reply_times).await {
        Ok(()) => {
            // 记录成功
            {
                let conn = db.lock().expect("lock db");
                super::quota::record_like(&conn, operator_id, reply_times, "reply", true, None)?;
            }
            let _ = app.emit("like:reply-complete", ReplyLikeResult {
                operator_id,
                times: reply_times,
                success: true,
                skipped: false,
                skip_reason: None,
            });
            tracing::info!("回赞成功: QQ {}，{} 次", operator_id, reply_times);
        }
        Err(e) => {
            // 记录失败
            let err_msg = e.to_string();
            {
                let conn = db.lock().expect("lock db");
                super::quota::record_like(
                    &conn,
                    operator_id,
                    reply_times,
                    "reply",
                    false,
                    Some(&err_msg),
                )?;
            }
            let _ = app.emit("like:reply-complete", ReplyLikeResult {
                operator_id,
                times: reply_times,
                success: false,
                skipped: false,
                skip_reason: None,
            });
            tracing::warn!("回赞失败: QQ {}, 错误: {}", operator_id, err_msg);
        }
    }

    Ok(())
}
