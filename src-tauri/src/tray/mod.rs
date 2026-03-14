use anyhow::Result;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayState {
    Running,
    Pending,
    Error,
}

/// 在 setup 中调用，创建系统托盘
pub fn create_tray(app: &tauri::App) -> Result<()> {
    let menu = build_tray_menu(app)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(green_icon()?)
        .menu(&menu)
        .tooltip("QQ Auto Like Plus - 运行中")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    let _ = window.center();
                }
            }
        })
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
}

fn green_icon() -> Result<Image<'static>> {
    let img = Image::from_bytes(include_bytes!("../../icons/tray-green.png"))
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(img.to_owned())
}

fn yellow_icon() -> Result<Image<'static>> {
    let img = Image::from_bytes(include_bytes!("../../icons/tray-yellow.png"))
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(img.to_owned())
}

fn red_icon() -> Result<Image<'static>> {
    let img = Image::from_bytes(include_bytes!("../../icons/tray-red.png"))
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(img.to_owned())
}

fn build_tray_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let open_item = MenuItem::with_id(app, "open_panel", "打开面板", true, None::<&str>)?;
    let like_item = MenuItem::with_id(app, "start_like", "立即点赞", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "toggle_pause", "暂停", true, None::<&str>)?;
    let status_item =
        MenuItem::with_id(app, "napcat_status", "运行环境: 未知", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    Menu::with_items(app, &[&open_item, &like_item, &pause_item, &status_item, &quit_item])
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "open_panel" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                let _ = window.center();
            }
        }
        "start_like" => {
            let db = app
                .state::<crate::db::DbState>()
                .inner()
                .clone();
            let onebot = app
                .state::<crate::onebot::OneBotClientState>()
                .inner()
                .clone();
            let running = app
                .state::<crate::commands::like::BatchLikeRunning>()
                .inner()
                .clone();
            let app_handle = app.clone();

            tokio::spawn(async move {
                if running.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    tracing::warn!("批量点赞正在执行中，忽略托盘触发");
                    return;
                }
                let result = crate::engine::like_executor::run_batch_like(
                    &db, &onebot, &app_handle, "manual",
                )
                .await;
                running.store(false, std::sync::atomic::Ordering::SeqCst);
                match result {
                    Ok(r) => tracing::info!("手动批量点赞完成: {:?}", r),
                    Err(e) => tracing::error!("手动批量点赞异常: {}", e),
                }
            });
        }
        "toggle_pause" => {
            let scheduler = app
                .state::<crate::engine::scheduler::LikeSchedulerState>()
                .inner()
                .clone();
            let db = app
                .state::<crate::db::DbState>()
                .inner()
                .clone();
            let onebot = app
                .state::<crate::onebot::OneBotClientState>()
                .inner()
                .clone();
            let running = app
                .state::<crate::commands::like::BatchLikeRunning>()
                .inner()
                .clone();
            let app_handle = app.clone();

            tokio::spawn(async move {
                let status = scheduler.get_status().await;
                let result = if status.is_paused {
                    scheduler
                        .resume(db, onebot, app_handle.clone(), running)
                        .await
                } else {
                    scheduler.pause(&db, &app_handle).await
                };
                if let Err(e) = result {
                    tracing::error!("切换引擎状态失败: {}", e);
                }
            });
        }
        "quit" => {
            let app_handle = app.clone();
            app.dialog()
                .message("确定退出 QQ Auto Like Plus？退出后将停止自动点赞。")
                .title("确认退出")
                .kind(MessageDialogKind::Warning)
                .buttons(MessageDialogButtons::OkCancel)
                .show(move |confirmed| {
                    if confirmed {
                        let napcat_state = app_handle
                            .state::<crate::napcat::process::NapCatProcessState>()
                            .inner()
                            .clone();
                        let mut guard = match napcat_state.lock() {
                            Ok(g) => g,
                            Err(e) => e.into_inner(),
                        };
                        if let Some(ref mut process) = *guard {
                            if let Err(e) = process.stop() {
                                tracing::error!("退出时停止 NapCat 失败: {}", e);
                            }
                        }
                        app_handle.exit(0);
                    }
                });
        }
        _ => {}
    }
}

/// 更新托盘图标和 tooltip
pub fn update_tray_icon(app: &AppHandle, state: TrayState) -> Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let icon = match state {
            TrayState::Running => green_icon()?,
            TrayState::Pending => yellow_icon()?,
            TrayState::Error => red_icon()?,
        };
        let tooltip = match state {
            TrayState::Running => "QQ Auto Like Plus - 运行中",
            TrayState::Pending => "QQ Auto Like Plus - 等待中",
            TrayState::Error => "QQ Auto Like Plus - 异常",
        };
        tray.set_icon(Some(icon))?;
        tray.set_tooltip(Some(tooltip))?;
    }
    Ok(())
}

/// 重建托盘菜单以更新暂停/恢复文字和 NapCat 状态
pub fn update_tray_menu(
    app: &AppHandle,
    is_paused: bool,
    napcat_status_text: &str,
) -> Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let open_item = MenuItem::with_id(app, "open_panel", "打开面板", true, None::<&str>)?;
        let like_item = MenuItem::with_id(app, "start_like", "立即点赞", true, None::<&str>)?;
        let pause_text = if is_paused { "恢复" } else { "暂停" };
        let pause_item =
            MenuItem::with_id(app, "toggle_pause", pause_text, true, None::<&str>)?;
        let status_item =
            MenuItem::with_id(app, "napcat_status", napcat_status_text, false, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
        let menu = Menu::with_items(
            app,
            &[&open_item, &like_item, &pause_item, &status_item, &quit_item],
        )?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

/// 根据 NapCatStatus 判断 TrayState
pub fn resolve_tray_state(napcat_status: &crate::napcat::NapCatStatus, is_paused: bool) -> TrayState {
    use crate::napcat::NapCatStatus;
    match napcat_status {
        NapCatStatus::Error(_) => TrayState::Error,
        NapCatStatus::NotInstalled
        | NapCatStatus::Downloading
        | NapCatStatus::Extracting
        | NapCatStatus::Starting
        | NapCatStatus::WaitingForLogin => TrayState::Pending,
        NapCatStatus::Ready | NapCatStatus::Running => {
            if is_paused {
                TrayState::Pending
            } else {
                TrayState::Running
            }
        }
    }
}
