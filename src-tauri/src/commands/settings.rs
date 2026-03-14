use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::{Emitter, State};
use tauri_plugin_autostart::ManagerExt;

use crate::commands::like::BatchLikeRunning;
use crate::db::models;
use crate::engine::scheduler::LikeSchedulerState;
use crate::onebot::OneBotClientState;

#[tauri::command]
pub fn get_config(
    db: State<'_, Arc<Mutex<Connection>>>,
) -> Result<Vec<models::ConfigEntry>, String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    models::get_all_config(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_config(
    app: tauri::AppHandle,
    db: State<'_, Arc<Mutex<Connection>>>,
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

#[tauri::command]
pub fn enable_autostart(
    app: tauri::AppHandle,
    db: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    app.autolaunch().enable().map_err(|e| e.to_string())?;
    let conn = db.lock().map_err(|e| e.to_string())?;
    models::upsert_config(&conn, "auto_start", "true").map_err(|e| e.to_string())?;
    tracing::info!("开机自启已启用");
    Ok(())
}

#[tauri::command]
pub fn disable_autostart(
    app: tauri::AppHandle,
    db: State<'_, Arc<Mutex<Connection>>>,
) -> Result<(), String> {
    app.autolaunch().disable().map_err(|e| e.to_string())?;
    let conn = db.lock().map_err(|e| e.to_string())?;
    models::upsert_config(&conn, "auto_start", "false").map_err(|e| e.to_string())?;
    tracing::info!("开机自启已禁用");
    Ok(())
}

#[tauri::command]
pub fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}
