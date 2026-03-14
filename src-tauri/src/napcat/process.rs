use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::db::DbState;
use crate::errors::AppError;
use crate::onebot::OneBotClientState;

pub struct NapCatProcess {
    child: Child,
    api_port: u16,
}

impl NapCatProcess {
    pub fn new(child: Child, api_port: u16) -> Self {
        Self { child, api_port }
    }

    pub fn api_port(&self) -> u16 {
        self.api_port
    }

    pub fn stop(&mut self) -> Result<(), AppError> {
        self.child
            .kill()
            .map_err(|e| AppError::NapCat(format!("停止 NapCat 失败: {}", e)))?;
        let _ = self.child.wait();
        Ok(())
    }

    /// 检查子进程是否仍在运行（非阻塞）
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(status)) => {
                tracing::warn!("NapCat 进程已退出: {:?}", status);
                false
            }
            Err(e) => {
                tracing::error!("检查 NapCat 进程状态失败: {}", e);
                false
            }
        }
    }
}

pub type NapCatProcessState = Arc<Mutex<Option<NapCatProcess>>>;

/// 查找 NapCat 可执行入口。优先直接使用 node.exe 以确保 kill() 能正确终止进程。
fn find_napcat_executable(napcat_dir: &Path) -> Result<(PathBuf, Vec<String>), AppError> {
    // 优先：直接 node + napcat.mjs（kill 时可直接终止 node 进程）
    let node_path = napcat_dir.join("node").join("node.exe");
    if node_path.exists() {
        let script_candidates = [
            napcat_dir.join("napcat").join("napcat.mjs"),
            napcat_dir.join("napcat.mjs"),
        ];
        for script in &script_candidates {
            if script.exists() {
                return Ok((
                    node_path,
                    vec![script.to_string_lossy().to_string()],
                ));
            }
        }
    }

    // 回退：bat 启动脚本
    let bat_candidates = ["NapCat.Shell.bat", "launcher.bat", "napcat.bat"];
    for bat in &bat_candidates {
        let bat_path = napcat_dir.join(bat);
        if bat_path.exists() {
            return Ok((
                PathBuf::from("cmd"),
                vec!["/C".to_string(), bat_path.to_string_lossy().to_string()],
            ));
        }
    }

    Err(AppError::NapCat("未找到 NapCat 启动脚本".to_string()))
}

pub fn start_napcat_process(
    app_handle: &tauri::AppHandle,
    napcat_dir: &Path,
    api_port: u16,
    process_state: &NapCatProcessState,
    db: &DbState,
    onebot: &OneBotClientState,
) -> Result<(), AppError> {
    // 检查是否已在运行
    {
        let state = process_state
            .lock()
            .map_err(|e| AppError::NapCat(e.to_string()))?;
        if state.is_some() {
            return Err(AppError::NapCat("NapCat 进程已在运行".to_string()));
        }
    }

    let (program, args) = find_napcat_executable(napcat_dir)?;
    tracing::info!("启动 NapCat: {:?} {:?}", program, args);

    let mut child = Command::new(&program)
        .args(&args)
        .current_dir(napcat_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::NapCat(format!("启动 NapCat 失败: {}", e)))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // 保存进程到状态
    {
        let mut state = process_state
            .lock()
            .map_err(|e| AppError::NapCat(e.to_string()))?;
        *state = Some(NapCatProcess::new(child, api_port));
    }

    if let Err(e) = app_handle.emit("napcat:status-changed", &super::NapCatStatus::Starting) {
        tracing::warn!("emit napcat:status-changed 失败: {}", e);
    }

    // 后台读取 stdout，检测二维码输出
    if let Some(stdout) = stdout {
        let app_clone = app_handle.clone();
        tokio::task::spawn_blocking(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                tracing::debug!("NapCat stdout: {}", line);
                if line.contains("qrcode") || line.contains("QR") || line.contains("二维码") {
                    tracing::info!("检测到二维码输出: {}", line);
                    if let Err(e) = app_clone.emit("napcat:qr-code", line.trim()) {
                        tracing::warn!("emit napcat:qr-code 失败: {}", e);
                    }
                }
            }
            tracing::info!("NapCat stdout 流结束");
        });
    }

    // 后台读取 stderr
    if let Some(stderr) = stderr {
        let app_clone = app_handle.clone();
        tokio::task::spawn_blocking(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                tracing::warn!("NapCat stderr: {}", line);
                if line.contains("qrcode") || line.contains("QR") || line.contains("二维码") {
                    if let Err(e) = app_clone.emit("napcat:qr-code", line.trim()) {
                        tracing::warn!("emit napcat:qr-code 失败: {}", e);
                    }
                }
            }
            tracing::info!("NapCat stderr 流结束");
        });
    }

    // 备选：文件监控检测二维码
    let app_clone = app_handle.clone();
    let napcat_dir_clone = napcat_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        monitor_qr_file(&app_clone, &napcat_dir_clone);
    });

    // 后台登录轮询 → 成功后启动健康检查
    let app_clone = app_handle.clone();
    let db_clone = db.clone();
    let process_state_clone = process_state.clone();
    let onebot_clone = onebot.clone();
    tokio::spawn(async move {
        match poll_login_status(&app_clone, api_port, &db_clone).await {
            Ok(info) => {
                tracing::info!("登录成功: {} ({})，启动健康检查", info.nickname, info.qq_number);
                start_health_check(app_clone, process_state_clone, db_clone, onebot_clone).await;
            }
            Err(e) => {
                tracing::error!("登录轮询失败: {}", e);
                if let Err(emit_err) = app_clone.emit(
                    "napcat:status-changed",
                    &super::NapCatStatus::Error(e.to_string()),
                ) {
                    tracing::warn!("emit napcat:status-changed 失败: {}", emit_err);
                }
            }
        }
    });

    Ok(())
}

