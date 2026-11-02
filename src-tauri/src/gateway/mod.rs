//! WebSocket connection to the server ("gateway").
//!
//! One background task owns the socket: it authenticates with the
//! `Authorization` header, forwards server events to a [`GatewaySink`] (the
//! frontend), reconnects with backoff, and resolves pending requests.

mod backoff;
mod pending;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderValue, StatusCode, header::AUTHORIZATION};
use tokio_tungstenite::tungstenite::{self, Message as WsMessage};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::api::ApiClient;
use crate::error::{AppError, AppResult, ServerErrorBody};
use crate::models::{GatewayStatus, Message, ServerEvent};
use crate::url::ws_url;
use pending::{Expect, Outcome, Pending};

/// How long a request waits for its broadcast before giving up.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Minimum interval between two `activity` frames.
const ACTIVITY_INTERVAL: Duration = Duration::from_secs(60);

/// Receives what the gateway observes. Implemented by the Tauri layer, which
/// forwards everything to the frontend as events.
pub trait GatewaySink: Send + Sync + 'static {
    fn status(&self, status: GatewayStatus);
    fn event(&self, event: &ServerEvent);
}

struct Outgoing {
    request_id: Option<String>,
    text: String,
}

struct Shared {
    me: String,
    pending: Mutex<Pending>,
    connected: AtomicBool,
    last_sent: Mutex<Option<Instant>>,
}

struct Inner {
    shared: Arc<Shared>,
    out: mpsc::UnboundedSender<Outgoing>,
    task: JoinHandle<()>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Handle on the connection. Cheap to clone; the connection closes when the
/// last handle is dropped or [`stop`](Self::stop) is called.
#[derive(Clone)]
pub struct Gateway(Arc<Inner>);

impl Gateway {
    pub fn start(api: Arc<ApiClient>, me: String, sink: Arc<dyn GatewaySink>) -> Self {
        let shared = Arc::new(Shared {
            me,
            pending: Mutex::new(Pending::default()),
            connected: AtomicBool::new(false),
            last_sent: Mutex::new(None),
        });
        let (out, out_rx) = mpsc::unbounded_channel();
        let task = tokio::spawn(run(api, shared.clone(), sink, out_rx));
        Self(Arc::new(Inner { shared, out, task }))
    }

    pub fn stop(&self) {
        self.0.task.abort();
        self.0.shared.connected.store(false, Ordering::SeqCst);
        self.0
            .shared
            .pending
            .lock()
            .unwrap()
            .fail_all(AppError::disconnected());
    }

    pub fn is_connected(&self) -> bool {
        self.0.shared.connected.load(Ordering::SeqCst)
    }

    pub async fn send_message(&self, channel_id: &str, content: &str) -> AppResult<Message> {
        let expect = Expect::Created {
            channel_id: channel_id.to_owned(),
            content: content.to_owned(),
        };
        let data = json!({ "channelId": channel_id, "content": content });
        expect_message(self.request("sendMessage", data, expect).await?)
    }

    pub async fn edit_message(&self, message_id: &str, content: &str) -> AppResult<Message> {
        let expect = Expect::Updated {
            message_id: message_id.to_owned(),
        };
        let data = json!({ "messageId": message_id, "content": content });
        expect_message(self.request("editMessage", data, expect).await?)
    }

    pub async fn delete_message(&self, message_id: &str) -> AppResult<()> {
        let expect = Expect::Deleted {
            message_id: message_id.to_owned(),
        };
        self.request("deleteMessage", json!({ "messageId": message_id }), expect)
            .await
            .map(|_| ())
    }

    /// The server answers with a `presenceBroadcast` about ourselves, which the
    /// frontend receives as a normal event.
    pub fn set_invisible(&self, invisible: bool) -> AppResult<()> {
        self.fire("presenceUpdate", json!({ "invisible": invisible }))
    }

    /// Reports real user input, at most once a minute. Returns whether a frame
    /// was sent. Any other frame also counts as activity on the server.
    pub fn report_activity(&self) -> bool {
        let recent = self
            .0
            .shared
            .last_sent
            .lock()
            .unwrap()
            .is_some_and(|t| t.elapsed() < ACTIVITY_INTERVAL);
        !recent && self.fire("activity", json!({})).is_ok()
    }

