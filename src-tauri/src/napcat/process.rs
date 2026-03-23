use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::db::DbState;
use crate::errors::AppError;
use crate::onebot::OneBotClientState;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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

/// 检测桌面 QQ 是否正在运行
fn is_desktop_qq_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq QQ.exe", "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let running = stdout.contains("QQ.exe");
                if running {
                    tracing::warn!("检测到桌面 QQ 进程正在运行");
                }
                running
            }
            Err(e) => {
                tracing::warn!("检测桌面 QQ 进程失败: {}", e);
                false // 检测失败不阻止启动
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// 杀掉残留的 NapCat node.exe 进程。
/// 通过 WMIC 查找命令行包含 napcat 目录的 node.exe 进程并终止。
fn kill_orphan_napcat_processes(napcat_dir: &Path) {
    let napcat_path = napcat_dir.to_string_lossy().to_lowercase();

    #[cfg(target_os = "windows")]
    {
        // 用 wmic 查找所有 node.exe 进程的 PID 和命令行
        let output = Command::new("wmic")
            .args(["process", "where", "name='node.exe'", "get", "ProcessId,CommandLine", "/format:csv"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    let lower = line.to_lowercase();
                    if lower.contains(&napcat_path) || lower.contains("napcat") {
                        // CSV 格式: Node,CommandLine,ProcessId
                        let parts: Vec<&str> = line.split(',').collect();
                        if let Some(pid_str) = parts.last() {
                            let pid_str = pid_str.trim();
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                tracing::info!("发现残留 NapCat 进程 PID={}，正在终止", pid);
                                let _ = Command::new("taskkill")
                                    .args(["/F", "/PID", &pid.to_string()])
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output();
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("查找残留进程失败: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = napcat_path;
        // Unix: pkill -f "napcat_dir"
        let _ = Command::new("pkill")
            .args(["-f", &napcat_dir.to_string_lossy()])
            .output();
    }

    // 等待端口释放
    std::thread::sleep(Duration::from_secs(2));
}

/// 查找 NapCat 可执行入口。优先直接使用 node.exe 以确保 kill() 能正确终止进程。
/// 如果提供了 qq_number，会在参数中加入 `-q QQ号` 以快速登录。
fn find_napcat_executable(napcat_dir: &Path, qq_number: Option<&str>) -> Result<(PathBuf, Vec<String>), AppError> {
    let mut extra_args: Vec<String> = Vec::new();
    if let Some(qq) = qq_number {
        if !qq.is_empty() {
            extra_args.push("-q".to_string());
            extra_args.push(qq.to_string());
        }
    }

    // 优先：Shell.Windows.Node 包结构 — node.exe 在根目录，index.js 为入口
    let node_root = napcat_dir.join("node.exe");
    if node_root.exists() {
        let index_js = napcat_dir.join("index.js");
        if index_js.exists() {
            let mut args = vec![index_js.to_string_lossy().to_string()];
            args.extend(extra_args);
            return Ok((node_root, args));
        }
    }

    // 备选：node 在 node/ 子目录（旧版结构）
    let node_sub = napcat_dir.join("node").join("node.exe");
    if node_sub.exists() {
        let script_candidates = [
            napcat_dir.join("napcat").join("napcat.mjs"),
            napcat_dir.join("napcat.mjs"),
        ];
        for script in &script_candidates {
            if script.exists() {
                let mut args = vec![script.to_string_lossy().to_string()];
                args.extend(extra_args.clone());
                return Ok((node_sub, args));
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

    // 检测桌面 QQ（仅警告，不阻止启动）
    if is_desktop_qq_running() {
        tracing::warn!("桌面 QQ 正在运行，可能导致同协议冲突。建议关闭桌面 QQ 以获得最佳体验。");
        if let Err(e) = app_handle.emit("napcat:log", serde_json::json!({
            "level": "warn",
            "event": "qq_running",
            "message": "检测到桌面 QQ 正在运行，可能导致冲突。建议关闭桌面 QQ。"
        })) {
            tracing::warn!("emit napcat:log 失败: {}", e);
        }
    }

    // 杀掉残留的 NapCat node.exe 进程（上次异常退出可能残留）
    kill_orphan_napcat_processes(napcat_dir);

    // 启动前删除旧的二维码文件，防止发送过期二维码
    let old_qr = napcat_dir.join("napcat").join("cache").join("qrcode.png");
    if old_qr.exists() {
        let _ = std::fs::remove_file(&old_qr);
        tracing::info!("已删除旧二维码文件: {:?}", old_qr);
    }

    // 尝试从 DB 读取已保存的 QQ 号，支持快速登录
    let saved_qq: Option<String> = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        crate::db::models::get_config_by_key(&conn, "qq_number")
            .ok()
            .map(|c| c.value)
            .filter(|v| !v.is_empty())
    };
    if let Some(ref qq) = saved_qq {
        tracing::info!("已保存的 QQ 号: {}，将尝试快速登录", qq);
    }

    let (program, args) = find_napcat_executable(napcat_dir, saved_qq.as_deref())?;
    tracing::info!("启动 NapCat: {:?} {:?}", program, args);

    // 启动前记录配置摘要，便于排查问题
    log_napcat_config_summary(napcat_dir, api_port);

    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(napcat_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd
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

    // 后台读取 stdout，检测关键事件并结构化输出
    if let Some(stdout) = stdout {
        let app_clone = app_handle.clone();
        tokio::task::spawn_blocking(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                tracing::info!("NapCat stdout: {}", line);
                // 结构化检测关键事件
                parse_napcat_event(&app_clone, &line);
                // 检测二维码文件路径
                if let Some(path) = extract_qr_path(&line) {
                    emit_qr_data_uri(&app_clone, Path::new(&path));
                }
            }
            tracing::info!("NapCat stdout 流结束");
        });
    }

    // 后台读取 stderr，检测关键事件
    if let Some(stderr) = stderr {
        let app_clone = app_handle.clone();
        tokio::task::spawn_blocking(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                tracing::info!("NapCat stderr: {}", line);
                parse_napcat_event(&app_clone, &line);
                if let Some(path) = extract_qr_path(&line) {
                    emit_qr_data_uri(&app_clone, Path::new(&path));
                }
            }
            tracing::info!("NapCat stderr 流结束");
        });
    }

    // 备选：文件监控检测二维码（若 stdout 未捕获到）
    let app_clone = app_handle.clone();
    let napcat_dir_clone = napcat_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        monitor_qr_file(&app_clone, &napcat_dir_clone);
    });

    // 后台登录轮询 → 成功后启动健康检查
    let app_clone = app_handle.clone();
    let db_clone = db.clone();
    let process_state_clone = process_state.clone();
    let process_state_for_poll = process_state.clone();
    let onebot_clone = onebot.clone();
    tokio::spawn(async move {
        match poll_login_status(&app_clone, api_port, &db_clone, &process_state_for_poll).await {
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

/// 记录 NapCat 配置摘要，方便排查启动问题
fn log_napcat_config_summary(napcat_dir: &Path, api_port: u16) {
    tracing::info!("=== NapCat 启动配置摘要 ===");
    tracing::info!("  NapCat 目录: {:?}", napcat_dir);
    tracing::info!("  API 端口: {}", api_port);

    // config.json
    let config_path = napcat_dir.join("config.json");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let ver = json.get("curVersion").and_then(|v| v.as_str()).unwrap_or("?");
                    let build = json.get("buildId").and_then(|v| v.as_str()).unwrap_or("?");
                    tracing::info!("  config.json: curVersion={}, buildId={}", ver, build);
                }
            }
            Err(e) => tracing::warn!("  config.json 读取失败: {}", e),
        }
    } else {
        tracing::warn!("  config.json 不存在");
    }

    // qqnt.json
    let qqnt_path = napcat_dir.join("napcat").join("qqnt.json");
    if qqnt_path.exists() {
        match std::fs::read_to_string(&qqnt_path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let platform = json.get("platform").and_then(|v| v.as_str()).unwrap_or("?");
                    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("?");
                    tracing::info!("  qqnt.json: platform={}, version={}", platform, version);
                }
            }
            Err(e) => tracing::warn!("  qqnt.json 读取失败: {}", e),
        }
    } else {
        tracing::warn!("  qqnt.json 不存在");
    }

    // OneBot11 配置
    let onebot_path = napcat_dir.join("napcat").join("config").join("onebot11.json");
    if onebot_path.exists() {
        match std::fs::read_to_string(&onebot_path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // HTTP Server 端口
                    if let Some(servers) = json.pointer("/network/httpServers") {
                        if let Some(arr) = servers.as_array() {
                            for srv in arr {
                                let port = srv.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
                                let name = srv.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                                let enabled = srv.get("enable").and_then(|v| v.as_bool()).unwrap_or(false);
                                tracing::info!("  OneBot HTTP 服务: name={}, port={}, enabled={}", name, port, enabled);
                            }
                        }
                    }
                    // HTTP Client (webhook) URL
                    if let Some(clients) = json.pointer("/network/httpClients") {
                        if let Some(arr) = clients.as_array() {
                            for cli in arr {
                                let url = cli.get("url").and_then(|v| v.as_str()).unwrap_or("?");
                                let name = cli.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                                let enabled = cli.get("enable").and_then(|v| v.as_bool()).unwrap_or(false);
                                tracing::info!("  OneBot Webhook: name={}, url={}, enabled={}", name, url, enabled);
                            }
                        }
                    }
                }
            }
            Err(e) => tracing::warn!("  onebot11.json 读取失败: {}", e),
        }
    } else {
        tracing::warn!("  onebot11.json 不存在");
    }

    // index.js 协议状态
    let index_path = napcat_dir.join("index.js");
    if index_path.exists() {
        match std::fs::read_to_string(&index_path) {
            Ok(content) => {
                if content.contains("NAPCAT_LINUX_PROTOCOL") {
                    tracing::warn!("  index.js: 检测到 Linux 协议注入（异常！）");
                } else {
                    tracing::info!("  index.js: 正常（无协议覆盖）");
                }
            }
            Err(e) => tracing::warn!("  index.js 读取失败: {}", e),
        }
    }

    tracing::info!("=== 配置摘要结束 ===");
}

