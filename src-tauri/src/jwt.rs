//! Minimal JWT reading. The signature is not checked: the client only needs
//! the expiry to refresh tokens ahead of time, the server does the real check.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

#[derive(Deserialize)]
struct Claims {
    exp: Option<u64>,
}

/// Expiry of a JWT, in seconds since the Unix epoch.
pub fn expiry(token: &str) -> Option<u64> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    serde_json::from_slice::<Claims>(&bytes).ok()?.exp
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// True when the token expires within `margin_secs` (or has no readable expiry).
pub fn expires_within(token: &str, margin_secs: u64) -> bool {
    match expiry(token) {
        Some(exp) => exp <= now_secs() + margin_secs,
        None => true,
    }
}

#[cfg(test)]
pub(crate) fn make_token(exp: u64) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload =
        URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":{exp},"userId":"u1","type":"access"}}"#));
    format!("{header}.{payload}.signature")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_expiry() {
        assert_eq!(expiry(&make_token(1_900_000_000)), Some(1_900_000_000));
    }

    #[test]
    fn garbage_has_no_expiry() {
        assert_eq!(expiry("not-a-jwt"), None);
        assert_eq!(expiry("a.!!!.c"), None);
    }

    #[test]
    fn expiry_margin() {
        let now = now_secs();
        assert!(expires_within(&make_token(now + 10), 30));
        assert!(!expires_within(&make_token(now + 3600), 30));
        assert!(expires_within("garbage", 30));
    }
}
