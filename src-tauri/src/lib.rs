use tauri::{Listener, Manager};

mod commands;
mod config;
mod db;
mod engine;
mod errors;
mod friends;
mod napcat;
mod onebot;
mod stats;
mod tray;
mod webhook;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostarted"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tracing::info!("检测到重复实例，激活已有窗口");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .max_file_size(10_000_000) // 10 MB
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            let db_state =
                db::init_db(&app_data_dir).expect("failed to initialize database");
            app.manage(db_state.clone());

            // OneBotClient 初始化
            let api_port: u16 = {
                let conn = db_state.lock().expect("lock db for onebot init");
                db::models::get_config_by_key(&conn, "napcat_api_port")
                    .ok()
                    .and_then(|c| c.value.parse().ok())
                    .unwrap_or(3000)
            };
            let onebot_client: onebot::OneBotClientState =
                std::sync::Arc::new(onebot::OneBotClient::new(api_port));
            app.manage(onebot_client.clone());

            // BatchLikeRunning 状态
            let batch_running: commands::like::BatchLikeRunning =
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            app.manage(batch_running.clone());

            // LikeScheduler 初始化
            let like_scheduler = tauri::async_runtime::block_on(async {
                engine::scheduler::LikeScheduler::new()
                    .await
                    .expect("failed to create scheduler")
            });
            let scheduler_state: engine::scheduler::LikeSchedulerState =
                std::sync::Arc::new(like_scheduler);
            app.manage(scheduler_state.clone());

            // 后台启动调度器
            let db_for_sched = db_state.clone();
            let onebot_for_sched = onebot_client.clone();
            let running_for_sched = batch_running.clone();
            let app_handle = app.handle().clone();
            let sched = scheduler_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = sched.start(
                    db_for_sched,
                    onebot_for_sched,
                    app_handle,
                    running_for_sched,
                ).await {
                    tracing::error!("调度器启动失败: {}", e);
                }
            });

            // 启动时清理过期点赞历史（90 天前）
            let db_for_cleanup = db_state.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                if let Ok(conn) = db_for_cleanup.lock() {
                    match stats::queries::cleanup_old_history(&conn, 90) {
                        Ok(deleted) => {
                            if deleted > 0 {
                                tracing::info!("启动清理完成: 删除 {} 条过期记录", deleted);
                            }
                        }
                        Err(e) => tracing::error!("启动清理失败: {}", e),
                    }
                }
            });

            let napcat_state: napcat::process::NapCatProcessState =
                std::sync::Arc::new(std::sync::Mutex::new(None));
            app.manage(napcat_state);

            // Webhook 服务器启动
            let webhook_port: u16 = {
                let conn = db_state.lock().expect("lock db for webhook init");
                db::models::get_config_by_key(&conn, "webhook_port")
                    .ok()
                    .and_then(|c| c.value.parse().ok())
                    .unwrap_or(8080)
            };
            let webhook_app_handle = app.handle().clone();
            let webhook_handle: Option<webhook::WebhookServerHandle> = {
                match tauri::async_runtime::block_on(webhook::start(webhook_app_handle, webhook_port)) {
                    Ok(handle) => {
                        // 如果实际端口与配置不同（因为端口冲突回退），更新 DB
                        let actual_port = handle.port();
                        if actual_port != webhook_port {
                            let conn = db_state.lock().expect("lock db for webhook port update");
                            let _ = db::models::upsert_config(&conn, "webhook_port", &actual_port.to_string());
                            tracing::info!("Webhook 实际端口 {} 已保存到配置", actual_port);
                        }
                        Some(handle)
                    }
                    Err(e) => {
                        tracing::error!("Webhook 服务器启动失败: {}", e);
                        None
                    }
                }
            };
            app.manage::<commands::webhook::WebhookState>(std::sync::Mutex::new(webhook_handle));

            // 监听 webhook:profile-like 事件触发回赞
            let db_for_reply = db_state.clone();
            let onebot_for_reply = onebot_client.clone();
            let app_handle_for_reply = app.handle().clone();
            app.listen("webhook:profile-like", move |event| {
                let payload = event.payload();
                if let Ok(like_payload) =
                    serde_json::from_str::<onebot::types::ProfileLikePayload>(payload)
                {
                    let db = db_for_reply.clone();
                    let onebot = onebot_for_reply.clone();
                    let handle = app_handle_for_reply.clone();
                    tokio::spawn(async move {
                        if let Err(e) = engine::reply_handler::handle_reply_like(
                            like_payload.operator_id,
                            &db,
                            &onebot,
                            &handle,
                        )
                        .await
                        {
                            tracing::error!("回赞处理失败: {}", e);
                        }
                    });
                }
            });

            // 创建系统托盘（必须在所有 State 注册之后）
            tray::create_tray(app)?;

            // 监听 napcat:status-changed 事件更新托盘图标
            let app_handle_for_napcat = app.handle().clone();
            app.listen("napcat:status-changed", move |event| {
                let payload = event.payload();
                if let Ok(status) =
                    serde_json::from_str::<napcat::NapCatStatus>(payload)
                {
                    let scheduler = app_handle_for_napcat
                        .state::<engine::scheduler::LikeSchedulerState>()
                        .inner()
                        .clone();
                    let app_clone = app_handle_for_napcat.clone();
                    tokio::spawn(async move {
                        let engine_status = scheduler.get_status().await;
                        let tray_state =
                            tray::resolve_tray_state(&status, engine_status.is_paused);
                        if let Err(e) = tray::update_tray_icon(&app_clone, tray_state) {
                            tracing::error!("更新托盘图标失败: {}", e);
                        }
                        let napcat_text = match &status {
                            napcat::NapCatStatus::Running => "运行环境: 已连接".to_string(),
                            napcat::NapCatStatus::NotInstalled => {
                                "运行环境: 未安装".to_string()
                            }
                            napcat::NapCatStatus::Downloading => {
                                "运行环境: 下载中".to_string()
                            }
                            napcat::NapCatStatus::Extracting => {
                                "运行环境: 解压中".to_string()
                            }
                            napcat::NapCatStatus::Starting => {
                                "运行环境: 启动中".to_string()
                            }
                            napcat::NapCatStatus::WaitingForLogin => {
                                "运行环境: 等待登录".to_string()
                            }
                            napcat::NapCatStatus::Ready => "运行环境: 就绪".to_string(),
                            napcat::NapCatStatus::Error(msg) => {
                                format!("运行环境: 异常 - {}", msg)
                            }
                        };
                        if let Err(e) = tray::update_tray_menu(
                            &app_clone,
                            engine_status.is_paused,
                            &napcat_text,
                        ) {
                            tracing::error!("更新托盘菜单失败: {}", e);
                        }
                    });
                }
            });

            // 监听 engine:status-changed 事件更新暂停/恢复菜单文字
            let app_handle_for_engine = app.handle().clone();
            app.listen("engine:status-changed", move |event| {
                let payload = event.payload();
                if let Ok(engine_status) =
                    serde_json::from_str::<engine::scheduler::EngineStatus>(payload)
                {
                    // 更新菜单文字
                    if let Err(e) = tray::update_tray_menu(
                        &app_handle_for_engine,
                        engine_status.is_paused,
                        &format!(
                            "引擎: {}",
                            if engine_status.is_paused {
                                "已暂停"
                            } else {
                                "运行中"
                            }
                        ),
                    ) {
                        tracing::error!("更新托盘菜单失败: {}", e);
                    }
                    // 更新图标：暂停时黄色
                    let tray_state = if engine_status.is_paused {
                        tray::TrayState::Pending
                    } else {
                        tray::TrayState::Running
                    };
                    if let Err(e) =
                        tray::update_tray_icon(&app_handle_for_engine, tray_state)
                    {
                        tracing::error!("更新托盘图标失败: {}", e);
                    }
                }
            });

            // 检测开机自启模式，隐藏主窗口到托盘
            let is_autostarted = std::env::args().any(|arg| arg == "--autostarted");
            if is_autostarted {
                tracing::info!("检测到开机自启模式，隐藏主窗口");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                tracing::info!("面板窗口已隐藏到托盘");
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_config,
            commands::settings::update_config,
            commands::napcat::download_napcat,
            commands::napcat::import_napcat,
            commands::napcat::get_napcat_status,
            commands::napcat::start_napcat,
            commands::napcat::stop_napcat,
            commands::napcat::get_login_info_cmd,
            commands::napcat::restart_napcat,
            commands::napcat::open_napcat_dir,
            commands::napcat::clear_napcat_cache,
            commands::napcat::update_napcat,
            commands::like::get_daily_stats,
            commands::like::start_batch_like,
            commands::engine::pause_engine,
            commands::engine::resume_engine,
            commands::engine::get_next_run_time,
            commands::engine::get_engine_status,
            commands::settings::enable_autostart,
            commands::settings::disable_autostart,
            commands::settings::is_autostart_enabled,
            commands::webhook::get_webhook_status,
            commands::friends::get_friends,
            commands::friends::sync_friends,
            commands::friends::get_tags,
            commands::friends::create_tag,
            commands::friends::update_tag,
            commands::friends::delete_tag,
            commands::friends::set_friend_tags,
            commands::friends::update_tag_strategy,
            commands::stats::get_stats_daily,
            commands::stats::get_stats_weekly,
            commands::stats::get_stats_monthly,
            commands::stats::get_like_type_ratio,
            commands::stats::get_friend_ranking,
            commands::logs::get_startup_logs,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let state = app_handle
                .state::<commands::webhook::WebhookState>();
            if let Ok(guard) = state.lock() {
                if let Some(handle) = guard.as_ref() {
                    handle.shutdown();
                    tracing::info!("Webhook 服务器已触发停止");
                }
            };
        }
    });
}
