//! HTTP client for the Harmony REST API.
//!
//! Owns the token pair: tokens never leave the Rust side. Requests refresh the
//! access token shortly before it expires, and once more when the server
//! answers 401 anyway.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::{AppError, AppResult, ServerErrorBody};
use crate::models::TokenPair;
use crate::{jwt, url as server_url};

/// Refresh the access token when it expires within this many seconds.
const REFRESH_MARGIN_SECS: u64 = 30;

/// Session callbacks, implemented by the Tauri layer (keyring, events) and by
/// test doubles.
pub trait SessionHooks: Send + Sync {
    /// A new token pair was obtained (login or refresh).
    fn tokens_changed(&self, refresh_token: &str);
    /// The refresh token was rejected: the user has to log in again.
    fn session_expired(&self);
}

#[derive(Clone)]
struct Tokens {
    access: String,
    refresh: String,
}

pub struct ApiClient {
    http: reqwest::Client,
    base: Url,
    tokens: RwLock<Option<Tokens>>,
    /// Serializes refreshes so concurrent 401s trigger a single refresh.
    refresh_lock: tokio::sync::Mutex<()>,
    hooks: Arc<dyn SessionHooks>,
}

impl ApiClient {
    pub fn new(base: Url, hooks: Arc<dyn SessionHooks>) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .user_agent(concat!("harmony-client/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("the HTTP client configuration is valid");
        Self {
            http,
            base,
            tokens: RwLock::new(None),
            refresh_lock: tokio::sync::Mutex::new(()),
            hooks,
        }
    }

    pub fn base(&self) -> &Url {
        &self.base
    }

    /// Starts from a stored refresh token only (session restore). Call
    /// [`refresh`](Self::refresh) next to get an access token.
    pub fn set_refresh_token(&self, refresh: String) {
        *self.tokens.write().unwrap() = Some(Tokens {
            access: String::new(),
            refresh,
        });
    }

    pub(crate) fn set_tokens(&self, pair: TokenPair) {
        self.hooks.tokens_changed(&pair.refresh_token);
        *self.tokens.write().unwrap() = Some(Tokens {
            access: pair.access_token,
            refresh: pair.refresh_token,
        });
    }

    fn current(&self) -> Option<Tokens> {
        self.tokens.read().unwrap().clone()
    }

    fn url(&self, path: &str) -> Url {
        server_url::api_url(&self.base, path)
    }

    // ----- Auth -------------------------------------------------------------

    pub async fn login(&self, email: &str, password: &str) -> AppResult<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            email: &'a str,
            password: &'a str,
        }
        let res = self
            .http
            .post(self.url("auth/login"))
            .json(&Body { email, password })
            .send()
            .await?;
        let pair: TokenPair = parse(res).await?;
        self.set_tokens(pair);
        Ok(())
    }

    /// Creates an account. The server returns no tokens: log in afterwards.
    pub async fn register(&self, username: &str, email: &str, password: &str) -> AppResult<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            username: &'a str,
            email: &'a str,
            password: &'a str,
        }
        let res = self
            .http
            .post(self.url("auth/register"))
            .json(&Body {
                username,
                email,
                password,
            })
            .send()
            .await?;
        let _: serde_json::Value = parse(res).await?;
        Ok(())
    }

    /// Exchanges the refresh token for a new pair.
    ///
    /// `stale_access` is the access token the caller found unusable. If another
    /// task already replaced it while we waited for the lock, nothing is done.
    /// Pass `None` to always refresh.
    ///
    /// When the server rejects the refresh token, tokens are dropped and
    /// [`SessionHooks::session_expired`] is called.
    pub async fn refresh(&self, stale_access: Option<&str>) -> AppResult<()> {
        let _guard = self.refresh_lock.lock().await;
        let tokens = self.current().ok_or_else(AppError::not_authenticated)?;
        if let Some(stale) = stale_access
            && tokens.access != stale
            && !tokens.access.is_empty()
        {
            return Ok(());
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            refresh_token: &'a str,
        }
        let res = self
            .http
            .post(self.url("auth/refresh"))
            .json(&Body {
                refresh_token: &tokens.refresh,
            })
            .send()
            .await?;
        match parse::<TokenPair>(res).await {
            Ok(pair) => {
                self.set_tokens(pair);
                Ok(())
            }
            Err(err) => {
                if err.is_auth_failure() {
                    *self.tokens.write().unwrap() = None;
                    self.hooks.session_expired();
                }
                Err(err)
            }
        }
    }

    /// A usable access token, refreshed first if it is about to expire.
    pub async fn access_token(&self) -> AppResult<String> {
        let tokens = self.current().ok_or_else(AppError::not_authenticated)?;
        if !jwt::expires_within(&tokens.access, REFRESH_MARGIN_SECS) {
            return Ok(tokens.access);
        }
        self.refresh(Some(&tokens.access)).await?;
        self.current()
            .map(|t| t.access)
            .ok_or_else(AppError::not_authenticated)
    }

    // ----- Authenticated requests -------------------------------------------

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> AppResult<T> {
        self.authed(Method::GET, path, |req| req).await
    }

    async fn authed<T, F>(&self, method: Method, path: &str, build: F) -> AppResult<T>
    where
        T: DeserializeOwned,
        F: Fn(RequestBuilder) -> RequestBuilder,
    {
        let url = self.url(path);
        let token = self.access_token().await?;
        let res = build(self.http.request(method.clone(), url.clone()))
            .bearer_auth(&token)
            .send()
            .await?;
        if res.status() != StatusCode::UNAUTHORIZED {
            return parse(res).await;
        }
        // The token was refused before its expiry (clock skew, secret rotated…):
        // refresh and try exactly once more.
        self.refresh(Some(&token)).await?;
        let token = self.access_token().await?;
        let res = build(self.http.request(method, url))
            .bearer_auth(&token)
            .send()
            .await?;
        parse(res).await
    }
}

