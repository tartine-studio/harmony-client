//! Server address handling.

use crate::error::{AppError, AppResult};
use url::Url;

/// Normalizes what the user typed in the "server address" field.
///
/// Adds `https://` when no scheme is given, drops any path, query or trailing
/// slash, and only accepts http(s).
pub fn normalize_server_url(input: &str) -> AppResult<Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::new(AppError::INVALID_URL, "empty server address"));
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = Url::parse(&with_scheme)
        .map_err(|e| AppError::new(AppError::INVALID_URL, e.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(AppError::new(
            AppError::INVALID_URL,
            "the server address must be an http(s) URL",
        ));
    }
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

/// Displayable form of a normalized URL, without the trailing slash.
pub fn display(url: &Url) -> String {
    url.as_str().trim_end_matches('/').to_owned()
}

/// `https://host` → `https://host/api/<path>`.
pub fn api_url(base: &Url, path: &str) -> Url {
    let mut url = base.clone();
    url.set_path(&format!("/api/{}", path.trim_start_matches('/')));
    url
}

/// `https://host` → `wss://host/api/ws`.
pub fn ws_url(base: &Url) -> Url {
    let mut url = api_url(base, "ws");
    let scheme = if base.scheme() == "https" {
        "wss"
    } else {
        "ws"
    };
    url.set_scheme(scheme)
        .expect("ws and wss are valid schemes");
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_https_when_missing() {
        let url = normalize_server_url("chat.example.org").unwrap();
        assert_eq!(display(&url), "https://chat.example.org");
    }

    #[test]
    fn keeps_http_and_port() {
        let url = normalize_server_url(" http://localhost:8080/ ").unwrap();
        assert_eq!(display(&url), "http://localhost:8080");
    }

    #[test]
    fn drops_path_and_query() {
        let url = normalize_server_url("https://chat.example.org/foo?bar=1#x").unwrap();
        assert_eq!(display(&url), "https://chat.example.org");
    }

    #[test]
    fn rejects_other_schemes_and_garbage() {
        assert_eq!(
            normalize_server_url("ftp://x.org").unwrap_err().code,
            "INVALID_URL"
        );
        assert_eq!(normalize_server_url("   ").unwrap_err().code, "INVALID_URL");
        assert_eq!(
            normalize_server_url("http://").unwrap_err().code,
            "INVALID_URL"
        );
    }

    #[test]
    fn builds_api_and_ws_urls() {
        let base = normalize_server_url("https://chat.example.org").unwrap();
        assert_eq!(
            api_url(&base, "auth/login").as_str(),
            "https://chat.example.org/api/auth/login"
        );
        assert_eq!(ws_url(&base).as_str(), "wss://chat.example.org/api/ws");
        let local = normalize_server_url("http://127.0.0.1:3000").unwrap();
        assert_eq!(ws_url(&local).as_str(), "ws://127.0.0.1:3000/api/ws");
    }
}
