# Story 1.4: NapCat 进程启动与扫码登录

Status: Done

## Story

As a 用户,
I want 扫码登录 QQ 后应用自动开始工作,
so that 我只需要扫一次码就能用起来。

## Acceptance Criteria

1. **Given** NapCat 已解压到目标目录（Story 1.3）**When** 启动 NapCat 进程 **Then** 自动生成 NapCat OneBot 配置文件（HTTP API 端口 3000、Webhook 端口 8080、仅监听 localhost）
2. **Given** 配置文件已生成 **When** 启动 NapCat Shell 子进程 **Then** 通过 Tauri shell 插件启动 NapCat Shell 子进程
3. **Given** NapCat 进程启动成功 **When** NapCat 生成二维码 **Then** 二维码图片路径通过 Tauri event `emit("napcat:qr-code")` 推送给前端展示
4. **Given** 二维码已展示 **When** 用户扫码 **Then** 轮询 `/get_login_info` 检测登录状态
5. **Given** 扫码成功 **When** 获取到 QQ 号和昵称 **Then** 保存到 config 表（qq_number、qq_nickname）
6. **Given** 登录成功 **When** 信息保存完成 **Then** 通过 Tauri event `emit("napcat:login-success")` 通知前端
7. **Given** 应用退出 **When** NapCat 子进程仍在运行 **Then** 优雅停止 NapCat 子进程
8. **Given** 前端需要操作 **When** 调用 IPC **Then** 提供 Tauri commands：`start_napcat`、`stop_napcat`、`get_login_info`

## Tasks / Subtasks

