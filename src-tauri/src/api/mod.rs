//! Harmony REST endpoints.

mod client;

pub use client::{ApiClient, SessionHooks};

#[cfg(test)]
pub(crate) use client::tests::RecordingHooks;

use crate::error::AppResult;
use crate::models::{Channel, Message, User};

impl ApiClient {
    pub async fn me(&self) -> AppResult<User> {
        self.get("users/me").await
    }

    pub async fn users(&self) -> AppResult<Vec<User>> {
        self.get("users").await
    }

    /// Channels sorted by name (the server doesn't guarantee an order).
    pub async fn channels(&self) -> AppResult<Vec<Channel>> {
        let mut channels: Vec<Channel> = self.get("channels").await?;
        channels.sort_by_cached_key(|c| (c.name.to_lowercase(), c.id.clone()));
        Ok(channels)
    }

    /// Full history of a channel, oldest first.
    pub async fn messages(&self, channel_id: &str) -> AppResult<Vec<Message>> {
        let id = url::form_urlencoded::byte_serialize(channel_id.as_bytes()).collect::<String>();
        let mut messages: Vec<Message> = self.get(&format!("channels/{id}/messages")).await?;
        // Same-second messages come back in any order: keep a stable one.
        messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(messages)
    }
}
