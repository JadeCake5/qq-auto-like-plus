use tauri::{Emitter, Manager, State};
use tauri_plugin_shell::ShellExt;

use crate::db::DbState;
use crate::napcat::process::NapCatProcessState;
use crate::napcat::NapCatStatus;
use crate::onebot::OneBotClientState;

#[tauri::command]
pub async fn download_napcat(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Downloading);
    let zip_path = crate::napcat::downloader::download_napcat_zip(&app, &app_data_dir)
        .await
        .map_err(|e| {
            let _ = app.emit("napcat:status-changed", &NapCatStatus::Error(e.to_string()));
            e.to_string()
        })?;

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Extracting);
    crate::napcat::downloader::extract_napcat_zip(&app, &zip_path, &app_data_dir)
        .map_err(|e| {
            let _ = app.emit("napcat:status-changed", &NapCatStatus::Error(e.to_string()));
            e.to_string()
        })?;

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Ready);
    Ok(())
}

#[tauri::command]
pub async fn import_napcat(app: tauri::AppHandle, zip_path: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Extracting);
    crate::napcat::downloader::import_napcat_zip(
        &app,
        std::path::Path::new(&zip_path),
        &app_data_dir,
    )
    .map_err(|e| {
        let _ = app.emit("napcat:status-changed", &NapCatStatus::Error(e.to_string()));
        e.to_string()
    })?;

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Ready);
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

/// 清理 NapCat 缓存/会话数据（解决登录后 Worker 崩溃等问题）
#[tauri::command]
pub fn clear_napcat_cache(
    app: tauri::AppHandle,
    process_state: State<'_, NapCatProcessState>,
) -> Result<String, String> {
    // 确保 NapCat 没在运行
    {
        let state = process_state.lock().map_err(|e| e.to_string())?;
        if state.is_some() {
            return Err("请先停止 NapCat 再清理缓存".to_string());
        }
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let napcat_dir = app_data_dir.join("napcat");
    let mut cleared = Vec::new();

    // 清理 NapCat 内部缓存
    let cache_dirs = [
        napcat_dir.join("napcat").join("cache"),
        napcat_dir.join("napcat").join("logs"),
    ];
    for dir in &cache_dirs {
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                tracing::warn!("清理目录失败 {:?}: {}", dir, e);
            } else {
                let _ = std::fs::create_dir_all(dir);
                cleared.push(dir.file_name().unwrap_or_default().to_string_lossy().to_string());
            }
        }
    }

    // 清理 quickLoginCache（快速登录缓存）
    let quick_login = napcat_dir.join("napcat").join("quickLoginCache");
    if quick_login.exists() {
        let _ = std::fs::remove_dir_all(&quick_login);
        cleared.push("quickLoginCache".to_string());
    }

    let msg = if cleared.is_empty() {
        "没有需要清理的缓存".to_string()
    } else {
        format!("已清理: {}", cleared.join(", "))
    };
    tracing::info!("NapCat 缓存清理: {}", msg);
    Ok(msg)
}

/// 重新下载 NapCat（更新到最新版），先停止运行中的进程
#[tauri::command]
pub async fn update_napcat(
    app: tauri::AppHandle,
    process_state: State<'_, NapCatProcessState>,
) -> Result<(), String> {
    // 先停止正在运行的 NapCat
    {
        let mut state = process_state.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut process) = *state {
            let _ = process.stop();
            *state = None;
        }
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let napcat_dir = app_data_dir.join("napcat");

    // 备份配置文件
    let config_backup = napcat_dir.join("napcat").join("config");
    let temp_config = app_data_dir.join("_napcat_config_backup");
    if config_backup.exists() {
        let _ = crate::napcat::copy_dir_recursive(&config_backup, &temp_config);
    }

    // 删除旧的 NapCat 目录
    if napcat_dir.exists() {
        std::fs::remove_dir_all(&napcat_dir).map_err(|e| format!("删除旧版本失败: {}", e))?;
    }

    // 重新下载
    let _ = app.emit("napcat:status-changed", &NapCatStatus::Downloading);
    let zip_path = crate::napcat::downloader::download_napcat_zip(&app, &app_data_dir)
        .await
        .map_err(|e| {
            let _ = app.emit("napcat:status-changed", &NapCatStatus::Error(e.to_string()));
            e.to_string()
        })?;

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Extracting);
    crate::napcat::downloader::extract_napcat_zip(&app, &zip_path, &app_data_dir)
        .map_err(|e| {
            let _ = app.emit("napcat:status-changed", &NapCatStatus::Error(e.to_string()));
            e.to_string()
        })?;

    // 恢复配置
    if temp_config.exists() {
        let restored = napcat_dir.join("napcat").join("config");
        let _ = std::fs::create_dir_all(&restored);
        let _ = crate::napcat::copy_dir_recursive(&temp_config, &restored);
        let _ = std::fs::remove_dir_all(&temp_config);
    }

    let _ = app.emit("napcat:status-changed", &NapCatStatus::Ready);
    tracing::info!("NapCat 更新完成");
    Ok(())
}

#[tauri::command]
pub fn open_napcat_dir(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let napcat_dir = app_data_dir.join("napcat");
    if !napcat_dir.exists() {
        return Err("NapCat 目录不存在".to_string());
    }
    app.shell()
        .open(napcat_dir.to_string_lossy().to_string(), None)
        .map_err(|e| e.to_string())
}
