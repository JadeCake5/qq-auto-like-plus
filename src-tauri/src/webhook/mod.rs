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
}

impl WebhookServerHandle {
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    pub fn is_running(&self) -> bool {
        !self.join_handle.is_finished()
    }
}

pub async fn start(app_handle: AppHandle, port: u16) -> Result<WebhookServerHandle, String> {
    let state = WebhookState {
        app_handle: app_handle.clone(),
    };

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| {
            let msg = format!("Webhook 端口 {} 绑定失败: {}", port, e);
            tracing::error!("{}", msg);
            let _ = app_handle.emit(
                "webhook:error",
                serde_json::json!({ "message": msg }),
            );
            msg
        })?;

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);

    let shutdown_signal = async move {
        shutdown_rx.changed().await.ok();
    };

    tracing::info!("Webhook 服务器已启动: 127.0.0.1:{}", port);

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
    })
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
