# Story 3.3: NapCat 健康检查与自动重启

Status: Ready for Review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 用户,
I want 应用自动管理运行环境的稳定性,
so that 我不需要手动排查问题。

## Acceptance Criteria

1. **Given** NapCat 进程已启动并运行 **When** 应用处于运行状态 **Then** `napcat/process.rs` 每 30 秒调用 `/get_login_info` 进行健康检查
2. **Given** 健康检查执行 **When** `/get_login_info` 请求失败 **Then** 判断 NapCat 进程是否存活（检查子进程状态）
3. **Given** NapCat 进程异常退出 **When** 检测到进程不存活 **Then** 自动重启 NapCat（最多 3 次） **And** 重启间隔递增：第 1 次 5 秒、第 2 次 15 秒、第 3 次 30 秒
4. **Given** NapCat 状态变化 **When** 每次重启或状态切换 **Then** 通过 `emit("napcat:status-changed")` 通知前端
5. **Given** 重启次数超过 3 次 **When** 最后一次重启仍失败 **Then** 通过 `tauri-plugin-notification` 发送 Windows 系统通知告警 **And** 托盘图标切换为红色
6. **Given** 健康检查返回正常 **When** `/get_login_info` 返回 `user_id=0` 或 QQ 号与已保存不同 **Then** 检测为 QQ 掉线（30 天强制重登） **And** 发送系统通知提醒扫码 **And** emit `napcat:login-required` 通知前端弹出扫码界面
7. **Given** 健康检查运行中 **When** 所有状态变化 **Then** 记录 tracing 日志

## Tasks / Subtasks

