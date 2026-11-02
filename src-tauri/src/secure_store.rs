//! Persists the session in the OS credential store (Keychain, Windows
//! Credential Manager, Secret Service).

use crate::error::AppResult;
use serde::{Deserialize, Serialize};

const SERVICE: &str = "studio.tartine.harmony";
const ACCOUNT: &str = "session";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSession {
    pub server_url: String,
    pub refresh_token: String,
}

fn entry() -> AppResult<keyring::Entry> {
    Ok(keyring::Entry::new(SERVICE, ACCOUNT)?)
}

pub fn load() -> AppResult<Option<StoredSession>> {
    match entry()?.get_password() {
        Ok(json) => Ok(serde_json::from_str(&json).ok()),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn save(session: &StoredSession) -> AppResult<()> {
    let json = serde_json::to_string(session).expect("StoredSession always serializes");
    entry()?.set_password(&json)?;
    Ok(())
}

pub fn clear() -> AppResult<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
