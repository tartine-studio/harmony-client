//! Matches WebSocket requests with their outcome.
//!
//! The server never acknowledges a successful request: success shows up as the
//! broadcast it triggers (which doesn't carry the `requestId`), failure as an
//! `error` frame that does. Each request therefore waits for either.

use tokio::sync::oneshot;

use crate::error::{AppError, AppResult};
use crate::models::{Message, ServerEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    /// `messageCreated` by me in this channel with this exact content.
    Created { channel_id: String, content: String },
    /// `messageUpdated` for this message.
    Updated { message_id: String },
    /// `messageDeleted` for this message.
    Deleted { message_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Message(Message),
    Done,
}

pub type Reply = oneshot::Sender<AppResult<Outcome>>;

struct Entry {
    request_id: String,
    expect: Expect,
    reply: Reply,
}

/// Requests waiting for their outcome, oldest first.
#[derive(Default)]
pub struct Pending {
    entries: Vec<Entry>,
}

impl Pending {
    pub fn add(&mut self, request_id: String, expect: Expect, reply: Reply) {
        self.entries.push(Entry {
            request_id,
            expect,
            reply,
        });
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Forgets a request (timeout): its reply is dropped.
    pub fn remove(&mut self, request_id: &str) {
        self.entries.retain(|e| e.request_id != request_id);
    }

    /// Fails one request (`error` frame with its `requestId`).
    pub fn fail(&mut self, request_id: &str, err: AppError) {
        if let Some(i) = self.entries.iter().position(|e| e.request_id == request_id) {
            let _ = self.entries.remove(i).reply.send(Err(err));
        }
    }

    /// Fails everything (connection lost).
    pub fn fail_all(&mut self, err: AppError) {
        for entry in self.entries.drain(..) {
            let _ = entry.reply.send(Err(err.clone()));
        }
    }

    /// Resolves the oldest request this event answers. Identical messages sent
    /// in a row are therefore matched in order.
    pub fn resolve(&mut self, me: &str, event: &ServerEvent) {
        let Some(i) = self
            .entries
            .iter()
            .position(|e| answers(&e.expect, me, event))
        else {
            return;
        };
        let outcome = match event {
            ServerEvent::MessageCreated(m) | ServerEvent::MessageUpdated(m) => {
                Outcome::Message(m.clone())
            }
            _ => Outcome::Done,
        };
        let _ = self.entries.remove(i).reply.send(Ok(outcome));
    }
}

fn answers(expect: &Expect, me: &str, event: &ServerEvent) -> bool {
    match (expect, event) {
        (
            Expect::Created {
                channel_id,
                content,
            },
            ServerEvent::MessageCreated(m),
        ) => {
            m.user_id.as_deref() == Some(me) && &m.channel_id == channel_id && &m.content == content
        }
        (Expect::Updated { message_id }, ServerEvent::MessageUpdated(m)) => &m.id == message_id,
        (Expect::Deleted { message_id }, ServerEvent::MessageDeleted(d)) => {
            &d.message_id == message_id
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MessageDeleted;

    fn msg(id: &str, user: &str, channel: &str, content: &str) -> Message {
        Message {
            id: id.into(),
            user_id: Some(user.into()),
            channel_id: channel.into(),
            content: content.into(),
            created_at: "2026-11-02T10:00:00Z".into(),
            updated_at: "2026-11-02T10:00:00Z".into(),
        }
    }

    fn created(channel: &str, content: &str) -> Expect {
        Expect::Created {
            channel_id: channel.into(),
            content: content.into(),
        }
    }

    #[test]
    fn identical_sends_resolve_in_order() {
        let mut p = Pending::default();
        let (tx1, mut rx1) = oneshot::channel();
        let (tx2, mut rx2) = oneshot::channel();
        p.add("r1".into(), created("c1", "hi"), tx1);
        p.add("r2".into(), created("c1", "hi"), tx2);

        p.resolve(
            "me",
            &ServerEvent::MessageCreated(msg("m1", "me", "c1", "hi")),
        );
        assert_eq!(
            rx1.try_recv().unwrap(),
            Ok(Outcome::Message(msg("m1", "me", "c1", "hi")))
        );
        assert!(rx2.try_recv().is_err());

        p.resolve(
            "me",
            &ServerEvent::MessageCreated(msg("m2", "me", "c1", "hi")),
        );
        assert_eq!(
            rx2.try_recv().unwrap(),
            Ok(Outcome::Message(msg("m2", "me", "c1", "hi")))
        );
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn ignores_other_authors_channels_and_contents() {
        let mut p = Pending::default();
        let (tx, mut rx) = oneshot::channel();
        p.add("r1".into(), created("c1", "hi"), tx);
        p.resolve(
            "me",
            &ServerEvent::MessageCreated(msg("m1", "bob", "c1", "hi")),
        );
        p.resolve(
            "me",
            &ServerEvent::MessageCreated(msg("m2", "me", "c2", "hi")),
        );
        p.resolve(
            "me",
            &ServerEvent::MessageCreated(msg("m3", "me", "c1", "hello")),
        );
        assert!(rx.try_recv().is_err());
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn edit_and_delete_match_on_message_id() {
        let mut p = Pending::default();
        let (te, mut re) = oneshot::channel();
        let (td, mut rd) = oneshot::channel();
        p.add(
            "e".into(),
            Expect::Updated {
                message_id: "m1".into(),
            },
            te,
        );
        p.add(
            "d".into(),
            Expect::Deleted {
                message_id: "m2".into(),
            },
            td,
        );

        p.resolve(
            "me",
            &ServerEvent::MessageDeleted(MessageDeleted {
                message_id: "m2".into(),
            }),
        );
        assert_eq!(rd.try_recv().unwrap(), Ok(Outcome::Done));
        p.resolve(
            "me",
            &ServerEvent::MessageUpdated(msg("m1", "me", "c1", "edited")),
        );
        assert!(matches!(re.try_recv().unwrap(), Ok(Outcome::Message(m)) if m.content == "edited"));
    }

    #[test]
    fn error_frames_fail_only_their_request() {
        let mut p = Pending::default();
        let (t1, mut r1) = oneshot::channel();
        let (t2, mut r2) = oneshot::channel();
        p.add("r1".into(), created("c1", "a"), t1);
        p.add("r2".into(), created("c1", "b"), t2);
        p.fail("r2", AppError::new("NOT_FOUND", "channel not found"));
        assert_eq!(r2.try_recv().unwrap().unwrap_err().code, "NOT_FOUND");
        assert!(r1.try_recv().is_err());

        p.fail_all(AppError::disconnected());
        assert_eq!(r1.try_recv().unwrap().unwrap_err().code, "DISCONNECTED");
    }

    #[test]
    fn removed_requests_are_not_resolved() {
        let mut p = Pending::default();
        let (tx, mut rx) = oneshot::channel();
        p.add("r1".into(), created("c1", "hi"), tx);
        p.remove("r1");
        assert_eq!(p.len(), 0);
        assert!(rx.try_recv().is_err());
    }
}
