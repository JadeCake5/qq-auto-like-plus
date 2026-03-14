use tauri::{Manager, State};

use crate::db::DbState;
use crate::napcat::process::NapCatProcessState;
use crate::onebot::OneBotClientState;

#[tauri::command]
pub async fn download_napcat(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let zip_path = crate::napcat::downloader::download_napcat_zip(&app, &app_data_dir)
        .await
        .map_err(|e| e.to_string())?;
    crate::napcat::downloader::extract_napcat_zip(&app, &zip_path, &app_data_dir)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn import_napcat(app: tauri::AppHandle, zip_path: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    crate::napcat::downloader::import_napcat_zip(
        &app,
        std::path::Path::new(&zip_path),
        &app_data_dir,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_napcat_status(app: tauri::AppHandle) -> Result<crate::napcat::NapCatStatus, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(crate::napcat::check_napcat_status(&app_data_dir))
}

#[tauri::command]
pub async fn start_napcat(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    process_state: State<'_, NapCatProcessState>,
    onebot: State<'_, OneBotClientState>,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let napcat_dir = app_data_dir.join("napcat");
    let db = db.inner().clone();
    let process_state = process_state.inner().clone();
    let onebot = onebot.inner().clone();

    // 从 config 表读取 API 端口
    let api_port: u16 = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::models::get_config_by_key(&conn, "napcat_api_port")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(3000)
    };

    // 生成 NapCat 配置（阻塞 IO → spawn_blocking）
    let db_clone = db.clone();
    let app_data_dir_clone = app_data_dir.clone();
    tokio::task::spawn_blocking(move || {
        crate::napcat::config::generate_napcat_config(&app_data_dir_clone, &db_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // 启动进程 + 后台监控 + 健康检查
    crate::napcat::process::start_napcat_process(&app, &napcat_dir, api_port, &process_state, &db, &onebot)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn stop_napcat(process_state: State<'_, NapCatProcessState>) -> Result<(), String> {
    let mut state = process_state.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut process) = *state {
        process.stop().map_err(|e| e.to_string())?;
        *state = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_login_info_cmd(
    process_state: State<'_, NapCatProcessState>,
) -> Result<crate::napcat::LoginInfo, String> {
    let api_port = {
        let state = process_state.lock().map_err(|e| e.to_string())?;
        state
            .as_ref()
            .map(|p| p.api_port())
            .ok_or("NapCat 进程未运行".to_string())?
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/get_login_info", api_port))
        .json(&serde_json::json!({}))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("NapCat API 返回错误: {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(crate::napcat::LoginInfo {
        qq_number: body["data"]["user_id"].as_i64().unwrap_or(0).to_string(),
        nickname: body["data"]["nickname"]
            .as_str()
            .unwrap_or("")
            .to_string(),
    })
}

#[tauri::command]
pub async fn restart_napcat(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    process_state: State<'_, NapCatProcessState>,
    onebot: State<'_, OneBotClientState>,
) -> Result<(), String> {
    let db = db.inner().clone();
    let process_state = process_state.inner().clone();
    let onebot = onebot.inner().clone();

    crate::napcat::process::restart_napcat_cmd(app, process_state, db, onebot)
        .await
        .map_err(|e| e.to_string())
}
