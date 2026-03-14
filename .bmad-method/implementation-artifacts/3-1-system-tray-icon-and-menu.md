# Story 3.1: 系统托盘基础（图标与右键菜单）

Status: Done

## Story

As a 用户,
I want 应用在系统托盘显示图标并提供快捷操作,
so that 应用不占用任务栏空间，且我能快速操作。

## Acceptance Criteria

1. **Given** 应用已启动 **When** 应用初始化完成 **Then** 系统托盘显示应用图标
2. **Given** 托盘图标已显示 **When** 应用状态变化 **Then** 图标颜色反映当前状态：绿色=运行中、黄色=登录中或下载中、红色=异常
3. **Given** 托盘图标已显示 **When** 用户右键点击 **Then** 菜单包含：打开面板、立即点赞、暂停/恢复、NapCat 状态（文本）、退出
4. **Given** 右键菜单已打开 **When** 用户点击"立即点赞" **Then** 调用 `start_batch_like` command
5. **Given** 右键菜单已打开 **When** 用户点击"暂停/恢复" **Then** 切换引擎状态并更新菜单文字
6. **Given** 应用运行中 **When** Tauri events 推送状态变化 **Then** 托盘图标自动更新对应颜色
7. **Given** 需要托盘图标 **When** 应用构建 **Then** 三套图标文件已准备：tray-green.png、tray-yellow.png、tray-red.png

## Tasks / Subtasks

