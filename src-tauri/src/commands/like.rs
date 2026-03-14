use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Emitter, State};

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
