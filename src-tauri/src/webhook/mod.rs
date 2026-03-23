use axum::{extract::State, routing::post, Json, Router};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;

use crate::onebot::types::{OneBotEvent, ProfileLikePayload};

#[derive(Clone)]
struct WebhookState {
    app_handle: AppHandle,
}

pub struct WebhookServerHandle {
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    join_handle: tokio::task::JoinHandle<()>,
    port: u16,
}

impl WebhookServerHandle {
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    pub fn is_running(&self) -> bool {
        !self.join_handle.is_finished()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub async fn start(app_handle: AppHandle, port: u16) -> Result<WebhookServerHandle, String> {
    let state = WebhookState {
        app_handle: app_handle.clone(),
    };

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    // 尝试绑定端口，如果失败则重试几个备选端口
    let (listener, actual_port) = try_bind_port(port).await.map_err(|e| {
        let msg = format!("Webhook 端口绑定失败（已尝试多个端口）: {}", e);
        tracing::error!("{}", msg);
        let _ = app_handle.emit(
            "webhook:error",
            serde_json::json!({ "message": msg }),
        );
        msg
    })?;

    if actual_port != port {
        tracing::warn!(
            "Webhook 端口 {} 被占用，已使用备选端口 {}",
            port, actual_port
        );
    }

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

    let shutdown_signal = async move {
        shutdown_rx.changed().await.ok();
    };

    tracing::info!("Webhook 服务器已启动: 127.0.0.1:{}", actual_port);

    let join_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
        {
            tracing::error!("Webhook 服务器异常退出: {}", e);
        }
        tracing::info!("Webhook 服务器已停止");
    });

    Ok(WebhookServerHandle {
        shutdown_tx,
        join_handle,
        port: actual_port,
    })
}

/// 尝试绑定端口，失败时依次尝试备选端口
async fn try_bind_port(preferred: u16) -> Result<(TcpListener, u16), std::io::Error> {
    // 首先尝试首选端口
    match TcpListener::bind(("127.0.0.1", preferred)).await {
        Ok(listener) => return Ok((listener, preferred)),
        Err(e) => tracing::warn!("端口 {} 绑定失败: {}，尝试备选端口", preferred, e),
    }

    // 尝试几个备选端口
    let alternatives = [preferred + 1, preferred + 2, preferred + 10, 0];
    for port in alternatives {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => {
                let actual = listener.local_addr()?.port();
                return Ok((listener, actual));
            }
            Err(_) => continue,
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "所有备选端口均不可用",
    ))
}

async fn handle_webhook(
    State(state): State<WebhookState>,
    body: axum::body::Bytes,
) -> Json<Value> {
    let body_str = String::from_utf8_lossy(&body);

    let event: OneBotEvent = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(
                "Webhook 事件解析失败: {}, body: {}",
                e,
                &body_str[..body_str.len().min(200)]
            );
            return Json(serde_json::json!({ "status": "ok" }));
        }
    };

    if event.post_type == "notice"
        && event.notice_type == "notify"
        && event.sub_type == "profile_like"
    {
        tracing::info!(
            "收到 profile_like 事件: operator_id={}",
            event.operator_id
        );

        let payload = ProfileLikePayload {
            operator_id: event.operator_id,
            timestamp: event.time,
        };

        if let Err(e) = state.app_handle.emit("webhook:profile-like", &payload) {
            tracing::error!("emit webhook:profile-like 失败: {}", e);
        }
    } else {
        tracing::debug!(
            "忽略非 profile_like 事件: post_type={}, notice_type={}, sub_type={}",
            event.post_type,
            event.notice_type,
            event.sub_type
        );
    }

    Json(serde_json::json!({ "status": "ok" }))
}