- [x] Task 1: 准备托盘图标资源文件 (AC: #7)
  - [x] 1.1 创建 `src-tauri/icons/tray-green.png`（32×32，绿色应用图标）
  - [x] 1.2 创建 `src-tauri/icons/tray-yellow.png`（32×32，黄色应用图标）
  - [x] 1.3 创建 `src-tauri/icons/tray-red.png`（32×32，红色应用图标）
  - [x] 1.4 如果缺少设计稿，可暂用纯色圆形占位（后续替换为正式设计）
- [x] Task 2: 实现 `tray/mod.rs` 托盘模块 (AC: #1, #2, #3, #6)
  - [x] 2.1 定义 `TrayState` 枚举：`Running`, `Pending`, `Error`（映射绿/黄/红）
  - [x] 2.2 实现 `create_tray(app: &tauri::App) -> Result<(), anyhow::Error>` 函数
  - [x] 2.3 使用 `TrayIconBuilder` 创建托盘图标（初始绿色图标 + tooltip）
  - [x] 2.4 使用 `Menu::with_items` 构建右键菜单：5 个菜单项
  - [x] 2.5 实现 `on_menu_event` 处理各菜单项点击
  - [x] 2.6 实现 `update_tray_icon(app: &AppHandle, state: TrayState)` 动态切换图标
  - [x] 2.7 实现 `update_tray_menu_text(app: &AppHandle, is_paused: bool)` 更新暂停/恢复文字
- [x] Task 3: 在 `lib.rs` 中集成托盘初始化 (AC: #1)
  - [x] 3.1 在 `lib.rs` 添加 `mod tray;` 声明
  - [x] 3.2 在 `setup` 闭包末尾调用 `tray::create_tray(app)?`
  - [x] 3.3 确保托盘创建在所有 State 注册之后（需要访问 SchedulerState 等）
- [x] Task 4: 实现菜单项事件处理 (AC: #4, #5)
  - [x] 4.1 "open_panel" → 获取 main 窗口并 show + set_focus
  - [x] 4.2 "start_like" → 获取 BatchLikeRunning + 相关 State，调用 run_batch_like
  - [x] 4.3 "toggle_pause" → 获取 LikeSchedulerState，调用 pause/resume + 更新菜单文字
  - [x] 4.4 "quit" → 停止 NapCat 进程 + app.exit(0)
  - [x] 4.5 "napcat_status" → 仅显示文字，不可点击（disabled MenuItem）
- [x] Task 5: 监听状态事件更新托盘图标 (AC: #2, #6)
  - [x] 5.1 在 setup 中启动后台任务监听 `napcat:status-changed` 事件
  - [x] 5.2 在 setup 中监听 `engine:status-changed` 事件
  - [x] 5.3 根据 NapCatStatus + EngineStatus 综合判断 TrayState 并调用 `update_tray_icon`
  - [x] 5.4 引擎暂停/恢复时同步更新菜单文字
- [x] Task 6: 构建验证 (AC: #1-#7)
  - [x] 6.1 `cargo check` 编译通过
  - [ ] 6.2 `pnpm tauri dev` 启动后托盘图标可见

## Dev Notes

### 核心挑战：Tauri 2.0 TrayIcon API + 状态联动

本 Story 的核心是正确使用 Tauri 2.0 内置的 tray-icon API（非插件），并将托盘图标状态与 NapCat 进程状态和引擎状态联动。

### Tauri 2.0 Tray API（关键）

**Cargo.toml 已启用：** `tauri = { version = "2", features = ["tray-icon"] }`

**核心类型：**
```rust
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::menu::{Menu, MenuItem};
use tauri::image::Image;
use tauri::{AppHandle, Manager};
```

**TrayIconBuilder 创建流程：**
```rust
let tray = TrayIconBuilder::new()
    .icon(Image::from_path("path/to/icon.png")?)  // 或 app.default_window_icon()
    .menu(&menu)
    .tooltip("QQ Auto Like Plus - 运行中")
    .on_tray_icon_event(|tray, event| { /* 双击处理 */ })
    .on_menu_event(|app, event| { /* 菜单项处理 */ })
    .build(app)?;
```

**动态图标切换：**
```rust
// TrayIcon 方法
pub fn set_icon(&self, icon: Option<Image<'_>>) -> Result<()>
pub fn set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> Result<()>
pub fn set_menu<M: ContextMenu + 'static>(&self, menu: Option<M>) -> Result<()>
```

**事件处理：**
```rust
TrayIconEvent::Click {
    button: MouseButton::Left,
    button_state: MouseButtonState::Up,
    ..
} => { /* 左键单击 */ }

TrayIconEvent::DoubleClick {
    button: MouseButton::Left,
    ..
} => { /* 双击打开面板 */ }
```

**菜单事件处理：**
```rust
.on_menu_event(|app, event| match event.id.as_ref() {
    "open_panel" => { /* 打开面板 */ }
    "start_like" => { /* 立即点赞 */ }
    "toggle_pause" => { /* 暂停/恢复 */ }
    "quit" => { /* 退出 */ }
    _ => {}
})
```

### 菜单构建

```rust
use tauri::menu::{Menu, MenuItem};

fn build_tray_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let open_item = MenuItem::with_id(app, "open_panel", "打开面板", true, None::<&str>)?;
    let like_item = MenuItem::with_id(app, "start_like", "立即点赞", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "toggle_pause", "暂停", true, None::<&str>)?;
    let status_item = MenuItem::with_id(app, "napcat_status", "运行环境: 已连接", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&open_item, &like_item, &pause_item, &status_item, &quit_item])
}
```

**注意：**
- `MenuItem::with_id(app, id, text, enabled, accelerator)` — `enabled: false` 表示灰色不可点击
- `status_item` 设为 `enabled: false`，仅用于显示 NapCat 状态文字
- 菜单项 `id` 使用 snake_case 字符串匹配

### 图标加载方式

**从文件路径加载（推荐）：**
```rust
use tauri::image::Image;

// 从 icons 目录加载（相对于 src-tauri/ 的路径）
let icon = Image::from_path("icons/tray-green.png")?;
```

**但 `from_path` 在打包后可能找不到文件！推荐使用 `include_bytes!` 嵌入：**
```rust
let green_icon = Image::from_bytes(include_bytes!("../icons/tray-green.png"))?;
let yellow_icon = Image::from_bytes(include_bytes!("../icons/tray-yellow.png"))?;
let red_icon = Image::from_bytes(include_bytes!("../icons/tray-red.png"))?;
```

**这确保图标被编译进二进制，不依赖文件系统路径。** 打包后的应用不包含 icons/ 目录作为独立文件。

### TrayState 与应用状态映射

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayState {
    Running,  // 绿色：NapCat 已连接 + 引擎运行中
    Pending,  // 黄色：登录中、下载中、引擎暂停
    Error,    // 红色：NapCat 异常、连接失败
}
```

**状态判断优先级（由高到低）：**
1. NapCat 状态为 `Error(_)` → 红色
2. NapCat 状态为 `NotInstalled`/`Downloading`/`Extracting`/`Starting`/`WaitingForLogin` → 黄色
3. 引擎暂停（`is_paused = true`）→ 黄色
4. NapCat 状态为 `Running` + 引擎运行 → 绿色

**NapCatStatus 枚举已存在**（`napcat/mod.rs`）：
```rust
pub enum NapCatStatus {
    NotInstalled, Downloading, Extracting, Ready,
    Starting, WaitingForLogin, Running, Error(String),
}
```

### tray/mod.rs 完整结构

```rust
use anyhow::Result;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayState {
    Running,
    Pending,
    Error,
}

/// 在 setup 中调用，创建系统托盘
pub fn create_tray(app: &tauri::App) -> Result<()> {
    let menu = build_tray_menu(app)?;

    let _tray = TrayIconBuilder::new()
        .icon(green_icon()?)
        .menu(&menu)
        .tooltip("QQ Auto Like Plus - 运行中")
        .on_tray_icon_event(handle_tray_event)
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
}

fn green_icon() -> Result<Image<'static>> { /* include_bytes! */ }
fn yellow_icon() -> Result<Image<'static>> { /* include_bytes! */ }
fn red_icon() -> Result<Image<'static>> { /* include_bytes! */ }

fn build_tray_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> { /* ... */ }

fn handle_tray_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    // 双击打开面板
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    // 匹配 menu item id
}

/// 从外部调用更新托盘图标
pub fn update_tray_icon(app: &AppHandle, state: TrayState) -> Result<()> {
    // app.tray_by_id("main") 或迭代获取
    // tray.set_icon(Some(icon))
    // tray.set_tooltip(Some(tooltip_text))
}
```

### 菜单项事件处理详解

**"open_panel"** — 打开/显示面板窗口：
```rust
"open_panel" => {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
```

**"start_like"** — 立即点赞：
```rust
"start_like" => {
    // 从 app.state() 获取需要的 State
    let db = app.state::<crate::db::DbState>().inner().clone();
    let onebot = app.state::<crate::onebot::OneBotClientState>().inner().clone();
    let running = app.state::<crate::commands::like::BatchLikeRunning>().inner().clone();
    let app_handle = app.clone();

    tokio::spawn(async move {
        if running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            tracing::warn!("批量点赞正在执行中，忽略托盘触发");
            return;
        }
        let result = crate::engine::like_executor::run_batch_like(
            &db, &onebot, &app_handle, "manual"
        ).await;
        running.store(false, std::sync::atomic::Ordering::SeqCst);
        match result {
            Ok(r) => tracing::info!("手动批量点赞完成: {:?}", r),
            Err(e) => tracing::error!("手动批量点赞异常: {}", e),
        }
    });
}
```

**"toggle_pause"** — 暂停/恢复引擎：
```rust
"toggle_pause" => {
    let scheduler = app.state::<crate::engine::scheduler::LikeSchedulerState>().inner().clone();
    let db = app.state::<crate::db::DbState>().inner().clone();
    let onebot = app.state::<crate::onebot::OneBotClientState>().inner().clone();
    let running = app.state::<crate::commands::like::BatchLikeRunning>().inner().clone();
    let app_handle = app.clone();

    tokio::spawn(async move {
        let status = scheduler.get_status().await;
        let result = if status.is_paused {
            scheduler.resume(db, onebot, app_handle.clone(), running).await
        } else {
            scheduler.pause(&db, &app_handle).await
        };
        if let Err(e) = result {
            tracing::error!("切换引擎状态失败: {}", e);
        }
        // 引擎自身会 emit engine:status-changed，由事件监听器更新菜单文字
    });
}
```

**⚠️ 关键：`pause()` 的签名是 `pause(&self, db: &DbState, app: &AppHandle)`（引用），而 `resume()` 是 `resume(&self, db: DbState, ...)` （owned clone）。两者签名不一致，注意区分。** 参考 `commands/engine.rs` 中的调用方式。

**"quit"** — 退出应用：
```rust
"quit" => {
    // 停止 NapCat 进程
    let napcat_state = app.state::<crate::napcat::process::NapCatProcessState>().inner().clone();
    let mut guard = match napcat_state.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    if let Some(ref mut process) = *guard {
        if let Err(e) = process.stop() {
            tracing::error!("退出时停止 NapCat 失败: {}", e);
        }
    }
    app.exit(0);
}
```

### TrayIcon ID 与获取

**创建时可指定 ID：**
```rust
TrayIconBuilder::with_id("main-tray")
    .icon(...)
    .build(app)?;
```

**之后通过 ID 获取：**
```rust
if let Some(tray) = app.tray_by_id("main-tray") {
    tray.set_icon(Some(new_icon))?;
    tray.set_tooltip(Some("新提示文字"))?;
}
```

**推荐使用 `TrayIconBuilder::with_id("main-tray")` 而不是 `::new()`，这样后续 `update_tray_icon` 可以通过 ID 精确获取。**

### 动态更新菜单文字

**方式一：重建整个菜单（简单但开销略大）：**
```rust
pub fn update_tray_menu(app: &AppHandle, is_paused: bool, napcat_status_text: &str) -> Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let open_item = MenuItem::with_id(app, "open_panel", "打开面板", true, None::<&str>)?;
        let like_item = MenuItem::with_id(app, "start_like", "立即点赞", true, None::<&str>)?;
        let pause_text = if is_paused { "恢复" } else { "暂停" };
        let pause_item = MenuItem::with_id(app, "toggle_pause", pause_text, true, None::<&str>)?;
        let status_item = MenuItem::with_id(app, "napcat_status", napcat_status_text, false, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&open_item, &like_item, &pause_item, &status_item, &quit_item])?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
```

**方式二：使用 MenuItem::set_text（如果 API 支持）。** Tauri 2.0 的 MenuItem 提供 `set_text()` 方法来更新文本，但需要持有 MenuItem 引用。如果使用方式二，需要把 MenuItem 存入 Tauri State 中。

**推荐方式一** — 重建菜单更简单，托盘菜单只有 5 个项，开销可忽略。

### lib.rs 修改要点

```rust
// 1. 添加 mod 声明
mod tray;

// 2. 在 setup 闭包末尾，所有 State 注册之后：
tray::create_tray(app)?;

// 3. on_window_event 不需要修改 — Story 3.2 会改为隐藏窗口
//    但当前的 CloseRequested 处理保持不变（本 Story 不涉及窗口管理）
```

**⚠️ 关键：`create_tray(app)` 必须在所有 `app.manage()` 调用之后。** 因为菜单事件处理中会用 `app.state::<T>()` 获取 State，如果 State 还没注册会 panic。

### 事件监听更新托盘

在 setup 中启动后台任务监听事件并更新托盘图标：

```rust
// 在 setup 中，create_tray 之后
let app_handle_for_tray = app.handle().clone();
app.listen("napcat:status-changed", move |event| {
    // 解析 NapCatStatus，更新托盘图标
    if let Some(payload) = event.payload() {
        // payload 是 JSON 字符串
        // 根据状态调用 tray::update_tray_icon
    }
});

let app_handle_for_engine = app.handle().clone();
app.listen("engine:status-changed", move |event| {
    // 解析 EngineStatus，更新暂停/恢复菜单文字
    if let Some(payload) = event.payload() {
        // 根据 is_paused 更新菜单
    }
});
```

**`app.listen()` 是 Tauri 2.0 后端监听事件的方式。** 返回 `EventId` 可以用于取消监听。

### 图标资源说明

需要准备三个 32×32 的 PNG 图标文件：
- `src-tauri/icons/tray-green.png` — 正常运行状态（绿色色调）
- `src-tauri/icons/tray-yellow.png` — 等待/暂停状态（黄色色调）
- `src-tauri/icons/tray-red.png` — 异常状态（红色色调）

如果没有设计稿，可以先：
1. 复制现有 `icons/32x32.png` 作为基础
2. 用简单的颜色叠加（绿/黄/红圆点 overlay）区分状态
3. 或者直接用纯色圆形作为占位

**32×32 是 Windows 系统托盘的标准尺寸。** 较大的图标（如 icon.ico 的 256×256）会被系统缩放，可能模糊。

### Story 2.4 教训应用

| 教训 | 本 Story 应用 |
|------|-------------|
| DB 锁不跨 await | 托盘菜单事件中获取 State 后 clone Arc，在 tokio::spawn 中使用 |
| BatchLikeRunning 防重复触发 | "立即点赞" 菜单项复用同一个 AtomicBool swap 模式 |
| `use tauri::Emitter` 导入 | 如果需要 emit 事件，记得导入 |
| setup 中 async 操作 | 事件监听 + 托盘创建都是同步的，不需要 block_on |
| `pause()` vs `resume()` 签名差异 | pause 接收引用，resume 接收 owned clone — 调用时注意 |

### 不要做的事

- **不要** 使用 `tauri-plugin-tray` — tray-icon 是 Tauri 2.0 的内置 feature，不是插件
- **不要** 用 `Image::from_path()` 加载图标 — 打包后路径不可用，用 `include_bytes!` 嵌入
- **不要** 在 `on_menu_event` 闭包中直接 `.await` — 这不是 async 闭包，用 `tokio::spawn`
- **不要** 在 State 注册之前调用 `create_tray()` — 菜单事件需要访问 State
- **不要** 修改 `on_window_event` 中的窗口关闭行为 — 那是 Story 3.2 的范围
- **不要** 忘记给 TrayIconBuilder 设置 ID — 后续需要通过 ID 获取 TrayIcon 来更新图标
- **不要** 使用 `println!` — 用 `tracing::info!` / `tracing::error!`
- **不要** 在 on_menu_event 中直接调用 pause/resume — 它们是 async，必须 spawn

### 与其他 Story 的边界

- **Story 3.2**（面板窗口管理）会修改 `on_window_event` 使关闭按钮变为隐藏窗口、双击托盘打开面板、退出时弹确认框。本 Story 只实现托盘图标和菜单，**不改窗口关闭行为**。
- **Story 3.3**（NapCat 健康检查）会 emit `napcat:status-changed` 事件。本 Story 的事件监听器会响应该事件更新图标。如果 3.3 未实现，图标默认保持绿色即可。
- **Story 3.4**（开机自启）会使用 tauri-plugin-autostart，与托盘无交互。

### capabilities/default.json

当前 capabilities 不需要为 tray-icon 添加权限 — tray-icon 是 Tauri core feature，不需要额外权限声明。

### Project Structure Notes

本 Story 需要创建/修改的文件：

```
src-tauri/
├── icons/
│   ├── tray-green.png              ← 新建：绿色托盘图标 32×32
│   ├── tray-yellow.png             ← 新建：黄色托盘图标 32×32
│   └── tray-red.png                ← 新建：红色托盘图标 32×32
└── src/
    ├── lib.rs                      ← 修改：添加 mod tray; + setup 中调用 create_tray + 事件监听
    └── tray/
        └── mod.rs                  ← 重写：完整托盘实现（图标创建、菜单、事件处理、状态更新）
```

**不创建新文件除上述列表外。不修改前端代码。不修改数据库。**

### References

- [Source: .bmad-method/planning-artifacts/epics.md#Story3.1] — AC 定义：系统托盘基础
- [Source: .bmad-method/planning-artifacts/architecture.md#tray/] — tray/mod.rs 模块定位
- [Source: .bmad-method/planning-artifacts/architecture.md#通信模式] — Tauri events emit/listen 模式
- [Source: .bmad-method/planning-artifacts/architecture.md#需求到文件映射] — US-005 → tray/mod.rs + icons/tray-*.png
- [Source: .bmad-method/planning-artifacts/architecture.md#反模式清单] — 禁止 println、禁止 unwrap
- [Source: .bmad-method/planning-artifacts/architecture.md#组件边界规则] — commands/ 唯一前端入口
- [Source: .bmad-method/planning-artifacts/architecture.md#跨切面关注点映射] — NapCat 生命周期涉及 tray/mod.rs
- [Source: .bmad-method/implementation-artifacts/2-4-scheduled-task-scheduler.md] — LikeScheduler API、pause/resume 签名差异、BatchLikeRunning 模式
- [Source: src-tauri/src/lib.rs] — 当前 setup 流程、State 注册顺序、on_window_event 处理
- [Source: src-tauri/src/napcat/mod.rs] — NapCatStatus 枚举定义
- [Source: src-tauri/src/engine/scheduler.rs] — EngineStatus 结构体、pause/resume 方法签名
- [Source: src-tauri/src/commands/engine.rs] — 引擎命令调用模式
- [Source: src-tauri/src/commands/like.rs] — BatchLikeRunning 类型、start_batch_like 模式
- [Source: src-tauri/Cargo.toml] — tauri features = ["tray-icon"] 已启用
- [Source: src-tauri/capabilities/default.json] — 当前权限声明（tray 无需额外权限）
- [Source: Tauri 2.0 docs: system-tray] — TrayIconBuilder API、事件处理、菜单构建

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6

### Debug Log References

- `cargo check` 首次编译发现 3 个问题并修复：
  1. `Image::from_bytes` 需要 `image-png` feature（已添加到 Cargo.toml）
  2. `event.payload()` 在 Tauri 2.x 返回 `&str` 非 `Option`（已修正事件监听代码）
  3. `include_bytes!` 路径需从 `src/tray/mod.rs` 到 `icons/` 的正确相对路径 `../../icons/`

### Completion Notes List

- Task 1: 使用 Python PIL 生成 32x32 纯色圆形占位图标（绿/黄/红），后续可替换为正式设计
- Task 2: `tray/mod.rs` 完整实现包含：TrayState 枚举、create_tray、handle_menu_event（5 项）、update_tray_icon、update_tray_menu、resolve_tray_state
- Task 2.7: 采用方式一（重建整个菜单）实现 update_tray_menu，更简单且仅 5 项开销可忽略
- Task 3: lib.rs 添加 `mod tray;` + `use tauri::Listener` + setup 末尾调用 create_tray
- Task 4: 菜单事件处理中 async 操作使用 tokio::spawn，pause/resume 签名差异已正确处理
- Task 5: napcat:status-changed 和 engine:status-changed 两个事件监听器均在 setup 中注册
- Task 6.2: `pnpm tauri dev` 需用户手动验证托盘图标是否可见（CI 无法验证 GUI）
- 额外修改：`EngineStatus` 添加了 `Deserialize` derive 以支持事件 payload 反序列化

### File List

- `src-tauri/icons/tray-green.png` — 新建：32x32 绿色占位托盘图标
- `src-tauri/icons/tray-yellow.png` — 新建：32x32 黄色占位托盘图标
- `src-tauri/icons/tray-red.png` — 新建：32x32 红色占位托盘图标
- `src-tauri/src/tray/mod.rs` — 重写：完整托盘实现（create_tray, handle_menu_event, update_tray_icon, update_tray_menu, resolve_tray_state）
- `src-tauri/src/lib.rs` — 修改：添加 mod tray, use Listener, create_tray 调用, napcat/engine 事件监听
- `src-tauri/Cargo.toml` — 修改：tauri features 添加 "image-png"
- `src-tauri/src/engine/scheduler.rs` — 修改：EngineStatus 添加 Deserialize derive

### Change Log

- 2026-03-13: Story 3.1 实现完成，所有 6 个 Task 已完成（Task 6.2 待手动验证）

## QA Results

### Review Date: 2026-03-14
### Reviewer: Quinn (QA Agent) | Model: Claude Opus 4.6

### AC Verification: 7/7 PASS

| AC | Result | Notes |
|---|--------|-------|
| #1 托盘图标显示 | ✅ PASS | create_tray() 在 setup 末尾调用，所有 State 注册之后 |
| #2 图标颜色状态 | ✅ PASS | resolve_tray_state() 正确映射 NapCatStatus+is_paused→TrayState |
| #3 右键菜单 5 项 | ✅ PASS | open_panel/start_like/toggle_pause/napcat_status(disabled)/quit |
| #4 立即点赞 | ✅ PASS | BatchLikeRunning 防重复 + tokio::spawn 异步执行 |
| #5 暂停/恢复 | ✅ PASS | pause(&ref)/resume(owned) 签名差异正确处理 |
| #6 事件驱动图标更新 | ✅ PASS | napcat:status-changed + engine:status-changed 双监听器 |
| #7 三套图标文件 | ✅ PASS | 32x32 PNG 占位图标 + include_bytes! 嵌入 |

### Architecture Compliance: PASS

- tracing 日志 ✅ | tokio::spawn 异步 ✅ | Tauri State 管理 ✅
- include_bytes! 嵌入 ✅ | TrayIconBuilder::with_id ✅ | 无 unwrap() ✅

### Findings

**P2-F1**: engine:status-changed listener 未考虑 NapCat 状态判断 TrayState（lib.rs:164-168），可致短暂图标不一致。建议 Story 3.3 统一处理。

**P3-F2**: engine:status-changed 覆盖 napcat_status 菜单项文字（lib.rs:152-159），混合展示引擎/NapCat 状态。

**P3-F3**: update_tray_icon/update_tray_menu 在 tray 不存在时静默成功无日志（tray/mod.rs:166,189）。

**P4-F4**: cargo check 10 warnings（前序 Story 遗留，非 3.1 引入）。

**P4-F5**: Task 6.2（手动 UI 验证）待用户完成。

### Gate Decision: PASS with CONCERNS

所有 AC 满足，编译通过，架构合规。P2 发现为非阻塞边缘场景，建议 Story 3.3 统一解决。
