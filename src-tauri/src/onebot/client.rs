use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::types::*;

const MAX_RETRIES: u32 = 2;
const RETRY_INTERVAL: Duration = Duration::from_secs(1);

pub struct OneBotClient {
    client: reqwest::Client,
    base_url: String,
}

impl OneBotClient {
    pub fn new(api_port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("创建 HTTP 客户端失败");
        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", api_port),
        }
    }

    /// 通用 API 调用，含重试逻辑
    async fn call_api<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &impl Serialize,
    ) -> Result<Option<T>, OneBotError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut last_error: Option<OneBotError> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                tracing::info!("重试 OneBot API {}: 第 {} 次", endpoint, attempt);
                tokio::time::sleep(RETRY_INTERVAL).await;
            }

            tracing::info!("调用 OneBot API: {}", endpoint);

            match self.client.post(&url).json(body).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let err = OneBotError::ApiError {
                            retcode: status.as_u16() as i32,
                            message: format!("HTTP {}", status),
                        };
                        // HTTP 5xx 可重试
                        if status.is_server_error() {
                            tracing::error!("OneBot API {} HTTP 错误: {}", endpoint, err);
                            last_error = Some(err);
                            continue;
                        }
                        return Err(err);
                    }

                    let onebot_resp: OneBotResponse<T> =
                        resp.json().await.map_err(|e| {
                            OneBotError::Deserialize(e.to_string())
                        })?;

                    if onebot_resp.status != "ok" {
                        // OneBot 业务错误不重试
                        return Err(OneBotError::ApiError {
                            retcode: onebot_resp.retcode,
                            message: format!("status={}", onebot_resp.status),
                        });
                    }

                    return Ok(onebot_resp.data);
                }
                Err(e) => {
                    let classified = classify_reqwest_error(e);
                    tracing::error!("OneBot API {} 失败: {}", endpoint, classified);
                    last_error = Some(classified);
                }
            }
        }

        Err(last_error.unwrap_or(OneBotError::Network("未知错误".to_string())))
    }

    pub async fn send_like(&self, user_id: i64, times: i32) -> Result<(), OneBotError> {
        tracing::info!("点赞: user_id={}, times={}", user_id, times);
        let req = SendLikeRequest { user_id, times };
        let _: Option<serde_json::Value> = self.call_api("/send_like", &req).await?;
        Ok(())
    }

    pub async fn get_friend_list(&self) -> Result<Vec<FriendInfo>, OneBotError> {
        self.call_api("/get_friend_list", &serde_json::json!({}))
            .await?
            .ok_or_else(|| OneBotError::Deserialize("get_friend_list 返回 null data".to_string()))
    }

    pub async fn get_login_info(&self) -> Result<OneBotLoginInfo, OneBotError> {
        self.call_api("/get_login_info", &serde_json::json!({}))
            .await?
            .ok_or_else(|| OneBotError::Deserialize("get_login_info 返回 null data".to_string()))
    }
}

fn classify_reqwest_error(e: reqwest::Error) -> OneBotError {
    if e.is_connect() {
        OneBotError::ConnectionRefused(e.to_string())
    } else if e.is_timeout() {
        OneBotError::Timeout(e.to_string())
    } else {
        OneBotError::Network(e.to_string())
    }
}