fn monitor_qr_file(app_handle: &tauri::AppHandle, napcat_dir: &Path) {
    let possible_dirs = [
        napcat_dir.join("data"),
        napcat_dir.join("napcat"),
        napcat_dir.to_path_buf(),
    ];

    for _ in 0..60 {
        std::thread::sleep(Duration::from_millis(500));
        for dir in &possible_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "png") {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        if name.contains("qr") || name.contains("qrcode") {
                            tracing::info!("检测到二维码文件: {:?}", path);
                            if let Err(e) =
                                app_handle.emit("napcat:qr-code", path.to_string_lossy().as_ref())
                            {
                                tracing::warn!("emit napcat:qr-code 失败: {}", e);
                            }
                            return;
                        }
                    }
                }
            }
        }
    }
    tracing::debug!("30 秒内未检测到二维码文件");
}

pub async fn poll_login_status(
    app_handle: &tauri::AppHandle,
    api_port: u16,
    db: &DbState,
) -> Result<super::LoginInfo, AppError> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/get_login_info", api_port);
    let timeout = tokio::time::Instant::now() + Duration::from_secs(300);

    if let Err(e) =
        app_handle.emit("napcat:status-changed", &super::NapCatStatus::WaitingForLogin)
    {
        tracing::warn!("emit napcat:status-changed 失败: {}", e);
    }

    loop {
        if tokio::time::Instant::now() > timeout {
            return Err(AppError::NapCat("登录轮询超时（5 分钟）".to_string()));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        match client
            .post(&url)
            .json(&serde_json::json!({}))
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::debug!("NapCat API 返回非成功状态: {}", resp.status());
                    continue;
                }

                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if body["status"] == "ok" {
                        let user_id = body["data"]["user_id"].as_i64().unwrap_or(0);
                        let nickname =
                            body["data"]["nickname"].as_str().unwrap_or("").to_string();

                        if user_id > 0 {
                            {
                                let conn =
                                    db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
                                crate::db::models::upsert_config(
                                    &conn,
                                    "qq_number",
                                    &user_id.to_string(),
                                )?;
                                crate::db::models::upsert_config(
                                    &conn,
                                    "qq_nickname",
                                    &nickname,
                                )?;
                            }

                            let login_info = super::LoginInfo {
                                qq_number: user_id.to_string(),
                                nickname: nickname.clone(),
                            };

                            if let Err(e) =
                                app_handle.emit("napcat:login-success", &login_info)
                            {
                                tracing::warn!("emit napcat:login-success 失败: {}", e);
                            }
                            if let Err(e) = app_handle
                                .emit("napcat:status-changed", &super::NapCatStatus::Running)
                            {
                                tracing::warn!("emit napcat:status-changed 失败: {}", e);
                            }

                            tracing::info!("QQ 登录成功: {} ({})", nickname, user_id);
                            return Ok(login_info);
                        }
                    }
                }
            }
            Err(_) => {
                tracing::debug!("NapCat API 未就绪，继续等待...");
            }
        }
    }
}

