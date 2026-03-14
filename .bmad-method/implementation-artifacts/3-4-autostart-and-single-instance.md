# Story 3.4: 开机自启与单实例

Status: Done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 用户,
I want 应用随开机自动启动,
so that 我不用每次手动打开。

## Acceptance Criteria

1. **Given** 用户在设置中开启"开机自启" **When** Windows 系统启动 **Then** 应用通过 `tauri-plugin-autostart` 自动启动
2. **Given** 应用通过开机自启启动 **When** 应用窗口初始化完成 **Then** 面板窗口直接最小化到系统托盘（不弹出面板窗口）
3. **Given** 应用通过开机自启启动 **When** NapCat 环境已安装且配置完成 **Then** 自动启动 NapCat 并恢复上次运行状态（暂停/运行）
4. **Given** 应用已在运行 **When** 用户二次打开应用（双击快捷方式等） **Then** `tauri-plugin-single-instance` 防止多开，激活已有实例窗口并聚焦
5. **Given** 用户进入设置页 **When** 操作"开机自启"开关 **Then** 通过 Tauri command 控制 autostart 插件启用/禁用，开关状态持久化到 config 表

## Tasks / Subtasks

- [x] Task 1: 修改 autostart 插件初始化，添加 `--autostarted` 启动参数标记 (AC: #1, #2)
  - [x] 1.1 修改 `lib.rs` 中 `tauri_plugin_autostart::init()` 的第二个参数从 `None` 改为 `Some(vec!["--autostarted"])`
  - [x] 1.2 在 `setup` 闭包中检测 `std::env::args()` 是否包含 `--autostarted` 标志
  - [x] 1.3 如果检测到 `--autostarted`，在 setup 末尾将 main 窗口隐藏：`app.get_webview_window("main").map(|w| w.hide())`
  - [x] 1.4 正常启动（无 `--autostarted`）不做任何窗口隐藏操作，行为不变

- [x] Task 2: 添加 autostart Tauri commands (AC: #5)
  - [x] 2.1 在 `commands/settings.rs` 中添加 `enable_autostart` command：调用 `app.autolaunch().enable()` 并将 config 表 `auto_start` 设为 `"true"`
  - [x] 2.2 在 `commands/settings.rs` 中添加 `disable_autostart` command：调用 `app.autolaunch().disable()` 并将 config 表 `auto_start` 设为 `"false"`
  - [x] 2.3 在 `commands/settings.rs` 中添加 `is_autostart_enabled` command：调用 `app.autolaunch().is_enabled()` 返回 `bool`
  - [x] 2.4 所有 command 需 `use tauri_plugin_autostart::ManagerExt;` 引入 `autolaunch()` 扩展方法
  - [x] 2.5 在 `lib.rs` 的 `invoke_handler` 中注册三个新 command

- [x] Task 3: 增强 single-instance 回调 (AC: #4)
  - [x] 3.1 修改 `lib.rs` 中 `tauri_plugin_single_instance::init` 的回调：除了 `set_focus()` 外，先调用 `window.show()` 再调用 `window.unminimize()`，确保隐藏的窗口能被正确激活
  - [x] 3.2 回调中增加 tracing 日志：`tracing::info!("检测到重复实例，激活已有窗口")`

- [x] Task 4: 确保开机自启后自动恢复运行状态 (AC: #3)
  - [x] 4.1 确认当前 `lib.rs` setup 流程中调度器（LikeScheduler）在启动时已从 config 读取 `is_paused` 状态并恢复 — 验证此逻辑已存在
  - [x] 4.2 如果调度器未自动恢复暂停状态，需在 scheduler `start()` 中读取 config 表 `engine_paused` 键并恢复
  - [x] 4.3 **注意**：NapCat 自动启动不在本 Story 范围 — 当前架构中 NapCat 由前端触发 `invoke("start_napcat")`，开机自启后前端加载时应自动调用（Story 4.x 前端面板负责）。本 Story 仅确保后端 autostart 基础设施就位

- [x] Task 5: 更新 capabilities 权限 (AC: #5)
  - [x] 5.1 检查 `capabilities/default.json` — 当前已有 `autostart:default`，该权限包含 `allow-enable`、`allow-disable`、`allow-is-enabled`，**无需修改**
  - [x] 5.2 验证 `autostart:default` 权限覆盖了所有三个 command 的前端调用需求

- [x] Task 6: 构建验证 (AC: #1-#5)
  - [x] 6.1 `cargo check` 编译通过，无新增 warnings
  - [x] 6.2 确认 `--autostarted` 参数在正常 `pnpm tauri dev` 启动时不被触发（不含该参数）
  - [x] 6.3 确认三个新 command 已注册到 invoke_handler

## Dev Notes

### 核心挑战：最小改动实现开机自启 + 单实例增强

本 Story 改动量极小。autostart 和 single-instance 插件已在 Story 1.1 完成初始化，Cargo.toml 和 capabilities 均已配置。核心工作是：
1. 给 autostart 加启动参数标记以区分自启/手动启动
2. 添加三个 Tauri command 控制开关
3. 增强 single-instance 窗口激活逻辑

### 已存在的基础设施（不要重复！）

**lib.rs 中已有：**
```rust
// autostart 插件已初始化（line 15-18）
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,  // ← 需要改为 Some(vec!["--autostarted"])
))

// single-instance 插件已初始化（line 22-26）
.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_focus();  // ← 需要增强：先 show + unminimize 再 focus
    }
}))
```

**Cargo.toml 已有：**
- `tauri-plugin-autostart = "2"` ✅
- `tauri-plugin-single-instance = "2"` ✅

**capabilities/default.json 已有：**
- `"autostart:default"` ✅ — 包含 enable/disable/is_enabled 权限

**config 表已有默认值：**
- `auto_start` = `'false'` ✅（001_init.sql line 24）

### autostart 插件 Rust API（已验证最新 v2）

```rust
use tauri_plugin_autostart::ManagerExt;

// 获取 autolaunch manager
let autostart_manager = app.autolaunch();

// 启用开机自启
autostart_manager.enable().map_err(|e| e.to_string())?;

// 禁用开机自启
autostart_manager.disable().map_err(|e| e.to_string())?;

// 检查是否已启用
let enabled: bool = autostart_manager.is_enabled().map_err(|e| e.to_string())?;
```

**`ManagerExt` trait** 为 `AppHandle` 和 `App` 提供 `.autolaunch()` 方法，返回 `AutoLaunchManager`。

### autostart 启动参数检测

```rust
// 在 lib.rs setup 闭包中：
let is_autostarted = std::env::args().any(|arg| arg == "--autostarted");
if is_autostarted {
    tracing::info!("检测到开机自启模式，隐藏主窗口");
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}
```

**为什么用 `std::env::args()` 而不是 single-instance 的 `_args`？**
- `--autostarted` 是 autostart 插件在注册表中写入的额外参数
- Windows 开机自启时，autostart 插件将 exe 路径 + 额外参数写入 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- `std::env::args()` 在 setup 阶段就能读取，时机最早

### single-instance 窗口激活增强

当前回调只 `set_focus()`，但如果窗口被 `hide()` 了（关闭按钮/自启隐藏），`set_focus()` 无效。需要完整激活链：

```rust
.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
    tracing::info!("检测到重复实例，激活已有窗口");
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();        // 从隐藏恢复可见
        let _ = window.unminimize();  // 从最小化恢复
        let _ = window.set_focus();   // 聚焦
    }
}))
```

**顺序很重要**：`show()` → `unminimize()` → `set_focus()`。如果窗口是 hidden 状态，必须先 show 才能 focus。

### Tauri commands 实现模板

```rust
// commands/settings.rs 中新增：
use tauri_plugin_autostart::ManagerExt;

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
```

### invoke_handler 注册

```rust
// lib.rs invoke_handler 中新增：
commands::settings::enable_autostart,
commands::settings::disable_autostart,
commands::settings::is_autostart_enabled,
```

### 暂停状态恢复（AC #3）

当前 `engine/scheduler.rs` 的 `LikeScheduler::start()` 方法需要确认是否从 config 读取 `engine_paused` 状态。如果引擎状态是内存级的（`AtomicBool`），那么每次启动默认是 running 状态，这也是合理行为 — 开机自启后默认运行，用户上次暂停只是临时操作。

**推荐处理**：开机自启后默认恢复为运行状态（不持久化暂停）。这是合理的 UX — 用户设置了自启就是想让它自动工作。如果 PRD 明确要求持久化暂停状态，则需要在 `setup` 中读取 config 并设置 `is_paused`。

### NapCat 自动启动边界说明

**本 Story 不负责开机自启后自动启动 NapCat 进程。** 当前架构中 NapCat 启动由前端触发：
- 前端加载 → Dashboard 页面 → 检测 NapCat 状态 → 调用 `invoke("start_napcat")`
- 开机自启后窗口虽然隐藏，但 WebView 仍会加载前端代码，前端可以在后台执行初始化

这属于 Story 4.x（管理面板）的范围。本 Story 仅确保：
1. autostart 基础设施正确
2. 窗口隐藏行为正确
3. 单实例保护正确

### 不要做的事

- **不要** 修改 `Cargo.toml` — autostart 和 single-instance 依赖已存在
- **不要** 修改 `capabilities/default.json` — `autostart:default` 已包含所有需要的权限
- **不要** 修改数据库 schema — `auto_start` config 键已在 001_init.sql 默认值中
- **不要** 实现前端 UI — 设置页面是 Story 4.3 的范围
- **不要** 在后端自动启动 NapCat — 由前端在加载时触发
- **不要** 使用 `println!` — 用 `tracing::info!`
- **不要** 使用 `unwrap()` / `expect()` — 用 `?` 或 `.map_err()`
- **不要** 创建新文件 — 所有修改在已有文件中完成

### 与其他 Story 的边界

- **Story 3.1**（已完成）：提供了系统托盘。开机自启后窗口隐藏，托盘图标仍正常显示。
- **Story 3.2**（已完成）：提供了窗口管理（关闭→隐藏）。本 Story 的 autostart 隐藏使用相同机制 `window.hide()`。
- **Story 3.3**（已完成）：提供了 NapCat 健康检查。NapCat 启动后健康检查自动生效（通过 poll_login_status → start_health_check 链路）。
- **Story 4.3**（未开始）：设置面板将提供 UI 开关调用本 Story 的 `enable_autostart` / `disable_autostart` commands。

### 修改范围总结

```
src-tauri/src/
├── lib.rs               ← 修改：autostart init 添加 --autostarted 参数
│                           修改：setup 中检测自启标志并隐藏窗口
│                           修改：single-instance 回调增强（show + unminimize + focus）
│                           修改：invoke_handler 注册 3 个新 command
└── commands/
    └── settings.rs      ← 修改：添加 enable_autostart、disable_autostart、is_autostart_enabled commands
```

**不创建新文件。不修改前端代码。不修改数据库。不修改 Cargo.toml。不修改 capabilities。**

### Project Structure Notes

- autostart commands 放在 `commands/settings.rs` 中，符合架构边界（开机自启属于系统设置）
- 与 `update_config` command 同文件，保持设置相关操作集中
- autostart 状态同时通过插件 API 和 config 表双重持久化，确保前端查询和系统注册表一致

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story3.4] — AC 定义：开机自启与单实例
- [Source: .bmad-method/planning-artifacts/architecture.md#基础设施与部署] — 单实例: tauri-plugin-single-instance
- [Source: .bmad-method/planning-artifacts/architecture.md#需求到文件的精确映射] — US-012 开机自启: lib.rs (tauri-plugin-autostart), pages/Settings.tsx
- [Source: .bmad-method/planning-artifacts/architecture.md#跨切面关注点映射] — 系统集成: US-005 托盘, US-009 进程管理, US-012 自启
- [Source: .bmad-method/planning-artifacts/architecture.md#命名规范] — Tauri command snake_case
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式] — Tauri IPC invoke/emit
- [Source: .bmad-method/planning-artifacts/architecture.md#错误处理模式] — command 层 Result<T, String>
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println、unwrap
- [Source: .bmad-method/implementation-artifacts/3-3-napcat-health-check-and-restart.md#与其他Story的边界] — Story 3.4 使用 autostart 插件，NapCat 启动后健康检查自动生效
- [Source: .bmad-method/implementation-artifacts/3-2-panel-window-and-tray-interaction.md] — 窗口关闭→隐藏机制
- [Source: .bmad-method/implementation-artifacts/3-1-system-tray-icon-and-menu.md] — 托盘图标、TrayState
- [Source: src-tauri/src/lib.rs:15-18] — autostart 插件当前初始化代码
- [Source: src-tauri/src/lib.rs:22-26] — single-instance 插件当前初始化代码
- [Source: src-tauri/src/commands/settings.rs] — 现有 get_config、update_config commands
- [Source: src-tauri/src/db/models.rs] — upsert_config() 函数
- [Source: src-tauri/Cargo.toml:17-21] — tauri-plugin-autostart = "2", tauri-plugin-single-instance = "2"
- [Source: src-tauri/capabilities/default.json:10] — autostart:default 权限
- [Source: src-tauri/migrations/001_init.sql:24] — auto_start config 默认值 'false'
- [Source: https://v2.tauri.app/plugin/autostart/] — tauri-plugin-autostart v2 官方文档，ManagerExt trait API

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

无 — 所有任务一次通过，无调试日志。

### Completion Notes List

- Task 4.1/4.2: 调度器 `start()` 已在 `scheduler.rs:72-80` 从 config 读取 `engine_paused` 并恢复暂停状态，无需额外修改
- Task 5.1/5.2: `capabilities/default.json` 已有 `autostart:default`，无需修改
- 无新增 warnings，所有 9 个 warnings 均为已有代码的未使用导入/函数

### File List

- `src-tauri/src/lib.rs` — 修改: autostart init 添加 `--autostarted` 参数, setup 中检测自启并隐藏窗口, single-instance 回调增强 (show+unminimize+focus+日志), invoke_handler 注册 3 个新 command
- `src-tauri/src/commands/settings.rs` — 修改: 添加 `enable_autostart`、`disable_autostart`、`is_autostart_enabled` 三个 Tauri commands, 添加 `use tauri_plugin_autostart::ManagerExt`

### Change Log

- autostart 插件参数从 `None` 改为 `Some(vec!["--autostarted"])` 以区分自启/手动启动
- setup 闭包末尾添加 `--autostarted` 检测逻辑，自启时隐藏主窗口
- single-instance 回调从单纯 `set_focus()` 增强为 `show()` → `unminimize()` → `set_focus()` 完整激活链
- 新增 3 个 Tauri commands: `enable_autostart`、`disable_autostart`、`is_autostart_enabled`

## QA Results

### Reviewer
Quinn (QA Agent) | Model: Claude Opus 4.6 | Date: 2026-03-14

### Gate Decision: PASS

### AC Verification

| AC | 描述 | 结果 | 证据 |
|---|---|---|---|
| #1 | autostart 通过 tauri-plugin-autostart 启动 | PASS | lib.rs:15-18 init with Some(vec!["--autostarted"]); settings.rs:59-67 enable command |
| #2 | 自启后窗口隐藏到托盘 | PASS | lib.rs:182-188 检测 --autostarted → window.hide() |
| #3 | 自启后恢复运行状态 | PASS | scheduler.rs:72-80 从 config 读取 engine_paused 恢复; NapCat 启动由前端负责（边界合理） |
| #4 | single-instance 防多开 + 激活窗口 | PASS | lib.rs:22-28 show→unminimize→set_focus 完整链 |
| #5 | commands 控制 autostart + 状态持久化 | PASS | settings.rs:59-85 三个 commands; lib.rs:215-217 注册; capabilities/default.json:10 权限覆盖 |

### Architecture Compliance: PASS

- 无 println/unwrap/expect（新代码），错误处理用 .map_err()
- 通过 Tauri State 注入 DB，线程安全
- 不创建新文件、不修改 Cargo.toml / capabilities / DB schema
- 复用已有 upsert_config 函数和插件初始化

### Findings

- **P3-F1 (consistency)**: enable/disable_autostart 修改 config 表后未 emit config:updated 事件，破坏 config 变更通知一致性。建议 Story 4.3 时补充。
- **P3-F2 (atomicity)**: enable_autostart 先写 OS 注册表再写 DB，若 DB 失败则状态不一致。概率极低，建议后续考虑反转顺序或添加回滚。
- **P4-F1 (pre-existing)**: 9 个 cargo warnings 全为已有代码，非本 Story 引入。

### Risk Assessment: LOW

改动量极小（2 文件、~40 行），逻辑简单，复用已有基础设施，cargo check 通过无新增 warnings。