/// 从 stdout 行中提取二维码文件路径
/// NapCat 输出格式: "二维码已保存到 C:\...\qrcode.png"
fn extract_qr_path(line: &str) -> Option<String> {
    if let Some(idx) = line.find("已保存到") {
        let after = line[idx + "已保存到".len()..].trim();
        // 去除 ANSI 转义序列中可能残留的控制字符
        let path = after.trim_start();
        if !path.is_empty() && (path.ends_with(".png") || path.ends_with(".jpg")) {
            return Some(path.to_string());
        }
    }
    None
}

/// 解析 NapCat stdout/stderr 中的关键事件，emit 结构化事件给前端
fn parse_napcat_event(app_handle: &tauri::AppHandle, line: &str) {
    // 去除 ANSI 转义序列方便匹配
    let clean = strip_ansi_codes(line);

    // 登录成功
    if clean.contains("已登录成功") || clean.contains("登录成功") {
        tracing::info!("[事件] NapCat 登录成功");
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "success",
            "event": "login_success",
            "message": "QQ 登录成功"
        }));
    }
    // 登录错误
    else if clean.contains("Login Error") {
        // 提取 ErrType 和 ErrCode
        let err_type = extract_field(&clean, "ErrType:");
        let err_code = extract_field(&clean, "ErrCode:");
        let detail = match (err_type.as_deref(), err_code.as_deref()) {
            (Some("1"), Some("3")) => "二维码已过期，请重新扫码",
            (Some("1"), Some("8")) => "协议/版本不匹配（appid 与引擎平台不一致）",
            _ => "登录失败，请重试",
        };
        tracing::warn!("[事件] NapCat 登录错误: ErrType={}, ErrCode={}, 原因: {}",
            err_type.as_deref().unwrap_or("?"),
            err_code.as_deref().unwrap_or("?"),
            detail
        );
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "error",
            "event": "login_error",
            "message": format!("登录错误: {}", detail),
            "err_type": err_type,
            "err_code": err_code
        }));
    }
    // Worker 进程崩溃
    else if clean.contains("Worker进程退出") || clean.contains("Worker进程意外退出") {
        tracing::error!("[事件] NapCat Worker 进程崩溃: {}", clean);
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "error",
            "event": "worker_crash",
            "message": format!("NapCat 内部进程崩溃: {}", clean.trim())
        }));
    }
    // NapCat 主进程退出（Worker 连续崩溃 3 次后）
    else if clean.contains("主进程退出") {
        tracing::error!("[事件] NapCat 主进程退出: {}", clean);
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "error",
            "event": "process_exit",
            "message": "NapCat 进程已退出（Worker 连续崩溃），请尝试更新 NapCat 或清理缓存后重试"
        }));
        // 直接 emit 错误状态，让 UI 退出等待
        let _ = app_handle.emit(
            "napcat:status-changed",
            &super::NapCatStatus::Error("NapCat Worker 连续崩溃退出，请尝试「更新 NapCat」或「清理缓存」后重试".to_string()),
        );
    }
    // 账号冲突
    else if clean.contains("已登录") && clean.contains("无法重复登录") {
        tracing::error!("[事件] 账号冲突: 该 QQ 号已在其他设备登录（Windows 协议）");
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "error",
            "event": "account_conflict",
            "message": "该 QQ 号已在桌面 QQ 登录，无法重复登录。请先关闭桌面 QQ 再试。"
        }));
    }
    // 扫码授权失败
    else if clean.contains("扫码授权失败") {
        tracing::warn!("[事件] 扫码授权失败");
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "error",
            "event": "qr_auth_failed",
            "message": "扫码授权失败，可能是 QQ 版本过低或网络问题"
        }));
    }
    // NapCat 版本信息
    else if clean.contains("NapCat.Core Version:") || clean.contains("NapCat] [Core]") {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "version_info",
            "message": clean.trim()
        }));
    }
    // WebUi 信息
    else if clean.contains("WebUi") && clean.contains("Url:") {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "webui_ready",
            "message": clean.trim()
        }));
    }
    // 网络状态
    else if clean.contains("网络已连接") {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "network_connected",
            "message": "NapCat 网络已连接"
        }));
    }
    else if clean.contains("等待网络连接") {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "network_waiting",
            "message": "NapCat 正在等待网络连接..."
        }));
    }
    // 快速登录可用账号
    else if clean.contains("可用于快速登录") {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "quick_login_available",
            "message": clean.trim()
        }));
    }
    // 请扫描二维码
    else if clean.contains("请扫描下面的二维码") {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "qr_ready",
            "message": "二维码已生成，请使用手机 QQ 扫码"
        }));
    }
    // HTTP 服务端口
    else if clean.contains("httpServers") || (clean.contains("HTTP") && clean.contains("port")) {
        let _ = app_handle.emit("napcat:log", serde_json::json!({
            "level": "info",
            "event": "http_server",
            "message": clean.trim()
        }));
    }
}