/// 健康检查循环：每 30 秒检查 NapCat 连通性和登录状态
pub async fn start_health_check(
    app: tauri::AppHandle,
    process_state: NapCatProcessState,
    db: DbState,
    onebot: OneBotClientState,
) {
    let mut restart_count: u32 = 0;
    let restart_delays = [5u64, 15, 30];
    let mut offline_notified = false;
    let mut alive_but_unresponsive_count: u32 = 0;
    const MAX_UNRESPONSIVE: u32 = 5; // 5 次 × 30 秒 = 2.5 分钟无响应则视为僵死

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        match onebot.get_login_info().await {
            Ok(info) => {
                restart_count = 0;
                alive_but_unresponsive_count = 0;

                if info.user_id == 0 {
                    // QQ 掉线
                    if !offline_notified {
                        tracing::warn!("检测到 QQ 掉线（user_id=0）");
                        let _ = app.emit("napcat:status-changed", &super::NapCatStatus::WaitingForLogin);
                        let _ = app.emit("napcat:login-required", ());

                        app.notification()
                            .builder()
                            .title("QQ Auto Like Plus")
                            .body("QQ 需要重新登录，请打开面板扫码~")
                            .show()
                            .map_err(|e| tracing::error!("发送通知失败: {}", e))
                            .ok();

                        offline_notified = true;
                    }
                    // 继续循环，等待重新登录
                } else {
                    // user_id > 0，检查 QQ 号是否与已保存的一致
                    let saved_qq = {
                        let conn = match db.lock() {
                            Ok(g) => g,
                            Err(e) => e.into_inner(),
                        };
                        crate::db::models::get_config_by_key(&conn, "qq_number")
                            .ok()
                            .map(|c| c.value)
                    };

                    let current_qq = info.user_id.to_string();
                    if let Some(ref saved) = saved_qq {
                        if !saved.is_empty() && *saved != current_qq {
                            // QQ 号变更（可能被顶号）
                            tracing::warn!(
                                "检测到 QQ 号变更: 已保存={}, 当前={}，可能被顶号",
                                saved, current_qq
                            );
                            if !offline_notified {
                                let _ = app.emit("napcat:status-changed", &super::NapCatStatus::WaitingForLogin);
                                let _ = app.emit("napcat:login-required", ());

                                app.notification()
                                    .builder()
                                    .title("QQ Auto Like Plus")
                                    .body("QQ 需要重新登录，请打开面板扫码~")
                                    .show()
                                    .map_err(|e| tracing::error!("发送通知失败: {}", e))
                                    .ok();

                                offline_notified = true;
                            }
                            continue;
                        }
                    }

                    // 在线正常
                    if offline_notified {
                        tracing::info!("QQ 重新登录成功: user_id={}", info.user_id);
                        let _ = app.emit("napcat:status-changed", &super::NapCatStatus::Running);
                        offline_notified = false;
                    }

                    // 保存/更新 QQ 号
                    {
                        let conn = match db.lock() {
                            Ok(g) => g,
                            Err(e) => e.into_inner(),
                        };
                        let _ = crate::db::models::upsert_config(&conn, "qq_number", &current_qq);
                        let _ = crate::db::models::upsert_config(&conn, "qq_nickname", &info.nickname);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("健康检查 get_login_info 失败: {}", e);

                // 检查进程是否存活
                let alive = {
                    let mut guard = match process_state.lock() {
                        Ok(g) => g,
                        Err(e) => e.into_inner(),
                    };
                    guard.as_mut().map_or(false, |p| p.is_alive())
                };

                if alive {
                    alive_but_unresponsive_count += 1;
                    if alive_but_unresponsive_count >= MAX_UNRESPONSIVE {
                        tracing::warn!(
                            "NapCat 进程存活但连续 {} 次 API 无响应，视为僵死，强制重启",
                            alive_but_unresponsive_count
                        );
                        alive_but_unresponsive_count = 0;
                        // 走重启流程（fall through to else branch）
                    } else {
                        tracing::warn!(
                            "NapCat API 不通但进程存活（{}/{}），可能正在启动中",
                            alive_but_unresponsive_count, MAX_UNRESPONSIVE
                        );
                        continue;
                    }
                }

                // 进程已退出或僵死 → 自动重启
                {
                    if restart_count < 3 {
                        let delay = restart_delays[restart_count as usize];
                        tracing::warn!(
                            "NapCat 进程异常退出，{}秒后第{}次重启",
                            delay,
                            restart_count + 1
                        );

                        // 清理旧状态
                        {
                            let mut guard = match process_state.lock() {
                                Ok(g) => g,
                                Err(e) => e.into_inner(),
                            };
                            *guard = None;
                        }

                        let _ = app.emit("napcat:status-changed", &super::NapCatStatus::Starting);
                        tokio::time::sleep(Duration::from_secs(delay)).await;

                        match restart_napcat_internal(&app, &process_state, &db, &onebot).await {
                            Ok(()) => {
                                tracing::info!("NapCat 重启成功（第{}次），当前健康检查循环退出，由新 spawn 接管", restart_count + 1);
                                // break 退出当前循环，start_napcat_process 会 spawn 新的 poll_login → health_check
                                break;
                            }
                            Err(e) => {
                                tracing::error!("NapCat 重启失败（第{}次）: {}", restart_count + 1, e);
                                restart_count += 1;
                            }
                        }
                    } else {
                        // 超过 3 次重启上限
                        tracing::error!("NapCat 重启失败，已超过最大重试次数（3次）");
                        let _ = app.emit(
                            "napcat:status-changed",
                            &super::NapCatStatus::Error("重启失败，已超过最大重试次数".to_string()),
                        );

                        app.notification()
                            .builder()
                            .title("QQ Auto Like Plus")
                            .body("小助手遇到了一点问题，需要你帮帮忙~")
                            .show()
                            .map_err(|e| tracing::error!("发送通知失败: {}", e))
                            .ok();

                        break;
                    }
                }
            }
        }
    }
    tracing::info!("健康检查循环已退出");
}

/// 内部重启 NapCat 进程
async fn restart_napcat_internal(
    app: &tauri::AppHandle,
    process_state: &NapCatProcessState,
    db: &DbState,
    onebot: &OneBotClientState,
) -> Result<(), AppError> {
    // 清理旧进程状态
    {
        let mut guard = process_state.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        if let Some(ref mut p) = *guard {
            let _ = p.stop();
        }
        *guard = None;
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::NapCat(e.to_string()))?;
    let napcat_dir = app_data_dir.join("napcat");
    let api_port: u16 = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        crate::db::models::get_config_by_key(&conn, "napcat_api_port")
            .ok()
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(3000)
    };

    // 重新生成配置
    let db_clone = db.clone();
    let app_data_dir_clone = app_data_dir.clone();
    tokio::task::spawn_blocking(move || {
        crate::napcat::config::generate_napcat_config(&app_data_dir_clone, &db_clone)
    })
    .await
    .map_err(|e| AppError::NapCat(e.to_string()))?
    .map_err(|e| AppError::NapCat(e.to_string()))?;

    // 重新启动
    start_napcat_process(app, &napcat_dir, api_port, process_state, db, onebot)?;

    Ok(())
}

/// Tauri command: 手动重启 NapCat
pub async fn restart_napcat_cmd(
    app: tauri::AppHandle,
    process_state: NapCatProcessState,
    db: DbState,
    onebot: OneBotClientState,
) -> Result<(), AppError> {
    restart_napcat_internal(&app, &process_state, &db, &onebot).await
}