    fn enqueue(&self, request_id: Option<String>, kind: &str, data: Value) -> AppResult<()> {
        if !self.is_connected() {
            return Err(AppError::disconnected());
        }
        let mut frame = json!({ "type": kind, "data": data });
        if let Some(id) = &request_id {
            frame["requestId"] = json!(id);
        }
        self.0
            .out
            .send(Outgoing {
                request_id,
                text: frame.to_string(),
            })
            .map_err(|_| AppError::disconnected())?;
        *self.0.shared.last_sent.lock().unwrap() = Some(Instant::now());
        Ok(())
    }

    fn fire(&self, kind: &str, data: Value) -> AppResult<()> {
        self.enqueue(None, kind, data)
    }

    async fn request(&self, kind: &str, data: Value, expect: Expect) -> AppResult<Outcome> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let (reply, rx) = oneshot::channel();
        self.0
            .shared
            .pending
            .lock()
            .unwrap()
            .add(request_id.clone(), expect, reply);
        if let Err(err) = self.enqueue(Some(request_id.clone()), kind, data) {
            self.0.shared.pending.lock().unwrap().remove(&request_id);
            return Err(err);
        }
        match tokio::time::timeout(REQUEST_TIMEOUT, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(AppError::disconnected()),
            Err(_) => {
                self.0.shared.pending.lock().unwrap().remove(&request_id);
                Err(AppError::timeout())
            }
        }
    }
}

fn expect_message(outcome: Outcome) -> AppResult<Message> {
    match outcome {
        Outcome::Message(m) => Ok(m),
        Outcome::Done => Err(AppError::new(
            AppError::INVALID_RESPONSE,
            "expected a message",
        )),
    }
}

// ----- Connection task ------------------------------------------------------

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

enum ConnectError {
    /// Handshake refused with 401: the token used must be refreshed.
    Unauthorized(String),
    /// The session is gone (refresh rejected): stop for good.
    SessionLost,
    /// Anything else: retry later.
    Retry(String),
}

async fn connect(api: &ApiClient) -> Result<Socket, ConnectError> {
    let token = api.access_token().await.map_err(|e| {
        if e.is_auth_failure() || e.code == AppError::NOT_AUTHENTICATED {
            ConnectError::SessionLost
        } else {
            ConnectError::Retry(e.to_string())
        }
    })?;
    let mut request = ws_url(api.base())
        .as_str()
        .into_client_request()
        .map_err(|e| ConnectError::Retry(e.to_string()))?;
    let bearer = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|e| ConnectError::Retry(e.to_string()))?;
    request.headers_mut().insert(AUTHORIZATION, bearer);

    match tokio_tungstenite::connect_async(request).await {
        Ok((socket, _)) => Ok(socket),
        Err(tungstenite::Error::Http(res)) if res.status() == StatusCode::UNAUTHORIZED => {
            Err(ConnectError::Unauthorized(token))
        }
        Err(e) => Err(ConnectError::Retry(e.to_string())),
    }
}

async fn run(
    api: Arc<ApiClient>,
    shared: Arc<Shared>,
    sink: Arc<dyn GatewaySink>,
    mut out_rx: mpsc::UnboundedReceiver<Outgoing>,
) {
    let mut attempt: u32 = 0;
    let mut first = true;
    loop {
        sink.status(if first {
            GatewayStatus::Connecting
        } else {
            GatewayStatus::Reconnecting
        });
        first = false;

        match connect(&api).await {
            Ok(socket) => {
                attempt = 0;
                // Frames queued while we were offline belong to requests that
                // already failed.
                while let Ok(stale) = out_rx.try_recv() {
                    if let Some(id) = stale.request_id {
                        shared
                            .pending
                            .lock()
                            .unwrap()
                            .fail(&id, AppError::disconnected());
                    }
                }
                shared.connected.store(true, Ordering::SeqCst);
                sink.status(GatewayStatus::Connected);

                let closed_by_app = serve(socket, &shared, sink.as_ref(), &mut out_rx).await;

                shared.connected.store(false, Ordering::SeqCst);
                shared
                    .pending
                    .lock()
                    .unwrap()
                    .fail_all(AppError::disconnected());
                if closed_by_app {
                    sink.status(GatewayStatus::Disconnected);
                    return;
                }
                tracing::info!("gateway connection lost, reconnecting");
            }
            Err(ConnectError::SessionLost) => {
                sink.status(GatewayStatus::Disconnected);
                return;
            }
            Err(ConnectError::Unauthorized(token)) => match api.refresh(Some(&token)).await {
                // Fresh token: retry right away.
                Ok(()) if attempt == 0 => {
                    attempt = 1;
                    continue;
                }
                Ok(()) => {}
                Err(e) if e.is_auth_failure() => {
                    sink.status(GatewayStatus::Disconnected);
                    return;
                }
                Err(e) => tracing::warn!("gateway token refresh failed: {e}"),
            },
            Err(ConnectError::Retry(reason)) => {
                tracing::warn!("gateway connection failed: {reason}")
            }
        }

        sink.status(GatewayStatus::Reconnecting);
        tokio::time::sleep(backoff::delay(attempt, backoff::jitter())).await;
        attempt = attempt.saturating_add(1);
    }
}

