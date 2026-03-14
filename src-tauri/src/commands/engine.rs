use tauri::State;

use crate::commands::like::BatchLikeRunning;
use crate::db::DbState;
use crate::engine::scheduler::{EngineStatus, LikeSchedulerState};
use crate::onebot::OneBotClientState;

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
