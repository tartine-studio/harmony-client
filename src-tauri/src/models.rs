//! Wire types shared with the server and the frontend.
//!
//! Field names follow the protocol (camelCase). Every type here is exported to
//! `src/lib/bindings/` by ts-rs when running `cargo test`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ChannelType {
    Text,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Channel {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Message {
    pub id: String,
    /// `None` when the author's account was deleted.
    pub user_id: Option<String>,
    pub channel_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PresenceStatus {
    Online,
    Idle,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Presence {
    pub user_id: String,
    pub status: PresenceStatus,
    /// Only present on your own entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub invisible: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PresenceSnapshot {
    pub presences: Vec<Presence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MessageDeleted {
    pub message_id: String,
}

/// Server → client events the app handles. Other events (voice…) are ignored
/// for now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerEvent {
    PresenceSnapshot(PresenceSnapshot),
    PresenceBroadcast(Presence),
    MessageCreated(Message),
    MessageUpdated(Message),
    MessageDeleted(MessageDeleted),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum GatewayStatus {
    Connecting,
    Connected,
    Reconnecting,
    Disconnected,
}

/// What the frontend gets after logging in or restoring a session.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionInfo {
    pub server_url: String,
    pub me: User,
}

/// Token pair returned by login and refresh.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}