/// Pumps one connection until it drops. Returns `true` when the app closed it.
async fn serve(
    socket: Socket,
    shared: &Shared,
    sink: &dyn GatewaySink,
    out_rx: &mut mpsc::UnboundedReceiver<Outgoing>,
) -> bool {
    let (mut write, mut read) = socket.split();
    loop {
        tokio::select! {
            out = out_rx.recv() => match out {
                Some(frame) => {
                    if write.send(WsMessage::Text(frame.text.into())).await.is_err() {
                        if let Some(id) = frame.request_id {
                            shared.pending.lock().unwrap().fail(&id, AppError::disconnected());
                        }
                        return false;
                    }
                }
                None => {
                    let _ = write.close().await;
                    return true;
                }
            },
            incoming = read.next() => match incoming {
                Some(Ok(WsMessage::Text(text))) => handle_frame(&text, shared, sink),
                // tungstenite queues the pong itself; flushing sends it now.
                Some(Ok(WsMessage::Ping(_))) => { let _ = write.flush().await; }
                Some(Ok(WsMessage::Close(_))) | Some(Err(_)) | None => return false,
                Some(Ok(_)) => {}
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    data: Value,
    request_id: Option<String>,
}

fn handle_frame(text: &str, shared: &Shared, sink: &dyn GatewaySink) {
    let Ok(envelope) = serde_json::from_str::<Envelope>(text) else {
        tracing::warn!("unreadable gateway frame");
        return;
    };
    if envelope.kind == "error" {
        let body =
            serde_json::from_value::<ServerErrorBody>(envelope.data).unwrap_or(ServerErrorBody {
                error: "unknown error".into(),
                code: "INTERNAL_ERROR".into(),
            });
        match envelope.request_id {
            Some(id) => shared
                .pending
                .lock()
                .unwrap()
                .fail(&id, AppError::from_server(None, body)),
            None => tracing::warn!(
                "gateway error without request: {} ({})",
                body.error,
                body.code
            ),
        }
        return;
    }
    // Events the app doesn't handle yet (voice…) don't parse and are skipped.
    let Ok(event) = serde_json::from_value::<ServerEvent>(
        json!({ "type": envelope.kind, "data": envelope.data }),
    ) else {
        return;
    };
    sink.event(&event);
    shared.pending.lock().unwrap().resolve(&shared.me, &event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::RecordingHooks;
    use crate::jwt::{make_token, now_secs};
    use crate::models::TokenPair;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};

    #[derive(Default)]
    struct Recorder {
        statuses: Mutex<Vec<GatewayStatus>>,
        events: Mutex<Vec<ServerEvent>>,
    }

    impl GatewaySink for Recorder {
        fn status(&self, status: GatewayStatus) {
            self.statuses.lock().unwrap().push(status);
        }
        fn event(&self, event: &ServerEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    impl Recorder {
        async fn wait_for(&self, status: GatewayStatus, count: usize) {
            for _ in 0..200 {
                if self
                    .statuses
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|s| **s == status)
                    .count()
                    >= count
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            panic!(
                "never reached {status:?} x{count}: {:?}",
                self.statuses.lock().unwrap()
            );
        }
    }

    fn api_for(addr: std::net::SocketAddr, token: &str) -> Arc<ApiClient> {
        let api = ApiClient::new(
            url::Url::parse(&format!("http://{addr}")).unwrap(),
            Arc::new(RecordingHooks::default()),
        );
        api.set_tokens(TokenPair {
            access_token: token.into(),
            refresh_token: "r".into(),
        });
        Arc::new(api)
    }

    /// Accepts one connection, checking the bearer token, and returns it.
    #[allow(clippy::result_large_err)]
    async fn accept(listener: &TcpListener, expected_token: &str) -> WebSocketStream<TcpStream> {
        let (stream, _) = listener.accept().await.unwrap();
        let expected = format!("Bearer {expected_token}");
        tokio_tungstenite::accept_hdr_async(stream, move |req: &Request, res: Response| {
            assert_eq!(req.uri().path(), "/api/ws");
            let auth = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|v| v.to_str().ok());
            if auth == Some(expected.as_str()) {
                Ok(res)
            } else {
                Err(ErrorResponse::new(Some("unauthorized".into())))
            }
        })
        .await
        .unwrap()
    }

    async fn next_json(ws: &mut WebSocketStream<TcpStream>) -> Value {
        loop {
            if let WsMessage::Text(t) = ws.next().await.unwrap().unwrap() {
                return serde_json::from_str(&t).unwrap();
            }
        }
    }

    fn message_json(id: &str, user: &str, content: &str) -> Value {
        json!({ "id": id, "userId": user, "channelId": "c1", "content": content,
                "createdAt": "2026-11-02T10:00:00Z", "updatedAt": "2026-11-02T10:00:00Z" })
    }

    #[tokio::test]
    async fn authenticates_routes_events_and_correlates_requests() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let token = make_token(now_secs() + 900);
        let sink = Arc::new(Recorder::default());
        let gateway = Gateway::start(
            api_for(listener.local_addr().unwrap(), &token),
            "me".into(),
            sink.clone(),
        );

        let mut ws = accept(&listener, &token).await;
        ws.send(WsMessage::Text(
            json!({"type": "presenceSnapshot", "data": {"presences": [{"userId": "me", "status": "online", "invisible": false}]}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
        ws.send(WsMessage::Text(
            json!({"type": "voiceStateSnapshot", "data": {}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
        sink.wait_for(GatewayStatus::Connected, 1).await;

        // Send succeeds when the matching broadcast comes back.
        let g = gateway.clone();
        let send = tokio::spawn(async move { g.send_message("c1", "hello").await });
        let frame = next_json(&mut ws).await;
        assert_eq!(frame["type"], "sendMessage");
        assert_eq!(
            frame["data"],
            json!({"channelId": "c1", "content": "hello"})
        );
        let request_id = frame["requestId"].as_str().unwrap().to_owned();
        ws.send(WsMessage::Text(
            json!({"type": "messageCreated", "data": message_json("m1", "me", "hello")})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
        assert_eq!(send.await.unwrap().unwrap().id, "m1");
        assert!(!request_id.is_empty());

        // Failure comes back as an error frame with the same requestId.
        let g = gateway.clone();
        let delete = tokio::spawn(async move { g.delete_message("m9").await });
        let frame = next_json(&mut ws).await;
        assert_eq!(frame["type"], "deleteMessage");
        ws.send(WsMessage::Text(
            json!({"type": "error", "requestId": frame["requestId"], "data": {"error": "forbidden", "code": "FORBIDDEN"}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
        assert_eq!(delete.await.unwrap().unwrap_err().code, "FORBIDDEN");

        // Activity is throttled: the send above already counted.
        assert!(!gateway.report_activity());

        let events = sink.events.lock().unwrap().clone();
        assert!(
            matches!(&events[0], ServerEvent::PresenceSnapshot(s) if s.presences[0].invisible == Some(false))
        );
        assert!(matches!(&events[1], ServerEvent::MessageCreated(m) if m.id == "m1"));
        assert_eq!(events.len(), 2, "unknown voice events are skipped");
        gateway.stop();
    }

    #[tokio::test]
    async fn reconnects_after_the_server_drops_and_fails_pending_requests() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let token = make_token(now_secs() + 900);
        let sink = Arc::new(Recorder::default());
        let gateway = Gateway::start(
            api_for(listener.local_addr().unwrap(), &token),
            "me".into(),
            sink.clone(),
        );

        let mut ws = accept(&listener, &token).await;
        sink.wait_for(GatewayStatus::Connected, 1).await;
        let g = gateway.clone();
        let send = tokio::spawn(async move { g.send_message("c1", "lost").await });
        let _ = next_json(&mut ws).await;
        drop(ws);

        assert_eq!(send.await.unwrap().unwrap_err().code, "DISCONNECTED");
        sink.wait_for(GatewayStatus::Reconnecting, 1).await;
        assert!(gateway.send_message("c1", "offline").await.is_err());

        let _ws = accept(&listener, &token).await;
        sink.wait_for(GatewayStatus::Connected, 2).await;
        assert!(gateway.is_connected());
        gateway.stop();
    }
}
