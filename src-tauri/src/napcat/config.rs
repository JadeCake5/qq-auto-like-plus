use std::path::Path;

use crate::db::DbState;
use crate::errors::AppError;

/// 构建 NapCat v4+ 格式的 OneBot11 配置 JSON
fn build_onebot11_config(api_port: u16, webhook_port: u16) -> serde_json::Value {
    serde_json::json!({
        "network": {
            "httpServers": [
                {
                    "name": "auto-like-api",
                    "enable": true,
                    "host": "127.0.0.1",
                    "port": api_port,
                    "secret": "",
                    "enableCors": true,
                    "enableWebsocket": false
                }
            ],
            "httpClients": [
                {
                    "name": "auto-like-webhook",
                    "enable": true,
                    "url": format!("http://127.0.0.1:{}/webhook", webhook_port),
                    "messagePostFormat": "array",
                    "reportSelfMessage": false,
                    "secret": ""
                }
            ],
            "httpSseServers": [],
            "websocketServers": [],
            "websocketClients": [],
            "plugins": []
        },
        "musicSignUrl": "",
        "enableLocalFile2Url": false,
        "parseMultMsg": false
    })
}

pub fn generate_napcat_config(app_data_dir: &Path, db: &DbState) -> Result<(), AppError> {
    let napcat_dir = app_data_dir.join("napcat");
    // NapCat 实际配置目录在 napcat/napcat/config/
    let config_dir = napcat_dir.join("napcat").join("config");
    std::fs::create_dir_all(&config_dir)?;

    // 删除旧路径的配置文件（napcat/config/），防止旧格式干扰
    let old_config_dir = napcat_dir.join("config");
    if old_config_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&old_config_dir) {
            tracing::warn!("删除旧配置目录失败: {:?}: {}", old_config_dir, e);
        } else {
            tracing::info!("已删除旧配置目录: {:?}", old_config_dir);
        }
    }

    let api_port: u16 = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        crate::db::models::get_config_by_key(&conn, "napcat_api_port")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(3000)
    };

    let webhook_port: u16 = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        crate::db::models::get_config_by_key(&conn, "webhook_port")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(8080)
    };

    let config = build_onebot11_config(api_port, webhook_port);

    // 1. 写入默认配置
    let default_path = config_dir.join("onebot11.json");
    let config_str =
        serde_json::to_string_pretty(&config).map_err(|e| AppError::NapCat(e.to_string()))?;
    std::fs::write(&default_path, &config_str)?;
    tracing::info!("NapCat OneBot11 默认配置已生成: {:?}", default_path);

    // 2. 更新所有已有的 onebot11_{uin}.json 账号配置
    if let Ok(entries) = std::fs::read_dir(&config_dir) {
        let network = config.get("network").cloned();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("onebot11_") && name.ends_with(".json") {
                if let Err(e) = patch_account_onebot_config(&entry.path(), &network) {
                    tracing::warn!("更新账号配置 {} 失败: {}", name, e);
                }
            }
        }
    }

    // 3. 确保 NapCat 使用 Windows 原生协议（清理可能残留的协议覆盖）
    cleanup_protocol_overrides(&napcat_dir);

    Ok(())
}

/// 更新已有的 onebot11_{uin}.json：将 network 部分替换为我们的配置
fn patch_account_onebot_config(
    path: &Path,
    network: &Option<serde_json::Value>,
) -> Result<(), AppError> {
    let content = std::fs::read_to_string(path)?;
    let mut json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| AppError::NapCat(e.to_string()))?;

    if let Some(net) = network {
        json["network"] = net.clone();
        let patched =
            serde_json::to_string_pretty(&json).map_err(|e| AppError::NapCat(e.to_string()))?;
        std::fs::write(path, patched)?;
        tracing::info!("已更新账号配置: {:?}", path);
    }

    Ok(())
}

/// 清理可能残留的协议覆盖（Linux/MAC 协议注入），确保使用 Windows 原生协议。
fn cleanup_protocol_overrides(napcat_dir: &Path) {
    // 清理 index.js 中的 NAPCAT_LINUX_PROTOCOL 注入
    let index_path = napcat_dir.join("index.js");
    if index_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&index_path) {
            if content.contains("NAPCAT_LINUX_PROTOCOL") {
                let cleaned: String = content
                    .lines()
                    .filter(|line| !line.contains("NAPCAT_LINUX_PROTOCOL") && !line.contains("Object.defineProperty(process, 'platform'"))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Err(e) = std::fs::write(&index_path, &cleaned) {
                    tracing::warn!("清理 index.js 协议覆盖失败: {}", e);
                } else {
                    tracing::info!("已清理 index.js 中的协议覆盖代码");
                }
            }
        }
    }

    // 确保 qqnt.json platform = "win32"
    let qqnt_path = napcat_dir.join("napcat").join("qqnt.json");
    if qqnt_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&qqnt_path) {
            if let Ok(mut qqnt) = serde_json::from_str::<serde_json::Value>(&content) {
                let platform = qqnt.get("platform").and_then(|v| v.as_str()).unwrap_or("");
                if platform != "win32" {
                    qqnt["platform"] = serde_json::Value::String("win32".to_string());
                    if let Ok(patched) = serde_json::to_string_pretty(&qqnt) {
                        let _ = std::fs::write(&qqnt_path, patched);
                        tracing::info!("已将 qqnt.json platform 恢复为 \"win32\"");
                    }
                }
            }
        }
    }

    // 确保 config.json 使用 Windows 版本号
    let config_path = napcat_dir.join("config.json");
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) {
                let cur_ver = config.get("curVersion").and_then(|v| v.as_str()).unwrap_or("");
                // Linux 版本号以 3.x 开头，Windows 以 9.x 开头
                if cur_ver.starts_with("3.") {
                    // 从 qqnt.json 读取正确的 Windows 版本号
                    let win_version = if qqnt_path.exists() {
                        std::fs::read_to_string(&qqnt_path)
                            .ok()
                            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                            .and_then(|j| j.get("version").and_then(|v| v.as_str()).map(String::from))
                            .unwrap_or_else(|| "9.9.26-44343".to_string())
                    } else {
                        "9.9.26-44343".to_string()
                    };
                    let build_id = win_version.split('-').last().unwrap_or("44343").to_string();
                    config["baseVersion"] = serde_json::Value::String(win_version.clone());
                    config["curVersion"] = serde_json::Value::String(win_version.clone());
                    config["buildId"] = serde_json::Value::String(build_id);
                    if let Ok(patched) = serde_json::to_string_pretty(&config) {
                        let _ = std::fs::write(&config_path, patched);
                        tracing::info!("已将 config.json 版本号恢复为 Windows 版本: {}", win_version);
                    }
                }
            }
        }
    }
}
