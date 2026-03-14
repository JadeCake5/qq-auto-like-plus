pub mod client;
pub mod types;

pub use client::OneBotClient;
pub use types::*;

pub type OneBotClientState = std::sync::Arc<OneBotClient>;
