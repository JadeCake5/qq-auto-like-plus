use thiserror::Error;

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
    NapCat(String),
    #[error("OneBot 错误: {0}")]
    OneBot(#[from] crate::onebot::OneBotError),
    #[error("名额已耗尽: {0}")]
    QuotaExhausted(String),
}