/// 简易去除 ANSI 转义序列
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // 跳过 ESC[ ... m 序列
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// 从文本中提取 "Key: Value" 格式的字段值
fn extract_field(text: &str, key: &str) -> Option<String> {
    if let Some(idx) = text.find(key) {
        let after = text[idx + key.len()..].trim();
        let value: String = after.chars().take_while(|c| !c.is_whitespace()).collect();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

/// 读取图片文件并以 data URI 形式 emit napcat:qr-code 事件。
/// 等待文件写完（文件大小稳定且 >= 1KB）后再读取。
fn emit_qr_data_uri(app_handle: &tauri::AppHandle, path: &Path) {
    // 等待文件写入完成：连续两次读取大小一致且 > 0
    // NapCat 生成的 QR 码 PNG 很小（约 600B），不能用 1KB 阈值
    let mut last_size: u64 = 0;
    let mut stable_count = 0;
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(200));
        match std::fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();
                if size > 0 && size == last_size {
                    stable_count += 1;
                    if stable_count >= 2 {
                        break;
                    }
                } else {
                    stable_count = 0;
                }
                last_size = size;
            }
            Err(_) => return,
        }
    }

    match std::fs::read(path) {
        Ok(bytes) => {
            if bytes.len() < 100 {
                tracing::warn!("二维码文件太小 ({}B)，跳过: {:?}", bytes.len(), path);
                return;
            }
            // 验证 PNG 头部
            if bytes.len() < 8 || &bytes[..4] != b"\x89PNG" {
                tracing::warn!("二维码文件不是有效 PNG: {:?}", path);
                return;
            }
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let data_uri = format!("data:image/png;base64,{}", b64);
            if let Err(e) = app_handle.emit("napcat:qr-code", &data_uri) {
                tracing::warn!("emit napcat:qr-code 失败: {}", e);
            } else {
                tracing::info!("已发送二维码 data URI ({}B)", bytes.len());
            }
        }
        Err(e) => {
            tracing::warn!("读取二维码文件失败: {:?}: {}", path, e);
        }
    }
}

