use std::path::Path;

use crate::db::DbState;
use crate::errors::AppError;

pub fn generate_napcat_config(app_data_dir: &Path, db: &DbState) -> Result<(), AppError> {
    let napcat_dir = app_data_dir.join("napcat");
    let config_dir = napcat_dir.join("config");
    std::fs::create_dir_all(&config_dir)?;

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

    let config = serde_json::json!({
        "http": {
            "enable": true,
            "host": "127.0.0.1",
            "port": api_port,
            "secret": "",
            "enableHeart": false,
            "enablePost": false
        },
        "httpServers": [
            {
                "name": "auto-like-webhook",
                "enable": true,
                "url": format!("http://127.0.0.1:{}/webhook", webhook_port),
                "secret": "",
                "reportSelfMessage": false
            }
        ],
        "ws": {
            "enable": false
        },
        "wsServers": [],
        "reverseWs": {
            "enable": false
        },
        "GroupLocalTime": {
            "Record": false,
            "RecordList": []
        },
        "debug": false,
        "heartInterval": 30000,
        "messagePostFormat": "array",
        "enableLocalFile2Url": false,
        "musicSignUrl": "",
        "reportSelfMessage": false,
        "token": ""
    });

    let config_path = config_dir.join("onebot11.json");
    let config_str =
        serde_json::to_string_pretty(&config).map_err(|e| AppError::NapCat(e.to_string()))?;
    std::fs::write(&config_path, config_str)?;

    tracing::info!("NapCat 配置已生成: {:?}", config_path);
    Ok(())
}