/// Decodes a successful JSON body, or turns `{error, code}` into an `AppError`.
async fn parse<T: DeserializeOwned>(res: Response) -> AppResult<T> {
    let status = res.status();
    if status.is_success() {
        return res.json::<T>().await.map_err(|e| {
            AppError::new(AppError::INVALID_RESPONSE, e.to_string()).with_status(status.as_u16())
        });
    }
    let text = res.text().await.unwrap_or_default();
    Err(match serde_json::from_str::<ServerErrorBody>(&text) {
        Ok(body) => AppError::from_server(Some(status.as_u16()), body),
        Err(_) => {
            AppError::new(&format!("HTTP_{}", status.as_u16()), text).with_status(status.as_u16())
        }
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::jwt::{make_token, now_secs};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[derive(Default)]
    pub struct RecordingHooks {
        pub refreshed: Mutex<Vec<String>>,
        pub expired: AtomicUsize,
    }

    impl SessionHooks for RecordingHooks {
        fn tokens_changed(&self, refresh_token: &str) {
            self.refreshed
                .lock()
                .unwrap()
                .push(refresh_token.to_owned());
        }
        fn session_expired(&self) {
            self.expired.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn client(server: &MockServer, hooks: Arc<RecordingHooks>) -> ApiClient {
        ApiClient::new(Url::parse(&server.uri()).unwrap(), hooks)
    }

    fn pair(access: &str, refresh: &str) -> serde_json::Value {
        serde_json::json!({ "accessToken": access, "refreshToken": refresh })
    }

    #[tokio::test]
    async fn login_stores_tokens_and_notifies() {
        let server = MockServer::start().await;
        let access = make_token(now_secs() + 900);
        Mock::given(method("POST"))
            .and(path("/api/auth/login"))
            .and(body_json(
                serde_json::json!({"email": "a@b.c", "password": "secret123"}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(pair(&access, "r1")))
            .mount(&server)
            .await;
        let hooks = Arc::new(RecordingHooks::default());
        let api = client(&server, hooks.clone());
        api.login("a@b.c", "secret123").await.unwrap();
        assert_eq!(api.access_token().await.unwrap(), access);
        assert_eq!(*hooks.refreshed.lock().unwrap(), vec!["r1"]);
    }

    #[tokio::test]
    async fn server_errors_are_mapped() {
        let server = MockServer::start().await;
        Mock::given(path("/api/auth/login"))
            .respond_with(ResponseTemplate::new(401).set_body_json(
                serde_json::json!({"error": "invalid email or password", "code": "INVALID_CREDENTIALS"}),
            ))
            .mount(&server)
            .await;
        Mock::given(path("/api/auth/register"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Bad Gateway"))
            .mount(&server)
            .await;
        let api = client(&server, Arc::default());
        let err = api.login("a@b.c", "nope").await.unwrap_err();
        assert_eq!(
            (err.code.as_str(), err.status),
            ("INVALID_CREDENTIALS", Some(401))
        );
        let err = api.register("al", "a@b.c", "password").await.unwrap_err();
        assert_eq!((err.code.as_str(), err.status), ("HTTP_502", Some(502)));
    }

    #[tokio::test]
    async fn unreachable_server_is_a_network_error() {
        let api = ApiClient::new(
            Url::parse("http://127.0.0.1:9").unwrap(),
            Arc::new(RecordingHooks::default()),
        );
        assert_eq!(api.login("a@b.c", "x").await.unwrap_err().code, "NETWORK");
    }

    #[tokio::test]
    async fn retries_once_after_401_with_a_single_refresh() {
        let server = MockServer::start().await;
        let old = make_token(now_secs() + 900);
        let new = make_token(now_secs() + 901);
        Mock::given(path("/api/auth/refresh"))
            .and(body_json(serde_json::json!({"refreshToken": "r1"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(pair(&new, "r2")))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(path("/api/users/me"))
            .and(header("authorization", format!("Bearer {old}").as_str()))
            .respond_with(ResponseTemplate::new(401).set_body_json(
                serde_json::json!({"error": "unauthorized", "code": "UNAUTHORIZED"}),
            ))
            .mount(&server)
            .await;
        Mock::given(path("/api/users/me"))
            .and(header("authorization", format!("Bearer {new}").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;

        let hooks = Arc::new(RecordingHooks::default());
        let api = Arc::new(client(&server, hooks.clone()));
        api.set_tokens(TokenPair {
            access_token: old,
            refresh_token: "r1".into(),
        });
        let (a, b, c) = tokio::join!(
            api.get::<serde_json::Value>("users/me"),
            api.get::<serde_json::Value>("users/me"),
            api.get::<serde_json::Value>("users/me"),
        );
        assert!(a.is_ok() && b.is_ok() && c.is_ok());
        assert_eq!(hooks.refreshed.lock().unwrap().last().unwrap(), "r2");
        // `expect(1)` on the refresh mock is checked when the server drops.
    }

    #[tokio::test]
    async fn rejected_refresh_expires_the_session() {
        let server = MockServer::start().await;
        Mock::given(path("/api/auth/refresh"))
            .respond_with(ResponseTemplate::new(401).set_body_json(
                serde_json::json!({"error": "invalid refresh token", "code": "INVALID_TOKEN"}),
            ))
            .mount(&server)
            .await;
        let hooks = Arc::new(RecordingHooks::default());
        let api = client(&server, hooks.clone());
        api.set_refresh_token("old".into());
        let err = api.get::<serde_json::Value>("users/me").await.unwrap_err();
        assert_eq!(err.code, "INVALID_TOKEN");
        assert_eq!(hooks.expired.load(Ordering::SeqCst), 1);
        assert_eq!(
            api.access_token().await.unwrap_err().code,
            "NOT_AUTHENTICATED"
        );
    }

    #[tokio::test]
    async fn network_failure_during_refresh_keeps_the_session() {
        let hooks = Arc::new(RecordingHooks::default());
        let api = ApiClient::new(Url::parse("http://127.0.0.1:9").unwrap(), hooks.clone());
        api.set_refresh_token("r".into());
        assert_eq!(api.refresh(None).await.unwrap_err().code, "NETWORK");
        assert_eq!(hooks.expired.load(Ordering::SeqCst), 0);
        assert!(api.current().is_some());
    }
}
