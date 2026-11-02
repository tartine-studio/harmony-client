//! Errors returned to the frontend.
//!
//! The frontend never shows `message` directly: it translates `code` into the
//! user's language. `message` is kept for logs and debugging.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[error("{code}: {message}")]
pub struct AppError {
    /// Server error code (`INVALID_CREDENTIALS`, `REGISTRATION_FAILED`…) or one
    /// of the client codes defined below.
    pub code: String,
    /// HTTP status when the error came from a REST call.
    pub status: Option<u16>,
    pub message: String,
}

/// Body of every REST error, and `data` of WebSocket `error` frames.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerErrorBody {
    pub error: String,
    pub code: String,
}

impl AppError {
    pub const NETWORK: &'static str = "NETWORK";
    pub const TIMEOUT: &'static str = "TIMEOUT";
    pub const NOT_AUTHENTICATED: &'static str = "NOT_AUTHENTICATED";
    pub const KEYRING: &'static str = "KEYRING";
    pub const INVALID_URL: &'static str = "INVALID_URL";
    pub const INVALID_RESPONSE: &'static str = "INVALID_RESPONSE";
    pub const DISCONNECTED: &'static str = "DISCONNECTED";

    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            status: None,
            message: message.into(),
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    pub fn from_server(status: Option<u16>, body: ServerErrorBody) -> Self {
        Self {
            code: body.code,
            status,
            message: body.error,
        }
    }

    pub fn not_authenticated() -> Self {
        Self::new(Self::NOT_AUTHENTICATED, "no active session")
    }

    pub fn disconnected() -> Self {
        Self::new(Self::DISCONNECTED, "not connected to the server")
    }

    pub fn timeout() -> Self {
        Self::new(Self::TIMEOUT, "the server did not answer in time")
    }

    /// True when the session can't be recovered and the user must log in again.
    pub fn is_auth_failure(&self) -> bool {
        matches!(self.code.as_str(), "INVALID_TOKEN" | "UNAUTHORIZED")
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::timeout()
        } else if err.is_decode() {
            Self::new(Self::INVALID_RESPONSE, err.to_string())
        } else {
            Self::new(Self::NETWORK, err.to_string())
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(err: keyring::Error) -> Self {
        Self::new(Self::KEYRING, err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