/// 文件监控检测二维码（stdout 未捕获时兜底）。
/// 持续监控二维码文件变化，以便刷新后能重新发送。
fn monitor_qr_file(app_handle: &tauri::AppHandle, napcat_dir: &Path) {
    let possible_dirs = [
        napcat_dir.join("napcat").join("cache"),
        napcat_dir.join("napcat").join("data"),
        napcat_dir.join("napcat"),
        napcat_dir.join("data"),
        napcat_dir.to_path_buf(),
    ];

    let mut last_modified: Option<std::time::SystemTime> = None;

    // 持续监控 10 分钟（600 × 1s），与登录轮询超时一致
    for _ in 0..600 {
        std::thread::sleep(Duration::from_secs(1));
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
                            // 检查文件是否有更新
                            let modified = std::fs::metadata(&path)
                                .ok()
                                .and_then(|m| m.modified().ok());
                            if modified != last_modified && modified.is_some() {
                                last_modified = modified;
                                tracing::info!("文件监控检测到二维码更新: {:?}", path);
                                emit_qr_data_uri(app_handle, &path);
                            }
                        }
                    }
                }
            }
        }
    }
    tracing::debug!("文件监控超时退出（10 分钟）");
}

pub async fn poll_login_status(
    app_handle: &tauri::AppHandle,
    api_port: u16,
    db: &DbState,
    process_state: &NapCatProcessState,
) -> Result<super::LoginInfo, AppError> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/get_login_info", api_port);
    let timeout = tokio::time::Instant::now() + Duration::from_secs(600); // 10 分钟

    if let Err(e) =
        app_handle.emit("napcat:status-changed", &super::NapCatStatus::WaitingForLogin)
    {
        tracing::warn!("emit napcat:status-changed 失败: {}", e);
    }

    tracing::info!("开始登录轮询: API 地址 http://127.0.0.1:{}/get_login_info, 超时 600 秒", api_port);

    let mut attempt: u64 = 0;
    let mut consecutive_connection_errors: u32 = 0;
    loop {
        if tokio::time::Instant::now() > timeout {
            tracing::error!("登录轮询超时（10 分钟），共尝试 {} 次", attempt);
            return Err(AppError::NapCat("登录轮询超时（10 分钟）".to_string()));
        }

        // 检查 NapCat 进程是否还活着
        let process_alive = {
            let mut guard = match process_state.lock() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            guard.as_mut().map_or(false, |p| p.is_alive())
        };
        if !process_alive {
            // 进程已退出，等待一小段时间让 stdout 解析器有机会 emit 错误事件
            tokio::time::sleep(Duration::from_secs(2)).await;
            tracing::error!("登录轮询: NapCat 进程已退出，停止轮询");
            return Err(AppError::NapCat("NapCat 进程已退出".to_string()));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
        attempt += 1;

        match client
            .post(&url)
            .json(&serde_json::json!({}))
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                consecutive_connection_errors = 0;
                let status_code = resp.status();
                if !status_code.is_success() {
                    if attempt <= 5 || attempt % 10 == 0 {
                        tracing::info!(
                            "登录轮询 #{}: NapCat API 返回 HTTP {} (服务未就绪)",
                            attempt, status_code
                        );
                    }
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
            Err(e) => {
                consecutive_connection_errors += 1;
                if attempt <= 3 {
                    tracing::info!(
                        "登录轮询 #{}: NapCat API 未就绪 ({}), 继续等待...",
                        attempt, e
                    );
                } else if attempt % 15 == 0 {
                    tracing::warn!(
                        "登录轮询 #{}: NapCat API 仍不可达 ({}), 持续等待...",
                        attempt, e
                    );
                }
                // 连续 10 次连接失败（约 20 秒）且进程可能已死
                if consecutive_connection_errors >= 10 {
                    let alive = {
                        let mut guard = match process_state.lock() {
                            Ok(g) => g,
                            Err(e) => e.into_inner(),
                        };
                        guard.as_mut().map_or(false, |p| p.is_alive())
                    };
                    if !alive {
                        tracing::error!("登录轮询: 连续 {} 次连接失败且进程已退出", consecutive_connection_errors);
                        return Err(AppError::NapCat("NapCat 进程已退出".to_string()));
                    }
                }
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
    // 登录成功后给 NapCat 充足的初始化时间，避免 503 误判为僵死
    tracing::info!("健康检查: 等待 60 秒让 NapCat 完成初始化...");
    tokio::time::sleep(Duration::from_secs(60)).await;

    let mut restart_count: u32 = 0;
    let restart_delays = [5u64, 15, 30];
    let mut offline_notified = false;
    let mut alive_but_unresponsive_count: u32 = 0;
    const MAX_UNRESPONSIVE: u32 = 10; // 10 次 × 30 秒 = 5 分钟无响应则视为僵死

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
                let err_str = e.to_string();
                tracing::warn!("健康检查 get_login_info 失败: {}", err_str);

                // 503 表示服务暂时不可用（NapCat 还在初始化），不算"无响应"
                let is_503 = err_str.contains("503");
                if is_503 {
                    tracing::info!("NapCat API 返回 503，服务初始化中，继续等待");
                    continue;
                }

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
