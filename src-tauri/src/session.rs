//! The logged-in session: API client, gateway and current user.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::api::{ApiClient, SessionHooks};
use crate::error::{AppError, AppResult};
use crate::gateway::{Gateway, GatewaySink};
use crate::models::{SessionInfo, User};
use crate::secure_store::{self, StoredSession};
use crate::url::{display, normalize_server_url};

pub struct Active {
    pub api: Arc<ApiClient>,
    pub gateway: Gateway,
    pub me: User,
    pub server_url: String,
}

impl Active {
    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            server_url: self.server_url.clone(),
            me: self.me.clone(),
        }
    }
}

/// Builds the hooks and sink for a given server. Lets the Tauri layer plug in
/// the keyring and event emission.
pub trait Platform: Send + Sync + 'static {
    fn hooks(&self, server_url: &str) -> Arc<dyn SessionHooks>;
    fn sink(&self) -> Arc<dyn GatewaySink>;
}

#[derive(Default)]
pub struct SessionState {
    active: Mutex<Option<Active>>,
}

impl SessionState {
    pub async fn api(&self) -> AppResult<Arc<ApiClient>> {
        self.active
            .lock()
            .await
            .as_ref()
            .map(|a| a.api.clone())
            .ok_or_else(AppError::not_authenticated)
    }

    pub async fn gateway(&self) -> AppResult<Gateway> {
        self.active
            .lock()
            .await
            .as_ref()
            .map(|a| a.gateway.clone())
            .ok_or_else(AppError::not_authenticated)
    }

    /// Resumes the session stored in the keyring, if any and still valid.
    /// Returns the current session as is when one is already open (e.g. after
    /// a frontend reload).
    pub async fn restore(&self, platform: &dyn Platform) -> AppResult<Option<SessionInfo>> {
        let mut active = self.active.lock().await;
        if let Some(current) = active.as_ref() {
            return Ok(Some(current.info()));
        }
        let Some(stored) = secure_store::load()? else {
            return Ok(None);
        };
        let Ok(base) = normalize_server_url(&stored.server_url) else {
            secure_store::clear()?;
            return Ok(None);
        };
        let server_url = display(&base);
        let api = Arc::new(ApiClient::new(base, platform.hooks(&server_url)));
        api.set_refresh_token(stored.refresh_token);
        match api.refresh(None).await {
            Ok(()) => {}
            Err(e) if e.is_auth_failure() => {
                secure_store::clear()?;
                return Ok(None);
            }
            Err(e) => return Err(e),
        }
        let session = open(api, server_url, platform).await?;
        let info = session.info();
        *active = Some(session);
        Ok(Some(info))
    }

    pub async fn login(
        &self,
        platform: &dyn Platform,
        server_url: &str,
        email: &str,
        password: &str,
    ) -> AppResult<SessionInfo> {
        let base = normalize_server_url(server_url)?;
        let server_url = display(&base);
        let api = Arc::new(ApiClient::new(base, platform.hooks(&server_url)));
        api.login(email.trim(), password).await?;
        self.install(open(api, server_url, platform).await?).await
    }

    pub async fn register(
        &self,
        platform: &dyn Platform,
        server_url: &str,
        username: &str,
        email: &str,
        password: &str,
    ) -> AppResult<SessionInfo> {
        let base = normalize_server_url(server_url)?;
        let server_url = display(&base);
        let api = Arc::new(ApiClient::new(base, platform.hooks(&server_url)));
        api.register(username.trim(), email.trim(), password)
            .await?;
        api.login(email.trim(), password).await?;
        self.install(open(api, server_url, platform).await?).await
    }

    pub async fn logout(&self) -> AppResult<()> {
        if let Some(session) = self.active.lock().await.take() {
            session.gateway.stop();
        }
        secure_store::clear()
    }

    async fn install(&self, session: Active) -> AppResult<SessionInfo> {
        let info = session.info();
        if let Some(previous) = self.active.lock().await.replace(session) {
            previous.gateway.stop();
        }
        Ok(info)
    }
}

async fn open(
    api: Arc<ApiClient>,
    server_url: String,
    platform: &dyn Platform,
) -> AppResult<Active> {
    let me = api.me().await?;
    let gateway = Gateway::start(api.clone(), me.id.clone(), platform.sink());
    Ok(Active {
        api,
        gateway,
        me,
        server_url,
    })
}

/// Keyring + event glue used by [`Platform`] implementations.
pub fn persist_refresh_token(server_url: &str, refresh_token: &str) {
    let stored = StoredSession {
        server_url: server_url.to_owned(),
        refresh_token: refresh_token.to_owned(),
    };
    if let Err(e) = secure_store::save(&stored) {
        tracing::warn!("could not save the session in the keyring: {e}");
    }
}