- [X] Task 1: 扩展 NapCatStatus 枚举和错误类型 (AC: #1-#8)
  - [X] 1.1 编辑 `src-tauri/src/napcat/mod.rs`：NapCatStatus 枚举新增 `Starting`、`WaitingForLogin`、`Running` 变体
  - [X] 1.2 编辑 `src-tauri/src/errors.rs`：添加 `NapCat(String)` 错误变体
  - [X] 1.3 编辑 `src/types/napcat.ts`：TypeScript 类型同步新增三个状态
- [X] Task 2: 实现 NapCat OneBot 配置生成 (AC: #1)
  - [X] 2.1 创建 `src-tauri/src/napcat/config.rs`：实现 `generate_napcat_config()` 函数
  - [X] 2.2 生成 NapCat OneBot11 配置 JSON 文件到 napcat 目录（HTTP API 端口从 config 表读取，默认 3000；Webhook 端口默认 8080；仅监听 127.0.0.1）
  - [X] 2.3 在 `napcat/mod.rs` 中添加 `pub mod config;`
- [X] Task 3: 实现 NapCat 进程管理 (AC: #2, #7)
  - [X] 3.1 创建 `src-tauri/src/napcat/process.rs`：实现 `NapCatProcess` 结构体，封装进程启动、停止、状态查询
  - [X] 3.2 使用 `tauri_plugin_shell::ShellExt` 的 `Command::new()` 创建子进程（非 sidecar，使用 shell 命令执行 napcat 目录下的启动脚本）
  - [X] 3.3 将 `NapCatProcess` 通过 Tauri State 管理（`Arc<Mutex<Option<NapCatProcess>>>`）
  - [X] 3.4 实现 `stop()` 方法：通过 `kill()` 终止子进程
  - [X] 3.5 在 `napcat/mod.rs` 中添加 `pub mod process;`
- [X] Task 4: 实现二维码检测与推送 (AC: #3)
  - [X] 4.1 在 `process.rs` 中实现 QR 码检测：监听 NapCat 进程 stdout/stderr 输出，解析包含二维码路径的行
  - [X] 4.2 二维码路径通过 `app_handle.emit("napcat:qr-code", &qr_path)` 推送给前端
  - [X] 4.3 备选方案：如果 NapCat 将二维码保存为文件，监控 napcat 数据目录变化检测 `qrcode.png` 文件
- [X] Task 5: 实现登录状态轮询 (AC: #4, #5, #6)
  - [X] 5.1 在 `process.rs` 中实现 `poll_login_status()` 异步函数：每 2 秒调用 `http://127.0.0.1:{api_port}/get_login_info`
  - [X] 5.2 解析响应 JSON，提取 `user_id`（QQ 号）和 `nickname`
  - [X] 5.3 登录成功时：保存 qq_number 和 qq_nickname 到 config 表（通过 db::models::upsert_config）
  - [X] 5.4 登录成功时：emit("napcat:login-success", { qqNumber, nickname })
  - [X] 5.5 状态变更时：emit("napcat:status-changed", &new_status)
  - [X] 5.6 轮询超时（5 分钟无响应）或 NapCat 进程退出时停止轮询并报告错误
- [X] Task 6: 实现 Tauri IPC Commands (AC: #8)
  - [X] 6.1 编辑 `src-tauri/src/commands/napcat.rs`：添加 `start_napcat`、`stop_napcat`、`get_login_info` 命令
  - [X] 6.2 `start_napcat`：调用配置生成 → 启动进程 → 开始二维码监控 → 启动登录轮询
  - [X] 6.3 `stop_napcat`：优雅停止 NapCat 进程
  - [X] 6.4 `get_login_info`：代理调用 NapCat HTTP API `/get_login_info` 并返回结果
  - [X] 6.5 在 `lib.rs` 的 `invoke_handler` 中注册三个新命令
- [X] Task 7: 应用退出清理 (AC: #7)
  - [X] 7.1 在 `lib.rs` 的 `setup()` 或通过 `on_window_event` 处理 `CloseRequested` 事件
  - [X] 7.2 退出前检查 NapCatProcess State，如果进程存在则调用 `stop()`
- [X] Task 8: 前端类型与 IPC 封装 (AC: #3, #6)
  - [X] 8.1 编辑 `src/types/napcat.ts`：添加 `LoginInfo` 接口
  - [X] 8.2 编辑 `src/lib/tauri.ts`：添加 `startNapcat()`、`stopNapcat()`、`getLoginInfo()` 类型安全封装
- [X] Task 9: 构建验证 (AC: #1-#8)
  - [X] 9.1 `cargo check` 编译通过
  - [X] 9.2 `npx tsc --noEmit` TypeScript 类型检查通过
  - [X] 9.3 `cargo build` 完整编译通过

## Dev Notes

### NapCat Shell OneKey 启动机制

NapCat Shell OneKey 包解压后目录结构类似：

```
napcat/
├── NapCat.Shell.bat       ← Windows 启动脚本（或 launcher.bat）
├── napcat/                ← NapCat 核心文件
│   ├── napcat.mjs         ← Node.js 入口
│   └── ...
├── node/                  ← 内嵌 Node.js 运行时
│   └── node.exe
└── config/                ← 配置文件目录
    └── onebot11_{qq号}.json
```

**重要：** 实际目录结构可能因 NapCat 版本不同有差异。Dev Agent 必须在运行时检查实际解压目录结构，找到启动脚本（`.bat` 文件）或直接构造 `node.exe napcat.mjs` 命令。

### NapCat OneBot 配置文件参考

NapCat 需要一个 OneBot11 配置文件才能正确启动 HTTP API 和 Webhook。配置文件通常位于 `napcat/config/` 目录。

```json
{
  "http": {
    "enable": true,
    "host": "127.0.0.1",
    "port": 3000,
    "secret": "",
    "enableHeart": false,
    "enablePost": false
  },
  "httpServers": [],
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
}
```

**注意：** 以上是参考配置结构。实际 NapCat 版本的配置字段可能不同。Dev Agent 应：

1. 检查 napcat 目录下是否有示例配置或文档
2. 至少确保 HTTP API 开启 + 端口 + host 正确
3. Webhook/反向 WS 的配置在 Story 5.1 中单独处理

### 进程启动方案

使用 `tauri-plugin-shell` 的 `Command` API 启动 NapCat：

```rust
use tauri_plugin_shell::ShellExt;

// 方案一：执行 bat 文件
let (mut rx, child) = app_handle.shell()
    .command("cmd")
    .args(["/C", &bat_path.to_string_lossy()])
    .spawn()
    .map_err(|e| AppError::NapCat(e.to_string()))?;

// 方案二：直接执行 node
let node_path = napcat_dir.join("node").join("node.exe");
let script_path = napcat_dir.join("napcat").join("napcat.mjs");
let (mut rx, child) = app_handle.shell()
    .command(node_path.to_string_lossy())
    .args([script_path.to_string_lossy().as_ref()])
    .spawn()
    .map_err(|e| AppError::NapCat(e.to_string()))?;
```

**关键：** `spawn()` 返回 `(CommandEvents, CommandChild)`，其中 `CommandEvents` 是一个 `Receiver` 可以接收 stdout/stderr/终止事件。使用 `tokio::spawn` 在后台持续读取输出，解析二维码路径。

### 二维码检测策略

NapCat 启动后生成二维码有两种常见方式：

1. **stdout 输出**：NapCat 在 stdout 打印二维码路径或 Base64 编码
2. **文件输出**：二维码图片保存到 `napcat/` 数据目录下

```rust
// 监听进程输出
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                let line_str = String::from_utf8_lossy(&line);
                // 检查是否包含二维码路径
                if line_str.contains("qrcode") || line_str.contains("QR") {
                    tracing::info!("检测到二维码输出: {}", line_str);
                    // 解析二维码路径并 emit
                    let _ = app_handle.emit("napcat:qr-code", &line_str.trim());
                }
                tracing::debug!("NapCat stdout: {}", line_str);
            }
            CommandEvent::Stderr(line) => {
                tracing::warn!("NapCat stderr: {}", String::from_utf8_lossy(&line));
            }
            CommandEvent::Terminated(status) => {
                tracing::warn!("NapCat 进程退出: {:?}", status);
                let _ = app_handle.emit("napcat:status-changed", "error");
                break;
            }
            _ => {}
        }
    }
});
```

**备选：文件监控方案**

如果 NapCat 不通过 stdout 输出二维码，需要监控文件系统：

```rust
// 每 500ms 检查二维码文件
let qr_dir = napcat_dir.join("data"); // 或其他可能的目录
tokio::spawn(async move {
    for _ in 0..60 { // 最多等 30 秒
        tokio::time::sleep(Duration::from_millis(500)).await;
        // 扫描目录下的 png 文件，找最新的二维码
        if let Ok(entries) = std::fs::read_dir(&qr_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "png") {
                    let _ = app_handle.emit("napcat:qr-code", path.to_string_lossy().as_ref());
                    return; // 找到就退出
                }
            }
        }
    }
    tracing::warn!("30 秒内未检测到二维码文件");
});
```

### 登录轮询参考

```rust
pub async fn poll_login_status(
    app_handle: &tauri::AppHandle,
    api_port: u16,
    db: &db::DbState,
) -> Result<LoginInfo, AppError> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/get_login_info", api_port);

    let timeout = tokio::time::Instant::now() + Duration::from_secs(300); // 5 分钟超时

    loop {
        if tokio::time::Instant::now() > timeout {
            return Err(AppError::NapCat("登录轮询超时（5 分钟）".to_string()));
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        match client.post(&url)
            .json(&serde_json::json!({}))
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    // OneBot 11 响应格式: { "status": "ok", "data": { "user_id": 123, "nickname": "xxx" } }
                    if body["status"] == "ok" {
                        let user_id = body["data"]["user_id"].as_i64().unwrap_or(0);
                        let nickname = body["data"]["nickname"].as_str().unwrap_or("").to_string();

                        if user_id > 0 {
                            // 保存到 config 表
                            {
                                let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
                                db::models::upsert_config(&conn, "qq_number", &user_id.to_string())?;
                                db::models::upsert_config(&conn, "qq_nickname", &nickname)?;
                            }

                            let login_info = LoginInfo {
                                qq_number: user_id.to_string(),
                                nickname: nickname.clone(),
                            };

                            let _ = app_handle.emit("napcat:login-success", &login_info);
                            let _ = app_handle.emit("napcat:status-changed", "running");

                            tracing::info!("QQ 登录成功: {} ({})", nickname, user_id);
                            return Ok(login_info);
                        }
                    }
                }
            }
            Err(_) => {
                // NapCat API 尚未就绪，继续轮询
                tracing::debug!("NapCat API 未就绪，继续等待...");
            }
        }
    }
}
```

### 扩展 NapCatStatus 枚举

```rust
// src-tauri/src/napcat/mod.rs — 修改后的完整枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NapCatStatus {
    NotInstalled,       // 未安装（napcat 目录不存在）
    Downloading,        // 正在下载
    Extracting,         // 正在解压
    Ready,              // 已安装但未运行
    Starting,           // 进程已启动，等待 API 就绪
    WaitingForLogin,    // API 就绪，等待扫码登录
    Running,            // 已登录，正常运行
    Error(String),      // 错误状态
}
```

### 扩展 errors.rs

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("配置项不存在: {0}")]
    ConfigNotFound(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("下载错误: {0}")]
    Download(#[from] reqwest::Error),
    #[error("解压错误: {0}")]
    Extract(String),
    #[error("NapCat 错误: {0}")]
    NapCat(String),  // ← 新增
}
```

### NapCatProcess 结构体参考

```rust
// src-tauri/src/napcat/process.rs
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;

pub struct NapCatProcess {
    child: CommandChild,
    api_port: u16,
}

impl NapCatProcess {
    pub fn new(child: CommandChild, api_port: u16) -> Self {
        Self { child, api_port }
    }

    pub fn api_port(&self) -> u16 {
        self.api_port
    }

    pub fn stop(&mut self) -> Result<(), crate::errors::AppError> {
        self.child.kill()
            .map_err(|e| crate::errors::AppError::NapCat(format!("停止 NapCat 失败: {}", e)))
    }
}

pub type NapCatProcessState = Arc<Mutex<Option<NapCatProcess>>>;
```

### LoginInfo 类型定义

```rust
// 在 napcat/mod.rs 中添加
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub qq_number: String,
    pub nickname: String,
}
```

```typescript
// src/types/napcat.ts 追加
export interface LoginInfo {
  qqNumber: string;
  nickname: string;
}
```

### Tauri Command 签名参考

```rust
// src-tauri/src/commands/napcat.rs — 新增命令
use crate::napcat::process::NapCatProcessState;

#[tauri::command]
pub async fn start_napcat(
    app: tauri::AppHandle,
    db: State<'_, Arc<Mutex<Connection>>>,
    process_state: State<'_, NapCatProcessState>,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // 1. 生成 NapCat OneBot 配置
    crate::napcat::config::generate_napcat_config(&app_data_dir, &db)
        .map_err(|e| e.to_string())?;

    // 2. 启动进程（使用 tauri_plugin_shell）
    // 3. 后台监听 stdout 检测二维码
    // 4. 后台启动登录轮询
    // 5. 保存 child 到 process_state

    Ok(())
}

#[tauri::command]
pub fn stop_napcat(
    process_state: State<'_, NapCatProcessState>,
) -> Result<(), String> {
    let mut state = process_state.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut process) = *state {
        process.stop().map_err(|e| e.to_string())?;
        *state = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_login_info(
    process_state: State<'_, NapCatProcessState>,
) -> Result<crate::napcat::LoginInfo, String> {
    let api_port = {
        let state = process_state.lock().map_err(|e| e.to_string())?;
        state.as_ref()
            .map(|p| p.api_port())
            .ok_or("NapCat 进程未运行".to_string())?
    };

    let client = reqwest::Client::new();
    let resp = client.post(format!("http://127.0.0.1:{}/get_login_info", api_port))
        .json(&serde_json::json!({}))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    // 解析 OneBot 响应...
    Ok(crate::napcat::LoginInfo {
        qq_number: body["data"]["user_id"].as_i64().unwrap_or(0).to_string(),
        nickname: body["data"]["nickname"].as_str().unwrap_or("").to_string(),
    })
}
```

### lib.rs 修改要点

```rust
// 新增 State 管理
use napcat::process::NapCatProcessState;

.setup(|app| {
    // ... 现有 db 初始化 ...

    // 初始化 NapCat 进程状态
    let napcat_state: NapCatProcessState = std::sync::Arc::new(std::sync::Mutex::new(None));
    app.manage(napcat_state);

    Ok(())
})
.invoke_handler(tauri::generate_handler![
    // 现有命令...
    commands::napcat::start_napcat,
    commands::napcat::stop_napcat,
    commands::napcat::get_login_info_cmd, // 避免与模块函数同名
])
```

### 前端类型扩展参考

```typescript
// src/types/napcat.ts — 完整更新
export type NapCatStatus =
  | "notInstalled"
  | "downloading"
  | "extracting"
  | "ready"
  | "starting"           // ← 新增
  | "waitingForLogin"    // ← 新增
  | "running"            // ← 新增
  | { error: string };

export interface LoginInfo {
  qqNumber: string;
  nickname: string;
}
// ... DownloadProgress, ExtractProgress 保持不变
```

```typescript
// src/lib/tauri.ts 追加
import type { LoginInfo } from "@/types/napcat";

export async function startNapcat(): Promise<void> {
  return invoke("start_napcat");
}

export async function stopNapcat(): Promise<void> {
  return invoke("stop_napcat");
}

export async function getLoginInfo(): Promise<LoginInfo> {
  return invoke<LoginInfo>("get_login_info_cmd");
}
```

### tauri-plugin-shell 权限

当前 `capabilities/default.json` 已有 `shell:allow-open`。使用 `Command::new()` + `spawn()` 需要额外权限。需要添加：

```json
{
  "permissions": [
    "core:default",
    "shell:allow-open",
    "shell:allow-spawn",
    "shell:default",
    "notification:default",
    "autostart:default",
    "log:default"
  ]
}
```

**重要：** Tauri 2.0 的 shell 插件权限模型较严格。如果 `shell:allow-spawn` 不够，可能需要在 `capabilities` 中配置 `shell:allow-execute` 或使用 scope 配置。但 Story 1.1 QA 已移除 `shell:allow-execute`（H3/M2），需要慎重评估。建议优先使用 `Command::new()` + `spawn()` 配合最小权限。如果受限，可改用 `std::process::Command`（Rust 标准库）+ `tokio::process::Command`（异步）直接启动进程，绕过 Tauri shell 插件。

### 强制规则清单

1. **所有 Rust 结构体** 必须添加 `#[serde(rename_all = "camelCase")]`
2. **Tauri commands** 返回 `Result<T, String>`，使用 `.map_err(|e| e.to_string())`
3. **禁止** `println!`，使用 `tracing::info!` / `warn!` / `error!`
4. **禁止** `unwrap()` / `expect()` 在非初始化代码中
5. **事件命名** 使用 `namespace:action` 格式（`napcat:qr-code`、`napcat:login-success`、`napcat:status-changed`）
6. **用户面向文本** 隐藏 NapCat 术语，展示为"运行环境"
7. **异步阻塞** Story 1.3 QA M2 发现的问题 — 同步 IO 操作必须用 `tokio::task::spawn_blocking()` 包裹，不得阻塞 Tokio 异步线程
8. **进程清理** 确保所有 `tokio::spawn` 启动的后台任务在 NapCat 停止时能正确取消

### Story 1.3 经验教训

- **M1 HTTP 状态码未检查**：本 Story 调用 NapCat API 时必须检查 HTTP 响应码，使用 `resp.error_for_status()` 或手动检查
- **M2 阻塞异步线程**：文件 IO 操作（配置文件生成、二维码文件读取）必须用 `spawn_blocking` 或异步 IO
- **M3 部分文件残留**：本 Story 的配置文件生成应该是覆盖写入（非增量），每次都完整重写
- **Story 1.2 L2 emit 错误丢弃**：emit 失败时应 `tracing::warn!` 记录，不要用 `let _ =`
- **pnpm 在 bash 环境不可用**：构建验证使用 `npx` 或直接 `cargo check` + `npx tsc --noEmit`

### 目录结构

本 Story 需要创建/修改的文件：

```
src-tauri/src/
├── lib.rs                    ← 修改：添加 NapCatProcessState 管理 + invoke_handler 新命令 + 退出清理
├── errors.rs                 ← 修改：添加 NapCat(String) 错误变体
├── napcat/
│   ├── mod.rs                ← 修改：新增 Starting/WaitingForLogin/Running 状态 + LoginInfo 结构体 + pub mod config; process;
│   ├── config.rs             ← 新建：generate_napcat_config() 配置文件生成
│   └── process.rs            ← 新建：NapCatProcess 结构体 + 进程启停 + 二维码检测 + 登录轮询
└── commands/
    └── napcat.rs             ← 修改：添加 start_napcat + stop_napcat + get_login_info 命令

src-tauri/capabilities/
└── default.json              ← 修改：添加 shell spawn 权限

src/
├── types/
│   └── napcat.ts             ← 修改：新增 starting/waitingForLogin/running 状态 + LoginInfo 接口
└── lib/
    └── tauri.ts              ← 修改：添加 startNapcat() + stopNapcat() + getLoginInfo() 封装
```

### Project Structure Notes

- `napcat/config.rs` 和 `napcat/process.rs` 是架构文档中定义的模块，本 Story 首次实现
- NapCatProcessState 作为 Tauri State 与 DbState 并列管理
- `commands/napcat.rs` 文件已存在（Story 1.3 创建），本 Story 在其中追加新命令
- 退出清理逻辑在 `lib.rs` 的 `setup()` 或 window event handler 中实现

### References

- [Source: .bmad-method/planning-artifacts/architecture.md#napcat/] — napcat/process.rs（进程启停+健康检查）、napcat/config.rs（生成 NapCat OneBot 配置文件）
- [Source: .bmad-method/planning-artifacts/architecture.md#实现模式与一致性规则] — 命名规范、错误处理分层、事件命名
- [Source: .bmad-method/planning-artifacts/architecture.md#Tauri 事件命名] — napcat:status-changed、napcat:login-required
- [Source: .bmad-method/planning-artifacts/architecture.md#API与通信] — Tauri IPC 命令模式、事件推送模式
- [Source: .bmad-method/planning-artifacts/architecture.md#安全策略] — Webhook 仅监听 localhost
- [Source: .bmad-method/planning-artifacts/epics.md#Story1.4] — AC 定义、技术要求
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#Core User Experience] — 扫码成功关键成功时刻、NapCat 术语隐藏
- [Source: .bmad-method/implementation-artifacts/1-3-napcat-download-and-extract.md#QA Results] — M1/M2/M3 经验教训
- [Source: .bmad-method/implementation-artifacts/1-2-sqlite-database-and-config.md#Dev Notes] — db 模块 CRUD API、config 表键值

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

### Completion Notes List

- Task 3.2 偏差：使用 `std::process::Command` 替代 `tauri_plugin_shell::ShellExt`。原因：Dev Notes 明确建议此方案作为替代，可避免 Tauri shell 插件复杂的权限配置（Story 1.1 QA 已移除 `shell:allow-execute`），且无需额外 capabilities 配置。
- 进程启动优先使用 `node.exe` + `napcat.mjs` 直接执行（避免 cmd.exe 包装层导致 `kill()` 无法终止实际 NapCat 进程），bat 文件作为回退方案。
- 所有 emit 失败均使用 `tracing::warn!` 记录（遵循 Story 1.2 L2 教训）。
- 配置文件生成使用 `tokio::task::spawn_blocking` 包裹（遵循 Story 1.3 M2 阻塞异步线程教训）。
- `on_window_event` 中使用 `match` + `e.into_inner()` 处理 poisoned mutex，确保退出清理即使在 mutex 中毒时也能执行。
- `stop()` 方法在 `kill()` 后调用 `wait()` 确保进程资源正确释放。
- 状态变更事件使用 `NapCatStatus` 枚举值序列化，确保与前端 TypeScript 类型完全一致。

### File List

- `src-tauri/src/errors.rs` — 修改：添加 `NapCat(String)` 错误变体
- `src-tauri/src/napcat/mod.rs` — 修改：新增 `Starting`/`WaitingForLogin`/`Running` 状态 + `LoginInfo` 结构体 + `pub mod config; process;`
- `src-tauri/src/napcat/config.rs` — 新建：`generate_napcat_config()` 配置文件生成
- `src-tauri/src/napcat/process.rs` — 新建：`NapCatProcess` 结构体 + 进程启停 + QR 检测(stdout+文件监控) + 登录轮询
- `src-tauri/src/commands/napcat.rs` — 修改：添加 `start_napcat` / `stop_napcat` / `get_login_info_cmd` 命令
- `src-tauri/src/lib.rs` — 修改：添加 `NapCatProcessState` 管理 + `on_window_event` 退出清理 + 注册新命令
- `src/types/napcat.ts` — 修改：新增 `starting`/`waitingForLogin`/`running` 状态 + `LoginInfo` 接口
- `src/lib/tauri.ts` — 修改：添加 `startNapcat()` / `stopNapcat()` / `getLoginInfo()` 类型安全封装

## QA Results

**Reviewer:** Quinn (Test Architect) — Claude Opus 4.6
**Date:** 2026-03-13
**Gate Decision:** CONCERNS
**Gate File:** `.bmad-method/test-artifacts/gates/1.4-napcat-process-and-qr-login.yml`

### Build Verification

| Check            | Result                                                     |
| ---------------- | ---------------------------------------------------------- |
| `cargo check`  | PASS (4 dead_code warnings — Story 1.2 遗留，非 1.4 新增) |
| `tsc --noEmit` | PASS (零错误)                                              |

### AC Coverage: 8/8 PASS

| AC | Description                                       | Result                                               |
| -- | ------------------------------------------------- | ---------------------------------------------------- |
| #1 | 自动生成 NapCat OneBot 配置文件                   | PASS                                                 |
| #2 | 通过子进程启动 NapCat Shell                       | PASS (偏差 D1: std::process 替代 tauri_plugin_shell) |
| #3 | 二维码路径通过 emit 推送前端                      | PASS                                                 |
| #4 | 轮询 /get_login_info 检测登录                     | PASS                                                 |
| #5 | 保存 qq_number/qq_nickname 到 config 表           | PASS                                                 |
| #6 | emit napcat:login-success 通知前端                | PASS                                                 |
| #7 | 退出时优雅停止 NapCat 子进程                      | PASS                                                 |
| #8 | 提供 start_napcat/stop_napcat/get_login_info 命令 | PASS                                                 |

### 前序教训执行情况

| 教训                       | 来源         | 执行                                                      |
| -------------------------- | ------------ | --------------------------------------------------------- |
| HTTP 状态码检查            | Story 1.3 M1 | PASS — poll_login_status 检查 resp.status().is_success() |
| spawn_blocking 包裹同步 IO | Story 1.3 M2 | PASS — config 生成用 spawn_blocking                      |
| emit 失败记录日志          | Story 1.2 L2 | PASS — 全部 emit 错误用 tracing::warn!                   |

### Issues Summary: 0 High / 3 Medium / 5 Low

**Medium Issues:**

- **M1**: 后台任务缺乏取消机制 — `stop_napcat` 后登录轮询(5分钟)和 QR 文件监控(30秒)继续空跑 (`process.rs:152-172`)
- **M2**: `get_login_info_cmd` 未检查 OneBot `status` 字段 — 错误响应时返回 qq_number="0" 的无效数据 (`commands/napcat.rs:107-114`)
- **M3**: 配置文件名 `onebot11.json` 可能不被 NapCat 识别（NapCat 期望 `onebot11_{qq号}.json`）(`config.rs:49`)

**Low Issues:**

- **L1**: api_port 在 start_napcat 和 generate_napcat_config 中重复读取
- **L2**: spawn_blocking 用于长时间阻塞任务，应改用 std::thread::spawn
- **L3**: QR 文件监控仅检测一次即返回，过期刷新后无法再次检测
- **L4**: WaitingForLogin 状态在轮询开始即 emit，而非 API 就绪时
- **L5**: Dev Notes 中 shell 权限描述与实际实现不一致（实际无需额外权限）

### Recommendation

PROCEED to Sprint 2 / Epic 2。修复优先级：

1. **M2 (立即)**: 添加 OneBot status 检查 — 一行代码变更
2. **M3 (验证)**: 实际运行 NapCat 验证配置文件名兼容性
3. **M1 (Sprint 2)**: 引入 CancellationToken 统一后台任务生命周期
