use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── OneBot 错误类型 ──

#[derive(Error, Debug)]
pub enum OneBotError {
    #[error("连接被拒绝: {0}")]
    ConnectionRefused(String),
    #[error("请求超时: {0}")]
    Timeout(String),
    #[error("API 返回错误 (retcode={retcode}): {message}")]
    ApiError { retcode: i32, message: String },
    #[error("网络错误: {0}")]
    Network(String),
    #[error("反序列化错误: {0}")]
    Deserialize(String),
}

// ── OneBot 通用响应 ──

#[derive(Debug, Deserialize)]
pub struct OneBotResponse<T> {
    pub status: String,
    pub retcode: i32,
    pub data: Option<T>,
}

// ── 请求类型（发送给 OneBot，保持 snake_case）──

#[derive(Debug, Serialize)]
pub struct SendLikeRequest {
    pub user_id: i64,
    pub times: i32,
}

// ── 响应类型（暴露给前端 camelCase + alias 兼容 OneBot snake_case）──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendInfo {
    #[serde(alias = "user_id")]
    pub user_id: i64,
    pub nickname: String,
    #[serde(default, alias = "remark")]
    pub remark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneBotLoginInfo {
    #[serde(alias = "user_id")]
    pub user_id: i64,
    pub nickname: String,
}

// ── Webhook 事件类型（OneBot 11 推送事件）──

#[derive(Debug, Clone, Deserialize)]
pub struct OneBotEvent {
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub self_id: i64,
    #[serde(default)]
    pub post_type: String,
    #[serde(default)]
    pub notice_type: String,
    #[serde(default)]
    pub sub_type: String,
    #[serde(default)]
    pub user_id: i64,
    #[serde(default, alias = "sender_id")]
    pub operator_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileLikePayload {
    pub operator_id: i64,
    pub timestamp: i64,
}