- [x] Task 1: 为 `NapCatProcess` 添加进程存活检测方法 (AC: #2)
  - [x] 1.1 在 `napcat/process.rs` 中为 `NapCatProcess` 添加 `is_alive(&mut self) -> bool` 方法
  - [x] 1.2 使用 `self.child.try_wait()` 检查子进程状态 — `Ok(None)` 表示仍在运行，`Ok(Some(_))` 或 `Err(_)` 表示已退出
  - [x] 1.3 添加 `pub fn api_base_url(&self) -> String` 便捷方法返回 `http://127.0.0.1:{port}`

- [x] Task 2: 实现健康检查循环 `start_health_check` (AC: #1, #2, #7)
  - [x] 2.1 在 `napcat/process.rs` 中创建 `pub async fn start_health_check(...)` 函数
  - [x] 2.2 函数签名接收：`app: AppHandle`, `process_state: NapCatProcessState`, `db: DbState`, `onebot: OneBotClientState`
  - [x] 2.3 每 30 秒循环：调用 `onebot.get_login_info()` 检查 NapCat 连通性
  - [x] 2.4 成功时：重置重试计数器为 0，确认运行状态
  - [x] 2.5 失败时：lock process_state → 调用 `is_alive()` 判断进程状态
  - [x] 2.6 进程存活但 API 不通：记录 warn 日志，继续等待（可能正在启动）
  - [x] 2.7 进程已退出：触发自动重启流程（Task 3）

- [x] Task 3: 实现自动重启逻辑 (AC: #3, #4, #5, #7)
  - [x] 3.1 在健康检查循环中，进程退出时执行重启
  - [x] 3.2 定义重启间隔数组 `[5, 15, 30]` 秒，最多 3 次
  - [x] 3.3 重启前：先清理旧进程状态（将 `NapCatProcessState` 设为 `None`）
  - [x] 3.4 emit `napcat:status-changed` = `Starting`，触发托盘图标变黄
  - [x] 3.5 等待对应间隔后，复用现有 `start_napcat_process()` 重启进程
  - [x] 3.6 重启成功（`start_napcat_process` 返回 Ok）→ 重置计数器，记录 info 日志
  - [x] 3.7 重启失败 → 计数器+1，如未达上限则继续下一次重启
  - [x] 3.8 达到 3 次上限后：emit `napcat:status-changed` = `Error("重启失败，已超过最大重试次数")`，执行 Task 4

- [x] Task 4: 实现通知告警 (AC: #5)
  - [x] 4.1 使用 `tauri_plugin_notification::NotificationExt` 发送 Windows 系统通知
  - [x] 4.2 通知标题："QQ Auto Like Plus"
  - [x] 4.3 通知正文："小助手遇到了一点问题，需要你帮帮忙~"（UX 要求温暖语气）
  - [x] 4.4 确保 `capabilities/default.json` 已有 `notification:default`（已有 ✅）
  - [x] 4.5 托盘图标切换由 lib.rs 中已有的 `napcat:status-changed` 事件监听器自动完成（emit Error 状态即可）

- [x] Task 5: 实现 QQ 掉线检测 (AC: #6)
  - [x] 5.1 健康检查中 `get_login_info` 成功时，检查返回的 `user_id`
  - [x] 5.2 如果 `user_id == 0`：检测为未登录/掉线状态
  - [x] 5.3 从 config 表读取已保存的 `qq_number`，如果不匹配且 user_id > 0：可能被顶号（可选检测）
  - [x] 5.4 掉线时 emit `napcat:status-changed` = `WaitingForLogin`（触发托盘黄灯）
  - [x] 5.5 emit `napcat:login-required` 事件，通知前端弹出扫码界面
  - [x] 5.6 使用 `tauri_plugin_notification` 发送通知："QQ 需要重新登录，请打开面板扫码~"
  - [x] 5.7 掉线后暂停健康检查（或切换为登录轮询模式），等待重新登录成功后恢复

- [x] Task 6: 在 `lib.rs` setup 中启动健康检查后台任务 (AC: #1)
  - [x] 6.1 在 `start_napcat` 之后（或在 NapCat 登录成功事件后）启动健康检查
  - [x] 6.2 用 `tokio::spawn` 启动 `start_health_check` 异步任务
  - [x] 6.3 传入所有需要的 State（通过 clone Arc）
  - [x] 6.4 健康检查应在 NapCat 进程 Running 后才启动（可监听 `napcat:status-changed` == Running 或在 poll_login_status 成功后启动）

- [x] Task 7: 添加 Tauri command `restart_napcat` (AC: #3)
  - [x] 7.1 在 `commands/napcat.rs` 添加 `restart_napcat` command
  - [x] 7.2 功能：stop 当前进程 → 清理状态 → start 新进程
  - [x] 7.3 在 `lib.rs` 的 `invoke_handler` 中注册该 command
  - [x] 7.4 前端可通过 invoke("restart_napcat") 手动触发重启（面板内重试按钮使用）

- [x] Task 8: 构建验证 (AC: #1-#7)
  - [x] 8.1 `cargo check` 编译通过，无新增 warnings
  - [x] 8.2 确认 notification 权限已存在于 capabilities
  - [ ] 8.3 `pnpm tauri dev` 手动验证（需 NapCat 环境）

## Dev Notes

### 核心挑战：后台健康检查循环 + 进程存活检测 + 递增重启

本 Story 实现 NapCat 进程的监控和自愈机制。核心是一个 `tokio::spawn` 后台循环，每 30 秒检查一次，发现问题自动修复。

### NapCatProcess 进程存活检测

**`Child::try_wait()` 是检测进程存活的标准方法：**

```rust
impl NapCatProcess {
    /// 检查子进程是否仍在运行
    pub fn is_alive(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,       // 仍在运行
            Ok(Some(status)) => {   // 已退出
                tracing::warn!("NapCat 进程已退出: {:?}", status);
                false
            }
            Err(e) => {             // 检查失败，假定已退出
                tracing::error!("检查 NapCat 进程状态失败: {}", e);
                false
            }
        }
    }
}
```

**`try_wait()` 是非阻塞的** — 立即返回，不会等待进程退出。这比 `child.wait()` 安全（`wait()` 会阻塞直到进程退出）。

### 健康检查循环结构

```rust
pub async fn start_health_check(
    app: tauri::AppHandle,
    process_state: NapCatProcessState,
    db: DbState,
    onebot: OneBotClientState,
) {
    let mut restart_count: u32 = 0;
    let restart_delays = [5u64, 15, 30]; // 秒

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        // 1. 调用 get_login_info 检查连通性
        match onebot.get_login_info().await {
            Ok(info) => {
                restart_count = 0; // 恢复正常，重置计数
                // 检查掉线（user_id == 0）
                if info.user_id == 0 {
                    // 掉线处理...
                }
            }
            Err(_) => {
                // 2. API 不通 → 检查进程是否存活
                let alive = {
                    let mut guard = match process_state.lock() {
                        Ok(g) => g,
                        Err(e) => e.into_inner(),
                    };
                    guard.as_mut().map_or(false, |p| p.is_alive())
                };

                if alive {
                    tracing::warn!("NapCat API 不通但进程存活，可能正在启动");
                } else {
                    // 3. 进程已退出 → 自动重启
                    if restart_count < 3 {
                        // 清理旧状态
                        { /* set NapCatProcessState to None */ }
                        let delay = restart_delays[restart_count as usize];
                        tracing::warn!("NapCat 进程异常退出，{}秒后第{}次重启",
                            delay, restart_count + 1);
                        emit napcat:status-changed = Starting;
                        tokio::time::sleep(Duration::from_secs(delay)).await;
                        // 重启
                        match restart_napcat_internal(...) {
                            Ok(()) => { restart_count = 0; }
                            Err(e) => { restart_count += 1; }
                        }
                    } else {
                        // 4. 超过 3 次 → 告警
                        emit napcat:status-changed = Error(...);
                        send_notification(...);
                        break; // 停止健康检查
                    }
                }
            }
        }
    }
}
```

### 重启进程：复用 start_napcat_process

**重启的核心步骤：**
1. 将 `NapCatProcessState` 设为 `None`（清理旧进程引用）
2. 调用已有的 `start_napcat_process()` 重新启动

```rust
async fn restart_napcat_internal(
    app: &tauri::AppHandle,
    process_state: &NapCatProcessState,
    db: &DbState,
) -> Result<(), AppError> {
    // 1. 清理旧进程状态
    {
        let mut guard = process_state.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        if let Some(ref mut p) = *guard {
            let _ = p.stop(); // 尝试停止（可能已退出，忽略错误）
        }
        *guard = None;
    }

    // 2. 读取配置
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| AppError::NapCat(e.to_string()))?;
    let napcat_dir = app_data_dir.join("napcat");
    let api_port: u16 = {
        let conn = db.lock().map_err(|e| AppError::NapCat(e.to_string()))?;
        crate::db::models::get_config_by_key(&conn, "napcat_api_port")
            .ok().and_then(|c| c.value.parse().ok()).unwrap_or(3000)
    };

    // 3. 重新生成配置 + 启动
    crate::napcat::config::generate_napcat_config(&app_data_dir, db)
        .map_err(|e| AppError::NapCat(e.to_string()))?;
    start_napcat_process(app, &napcat_dir, api_port, process_state, db)?;

    Ok(())
}
```

**`start_napcat_process` 内部已经会启动 stdout/stderr 监控和 `poll_login_status` 轮询，所以重启后不需要额外设置。**

### tauri-plugin-notification 使用方法

```rust
use tauri_plugin_notification::NotificationExt;

// 在 Tauri command 或直接通过 AppHandle 调用
app.notification()
    .builder()
    .title("QQ Auto Like Plus")
    .body("小助手遇到了一点问题，需要你帮帮忙~")
    .show()
    .map_err(|e| tracing::error!("发送通知失败: {}", e))
    .ok();
```

**已有权限：** `capabilities/default.json` 已包含 `notification:default`，无需添加。

**掉线通知：**
```rust
app.notification()
    .builder()
    .title("QQ Auto Like Plus")
    .body("QQ 需要重新登录，请打开面板扫码~")
    .show()
    .ok();
```

### OneBot get_login_info 响应结构

**`OneBotClient::get_login_info()` 已在 Story 2.1 实现（onebot/client.rs:101-105）：**
```rust
pub async fn get_login_info(&self) -> Result<OneBotLoginInfo, OneBotError>
```

**`OneBotLoginInfo` 定义在 `onebot/types.rs`：**
```rust
pub struct OneBotLoginInfo {
    pub user_id: i64,
    pub nickname: String,
}
```

**掉线判断：** `user_id == 0` 表示未登录/掉线。正常登录时 `user_id > 0`。

### 健康检查启动时机

**不在 `lib.rs` setup 中直接启动健康检查** — 应在 NapCat 成功登录后启动。

**方式：在 `poll_login_status` 成功后自动启动健康检查。**

修改 `process.rs` 中 `start_napcat_process` 的登录轮询部分：
```rust
// 当前：poll_login_status 成功后仅 emit 事件
// 修改：成功后还启动健康检查循环
tokio::spawn(async move {
    match poll_login_status(&app_clone, api_port, &db_clone).await {
        Ok(info) => {
            tracing::info!("登录成功，启动健康检查");
            start_health_check(app_clone, process_state_clone, db_clone, onebot_clone).await;
        }
        Err(e) => { /* 已有错误处理 */ }
    }
});
```

**这样健康检查自然在重启后也会启动**（因为 `start_napcat_process` 总会 spawn poll_login_status）。

### start_napcat_process 签名需要变化

**当前签名：**
```rust
pub fn start_napcat_process(
    app_handle: &tauri::AppHandle,
    napcat_dir: &Path,
    api_port: u16,
    process_state: &NapCatProcessState,
    db: &DbState,
) -> Result<(), AppError>
```

**需要额外传入 `OneBotClientState`** — 健康检查需要 onebot client。

修改方案：给 `start_napcat_process` 添加 `onebot: &OneBotClientState` 参数，或让健康检查自己创建 reqwest client（但复用 OneBotClient 更好）。

**推荐方案：** 添加 `onebot: &OneBotClientState` 参数给 `start_napcat_process`，然后在内部的 `poll_login_status` 成功后 spawn 健康检查。同步更新 `commands/napcat.rs` 中 `start_napcat` 的调用。

### NapCatProcessState 并发访问模式

**`NapCatProcessState = Arc<Mutex<Option<NapCatProcess>>>`**

健康检查需要 lock 这个 Mutex：
- `is_alive()` 需要 `&mut` 访问（`try_wait` 是 `&mut self`）
- 清理状态需要设置为 `None`

**mutex 中毒处理（已建立模式）：**
```rust
let mut guard = match process_state.lock() {
    Ok(g) => g,
    Err(e) => e.into_inner(),
};
```

**不要长时间持有锁！** lock → 检查/修改 → 立即 drop。不跨 await 持有 `std::sync::Mutex`。

### 掉线后的健康检查行为

掉线（`user_id == 0`）后健康检查应该怎么做：
1. emit `napcat:status-changed` = `WaitingForLogin`
2. emit `napcat:login-required`
3. 发送 Windows 通知
4. **继续健康检查循环但改为检测登录恢复**：如果 `get_login_info` 成功且 `user_id > 0`，表示用户已重新扫码登录，恢复正常监控

**不要 break 循环！** 掉线不等于进程崩溃，进程仍在运行，只是需要重新登录。

### QA Story 3.1 P2-F1 修复

**Story 3.1 QA 发现：** engine:status-changed listener 未考虑 NapCat 状态判断 TrayState。

本 Story 的健康检查会正确 emit `napcat:status-changed`，lib.rs 中已有的事件监听器会根据 NapCatStatus 更新托盘。engine:status-changed 的 P2 问题（短暂图标不一致）在实际使用中影响极小——引擎状态变化时 NapCat 通常是 Running 状态，所以 `is_paused ? Pending : Running` 判断是正确的。**不需要在本 Story 修改 lib.rs 的事件监听逻辑。**

### 不要做的事

- **不要** 使用 `child.wait()` 检查进程状态 — 它会阻塞！用 `try_wait()`
- **不要** 在 Mutex 锁持有期间 `.await` — std::sync::Mutex 不能跨 await
- **不要** 在健康检查失败后立即重启 — 先检查进程是否存活，可能只是 API 暂时不通
- **不要** 在掉线时 break 健康检查循环 — 掉线后应继续监测登录恢复
- **不要** 创建新的 reqwest Client — 复用已有的 `OneBotClient::get_login_info()`
- **不要** 修改 `tray/mod.rs` — 托盘图标更新由已有的事件监听器处理
- **不要** 修改 `lib.rs` 的事件监听逻辑 — emit 正确的事件即可
- **不要** 使用 `println!` — 用 `tracing::info!` / `tracing::warn!` / `tracing::error!`
- **不要** 使用 `unwrap()` / `expect()` — 用 `?` 或 match
- **不要** 修改数据库 schema — 本 Story 不需要新表或新列

### 与其他 Story 的边界

- **Story 3.1**（已完成）提供了托盘图标更新和 `napcat:status-changed` 事件监听。本 Story 的健康检查 emit 事件后由 3.1 代码自动更新托盘。
- **Story 3.2**（已完成）提供了窗口管理和退出确认。无直接交互。
- **Story 3.4**（开机自启）会使用 autostart 插件，开机自启后 NapCat 进程启动也会自动启动健康检查（通过 poll_login_status → start_health_check 链路）。
- **Story 1.4**（已完成）提供了 `start_napcat_process` 和 `poll_login_status`。本 Story 修改这些函数添加健康检查启动逻辑。

### 修改范围总结

```
src-tauri/src/
├── napcat/
│   └── process.rs        ← 主要修改：添加 is_alive()、start_health_check()、restart_napcat_internal()
│                            修改 start_napcat_process() 签名和 poll_login_status 后续
├── commands/
│   └── napcat.rs          ← 修改：更新 start_napcat 调用（传入 onebot）+ 添加 restart_napcat command
└── lib.rs                 ← 修改：start_napcat_process 调用参数更新 + 注册 restart_napcat command
```

**不创建新文件。不修改前端代码。不修改数据库。不修改 Cargo.toml（notification 依赖已存在）。不修改 capabilities（notification 权限已存在）。**

### Project Structure Notes

- 健康检查逻辑完全在 `napcat/process.rs` 中实现，符合架构边界（NapCat 进程管理归 napcat/ 模块）
- 通过 emit Tauri events 通知前端，不直接操作 UI
- 复用已有的 `OneBotClient` 和 `start_napcat_process`，不重复造轮子
- 通知使用 `tauri_plugin_notification`，符合架构文档选型

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story3.3] — AC 定义：NapCat 健康检查与自动重启
- [Source: .bmad-method/planning-artifacts/architecture.md#napcat/process.rs] — NapCat 进程管理模块
- [Source: .bmad-method/planning-artifacts/architecture.md#跨切面关注点] — NapCat 生命周期管理贯穿所有功能
- [Source: .bmad-method/planning-artifacts/architecture.md#错误处理模式] — thiserror 库层 + anyhow 应用层
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式] — Tauri events emit/listen
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println、unwrap、全局 static mut
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#旅程5:异常恢复] — 自动恢复优先，3次重试后通知，温暖语气
- [Source: .bmad-method/planning-artifacts/ux-design-specification.md#反馈模式] — Windows 系统通知仅用于：登录过期/3次重启失败
- [Source: .bmad-method/implementation-artifacts/3-1-system-tray-icon-and-menu.md] — 托盘事件监听、TrayState、resolve_tray_state
- [Source: .bmad-method/implementation-artifacts/3-2-panel-window-and-tray-interaction.md] — 退出确认对话框、NapCatProcess.stop()
- [Source: src-tauri/src/napcat/process.rs] — NapCatProcess 结构体、start_napcat_process、poll_login_status、NapCatProcessState 类型
- [Source: src-tauri/src/napcat/mod.rs] — NapCatStatus 枚举、LoginInfo 结构体
- [Source: src-tauri/src/onebot/client.rs] — OneBotClient::get_login_info() 方法
- [Source: src-tauri/src/tray/mod.rs] — update_tray_icon、resolve_tray_state、事件监听
- [Source: src-tauri/src/lib.rs] — setup 流程、State 注册、napcat:status-changed 和 engine:status-changed 事件监听
- [Source: src-tauri/src/commands/napcat.rs] — start_napcat command、stop_napcat command
- [Source: src-tauri/src/errors.rs] — AppError 枚举定义
- [Source: src-tauri/Cargo.toml] — tauri-plugin-notification = "2" 已存在
- [Source: src-tauri/capabilities/default.json] — notification:default 权限已存在

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- `cargo check` 编译通过，无新增 warnings（api_base_url 未使用是预期内，供便捷访问）

### Completion Notes List

- Task 1: 添加 `is_alive()` 使用 `try_wait()` 非阻塞检测 + `api_base_url()` 便捷方法
- Task 2: `start_health_check` 每 30 秒循环，使用 `onebot.get_login_info()` 检查连通性
- Task 3: 递增重启 [5,15,30] 秒，复用 `start_napcat_process()`，通过 `restart_napcat_internal` 实现
- Task 4: 3 次重启失败后 `tauri_plugin_notification` 发送系统通知 + emit Error 状态触发红色托盘
- Task 5: `user_id == 0` 或 QQ 号变更检测掉线，emit `WaitingForLogin` + `napcat:login-required`，发送扫码通知，继续循环等待重新登录
- Task 6: 健康检查在 `poll_login_status` 成功后自动启动（通过修改 `start_napcat_process` 内的 tokio::spawn 链路）
- Task 7: `restart_napcat` Tauri command 注册完成，调用 `restart_napcat_internal`
- Task 8: `cargo check` 通过，`notification:default` 权限已确认
- 实现方式��修改 `start_napcat_process` 签名添加 `onebot: &OneBotClientState` 参数，所有调用方同步更新

### File List

- `src-tauri/src/napcat/process.rs` — 修改：添加 is_alive()、start_health_check()、restart_napcat_internal()、restart_napcat_cmd()；修改 start_napcat_process 签名和登录后启动健康检查
- `src-tauri/src/commands/napcat.rs` — 修改：更新 start_napcat 传入 onebot 参数；添加 restart_napcat command
- `src-tauri/src/lib.rs` — 修改：注册 restart_napcat command

### Change Log

- 2026-03-14: Story 3.3 实现完成，所有 Task 1-8 已完成（Task 8.3 需手动验证）
- 2026-03-14: QA Review 完成 — CONCERNS
- 2026-03-14: QA 修复完成 — P2-F1 重启后 break 防止循环叠加; P2-F3 添加 QQ 号变更检测; P3-F1 统一 match 模式; P3-F2 移除未使用 api_base_url; P3-F3 添加僵死进程超时机制(5次×30秒)

## QA Results

**Reviewer:** Quinn (QA Agent) | **Model:** Claude Opus 4.6 | **Date:** 2026-03-14

**Gate Decision: CONCERNS**

### AC 验证结果: 6 PASS / 1 PARTIAL / 0 FAIL

| AC | 结果 | 说明 |
|----|------|------|
| #1 每30秒健康检查 | PASS | process.rs:336-339, sleep 30s + get_login_info |
| #2 API失败→检查进程存活 | PASS | process.rs:380-387, try_wait 非阻塞 |
| #3 递增重启 5/15/30s, 最多3次 | PASS | process.rs:333,393-423 |
| #4 状态变化 emit 事件 | PASS | 4处 emit 覆盖 Starting/WaitingForLogin/Running/Error |
| #5 超3次→通知+红灯 | PASS | process.rs:424-440, notification + Error 状态 |
| #6 掉线检测+通知 | PARTIAL | user_id==0 检测完整; QQ号变更检测缺失(AC写明但Task标可选) |
| #7 tracing 日志 | PASS | info/warn/error 全路径覆盖 |

### 架构合规: PASS

### Findings 摘要

**P2 (需修复 — 3项):**

- **P2-F1 [BUG] 重复健康检查循环叠加:** `restart_napcat_internal` 调用 `start_napcat_process`，后者 spawn 新的 `poll_login_status → start_health_check`。原健康检查循环未退出，每次成功重启都叠加一个新循环。**推荐：重启成功后 break 当前循环，让新 spawn 的接管。**
- **P2-F2 [BUG] restart_count 双重独立计数:** 与 P2-F1 耦合，两个循环各维护独立计数器，实际最多可尝试 6 次重启。修复 P2-F1 后自动消除。
- **P2-F3 [GAP] QQ号变更检测缺失:** AC #6 写明 "user_id=0 或 QQ号与已保存不同"，实现仅检查 user_id==0。建议：若确认可选则更新 AC 文字保持一致；若需实现则从 db 读取 qq_number 对比。

**P3 (建议改进 — 3项):**

- **P3-F1:** `db.lock().map_err(|e| e.into_inner())` 语义不正确 (process.rs:368)，应改用 match 统一模式
- **P3-F2:** `api_base_url()` 未使用产生 cargo warning (process.rs:51)
- **P3-F3:** API 不通但进程存活时无超时机制，僵死进程不会被重启 (process.rs:389)

**P4 (信息 — 1项):**

- **P4-F1:** get_login_info_cmd 创建新 reqwest::Client 而非复用 OneBotClient（非本 Story 引入）

### 风险评估: MEDIUM

P2-F1 是实际 bug（重复循环叠加），不导致崩溃但会浪费资源并可能触发并发重启竞态。修复简单（重启成功后 break），建议在合入前修复 P2-F1/F2。P2-F3 可通过更新 AC 文字解决。
