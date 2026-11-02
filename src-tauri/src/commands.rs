//! Tauri commands: the only entry points the frontend has into the core.

use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::api::SessionHooks;
use crate::error::{AppError, AppResult};
use crate::gateway::GatewaySink;
use crate::models::{Channel, GatewayStatus, Message, ServerEvent, SessionInfo, User};
use crate::secure_store;
use crate::session::{Platform, SessionState, persist_refresh_token};

/// Same limit as the REST API; the WebSocket doesn't enforce it.
pub const MAX_MESSAGE_CHARS: usize = 2000;

pub const EVENT_GATEWAY: &str = "gateway:event";
pub const EVENT_STATUS: &str = "gateway:status";
pub const EVENT_SESSION_EXPIRED: &str = "session:expired";

struct TauriPlatform(AppHandle);

struct TauriHooks {
    app: AppHandle,
    server_url: String,
}

impl SessionHooks for TauriHooks {
    fn tokens_changed(&self, refresh_token: &str) {
        persist_refresh_token(&self.server_url, refresh_token);
    }

    fn session_expired(&self) {
        let _ = secure_store::clear();
        let _ = self.app.emit(EVENT_SESSION_EXPIRED, ());
    }
}

struct TauriSink(AppHandle);

impl GatewaySink for TauriSink {
    fn status(&self, status: GatewayStatus) {
        let _ = self.0.emit(EVENT_STATUS, status);
    }

    fn event(&self, event: &ServerEvent) {
        let _ = self.0.emit(EVENT_GATEWAY, event);
    }
}

impl Platform for TauriPlatform {
    fn hooks(&self, server_url: &str) -> Arc<dyn SessionHooks> {
        Arc::new(TauriHooks {
            app: self.0.clone(),
            server_url: server_url.to_owned(),
        })
    }

    fn sink(&self) -> Arc<dyn GatewaySink> {
        Arc::new(TauriSink(self.0.clone()))
    }
}

/// Trims and checks a message body.
pub fn validate_content(content: &str) -> AppResult<String> {
    let content = content.trim();
    if content.is_empty() {
        return Err(AppError::new("MESSAGE_EMPTY", "the message is empty"));
    }
    if content.chars().count() > MAX_MESSAGE_CHARS {
        return Err(AppError::new(
            "MESSAGE_TOO_LONG",
            "the message is longer than 2000 characters",
        ));
    }
    Ok(content.to_owned())
}

// ----- Session ----------------------------------------------------------------

#[tauri::command]
pub async fn session_restore(
    app: AppHandle,
    state: State<'_, SessionState>,
) -> AppResult<Option<SessionInfo>> {
    state.restore(&TauriPlatform(app)).await
}

#[tauri::command]
pub async fn session_login(
    app: AppHandle,
    state: State<'_, SessionState>,
    server_url: String,
    email: String,
    password: String,
) -> AppResult<SessionInfo> {
    state
        .login(&TauriPlatform(app), &server_url, &email, &password)
        .await
}

#[tauri::command]
pub async fn session_register(
    app: AppHandle,
    state: State<'_, SessionState>,
    server_url: String,
    username: String,
    email: String,
    password: String,
) -> AppResult<SessionInfo> {
    state
        .register(
            &TauriPlatform(app),
            &server_url,
            &username,
            &email,
            &password,
        )
        .await
}

#[tauri::command]
pub async fn session_logout(state: State<'_, SessionState>) -> AppResult<()> {
    state.logout().await
}

// ----- Data ---------------------------------------------------------------------

#[tauri::command]
pub async fn list_users(state: State<'_, SessionState>) -> AppResult<Vec<User>> {
    state.api().await?.users().await
}

#[tauri::command]
pub async fn list_channels(state: State<'_, SessionState>) -> AppResult<Vec<Channel>> {
    state.api().await?.channels().await
}

#[tauri::command]
pub async fn list_messages(
    state: State<'_, SessionState>,
    channel_id: String,
) -> AppResult<Vec<Message>> {
    state.api().await?.messages(&channel_id).await
}

// ----- Gateway ------------------------------------------------------------------

#[tauri::command]
pub async fn send_message(
    state: State<'_, SessionState>,
    channel_id: String,
    content: String,
) -> AppResult<Message> {
    let content = validate_content(&content)?;
    state
        .gateway()
        .await?
        .send_message(&channel_id, &content)
        .await
}

#[tauri::command]
pub async fn edit_message(
    state: State<'_, SessionState>,
    message_id: String,
    content: String,
) -> AppResult<Message> {
    let content = validate_content(&content)?;
    state
        .gateway()
        .await?
        .edit_message(&message_id, &content)
        .await
}

#[tauri::command]
pub async fn delete_message(state: State<'_, SessionState>, message_id: String) -> AppResult<()> {
    state.gateway().await?.delete_message(&message_id).await
}

#[tauri::command]
pub async fn set_invisible(state: State<'_, SessionState>, invisible: bool) -> AppResult<()> {
    state.gateway().await?.set_invisible(invisible)
}

#[tauri::command]
pub async fn report_activity(state: State<'_, SessionState>) -> AppResult<()> {
    if let Ok(gateway) = state.gateway().await {
        gateway.report_activity();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_trimmed_and_bounded() {
        assert_eq!(validate_content("  hi \n").unwrap(), "hi");
        assert_eq!(validate_content(" \n ").unwrap_err().code, "MESSAGE_EMPTY");
        assert!(validate_content(&"é".repeat(2000)).is_ok());
        assert_eq!(
            validate_content(&"é".repeat(2001)).unwrap_err().code,
            "MESSAGE_TOO_LONG"
        );
    }
}
