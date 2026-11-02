//! End-to-end check of the core against a real Harmony server.
//!
//! Ignored by default. Run with a server that has at least one text channel:
//!
//! ```sh
//! HARMONY_E2E_URL=http://127.0.0.1:8080 cargo test e2e -- --ignored
//! ```

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::api::{ApiClient, RecordingHooks};
use crate::gateway::{Gateway, GatewaySink};
use crate::models::{ChannelType, GatewayStatus, PresenceStatus, ServerEvent};
use crate::url::normalize_server_url;

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

async fn eventually(what: &str, check: impl Fn() -> bool) {
    for _ in 0..100 {
        if check() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for {what}");
}

#[tokio::test]
#[ignore = "needs a running Harmony server (HARMONY_E2E_URL)"]
async fn e2e_register_chat_and_presence() {
    let base =
        normalize_server_url(&std::env::var("HARMONY_E2E_URL").expect("HARMONY_E2E_URL")).unwrap();
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let email = format!("e2e-{suffix}@example.org");

    // Sign up (no tokens returned), then log in.
    let api = Arc::new(ApiClient::new(
        base.clone(),
        Arc::new(RecordingHooks::default()),
    ));
    api.register("E2E Tester", &email, "password123")
        .await
        .unwrap();
    assert_eq!(
        api.register("E2E Tester", &email, "password123")
            .await
            .unwrap_err()
            .code,
        "REGISTRATION_FAILED"
    );
    assert_eq!(
        api.login(&email, "wrong-password").await.unwrap_err().code,
        "INVALID_CREDENTIALS"
    );
    api.login(&email, "password123").await.unwrap();
    let me = api.me().await.unwrap();
    assert_eq!(me.email, email);

    let channels = api.channels().await.unwrap();
    let text = channels
        .iter()
        .find(|c| c.kind == ChannelType::Text)
        .expect("the server needs a text channel");
    assert!(api.users().await.unwrap().iter().any(|u| u.id == me.id));

    // A second user watching the same server.
    let other_email = format!("e2e-other-{suffix}@example.org");
    let other_api = Arc::new(ApiClient::new(
        base.clone(),
        Arc::new(RecordingHooks::default()),
    ));
    other_api
        .register("Other", &other_email, "password123")
        .await
        .unwrap();
    other_api.login(&other_email, "password123").await.unwrap();
    let other_id = other_api.me().await.unwrap().id;
    let other_sink = Arc::new(Recorder::default());
    let other = Gateway::start(other_api.clone(), other_id.clone(), other_sink.clone());
    eventually("other connected", || other.is_connected()).await;

    let sink = Arc::new(Recorder::default());
    let gateway = Gateway::start(api.clone(), me.id.clone(), sink.clone());
    eventually("connected", || gateway.is_connected()).await;
    eventually("presence snapshot", || {
        sink.events.lock().unwrap().iter().any(|e| {
            matches!(e, ServerEvent::PresenceSnapshot(s)
                if s.presences.iter().any(|p| p.user_id == other_id && p.status == PresenceStatus::Online))
        })
    })
    .await;

    // Send, edit, delete: each resolves from the broadcast.
    let sent = gateway
        .send_message(&text.id, "hello from e2e")
        .await
        .unwrap();
    assert_eq!(sent.user_id.as_deref(), Some(me.id.as_str()));
    eventually("other sees the message", || {
        other_sink
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, ServerEvent::MessageCreated(m) if m.id == sent.id))
    })
    .await;
    let history = api.messages(&text.id).await.unwrap();
    assert!(history.iter().any(|m| m.id == sent.id));

    let edited = gateway
        .edit_message(&sent.id, "edited from e2e")
        .await
        .unwrap();
    assert_eq!(edited.content, "edited from e2e");

    // Someone else's message can't be deleted.
    let theirs = other.send_message(&text.id, "not yours").await.unwrap();
    assert_eq!(
        gateway.delete_message(&theirs.id).await.unwrap_err().code,
        "FORBIDDEN"
    );
    other.delete_message(&theirs.id).await.unwrap();

    gateway.delete_message(&sent.id).await.unwrap();
    assert!(
        !api.messages(&text.id)
            .await
            .unwrap()
            .iter()
            .any(|m| m.id == sent.id)
    );
    assert_eq!(
        gateway.delete_message(&sent.id).await.unwrap_err().code,
        "NOT_FOUND"
    );

    // Invisible: others see us offline, we see our own flag.
    gateway.set_invisible(true).unwrap();
    eventually("own invisible flag", || {
        sink.events.lock().unwrap().iter().any(|e| {
            matches!(e, ServerEvent::PresenceBroadcast(p) if p.user_id == me.id && p.invisible == Some(true))
        })
    })
    .await;
    eventually("others see us offline", || {
        other_sink.events.lock().unwrap().iter().any(|e| {
            matches!(e, ServerEvent::PresenceBroadcast(p) if p.user_id == me.id && p.status == PresenceStatus::Offline)
        })
    })
    .await;
    gateway.set_invisible(false).unwrap();

    // Forced refresh keeps working, and REST calls still succeed.
    api.refresh(None).await.unwrap();
    api.channels().await.unwrap();

    gateway.stop();
    other.stop();
}
