//! Append-only in-memory event log with local subscriptions.

use std::collections::HashMap;
use std::sync::Arc;

use aira_object::{AiraRef, ContentHash};
use thiserror::Error;

use crate::descriptor::{EventDescriptor, EventType};

/// Event log errors.
#[derive(Debug, Error)]
pub enum EventError {
    #[error("event equivocation: {0}")]
    Equivocation(AiraRef),
    #[error("event immutable: {0}")]
    Immutable(AiraRef),
    #[error("event not found: {0}")]
    NotFound(AiraRef),
    #[error("event missing signature")]
    MissingSignature,
    #[error("invalid event signature")]
    InvalidSignature,
    #[error("secret material not allowed in event payload")]
    SecretMaterial,
}

/// Opaque subscription id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

type Handler = Arc<dyn Fn(&EventDescriptor) + Send + Sync>;

/// Dyn-compatible sink for appending events (Policy / InvariantChecker).
pub trait EventSink {
    fn append(&mut self, event: EventDescriptor) -> Result<(), EventError>;
}

/// Event log query API (object/artifact indexes).
pub trait EventLog: EventSink {
    fn mutate(&mut self, event_id: &AiraRef) -> Result<(), EventError>;
    fn query_by_object_ref(&self, object_ref: &AiraRef) -> Vec<EventDescriptor>;
    fn query_by_artifact_ref(&self, artifact_ref: &AiraRef) -> Vec<EventDescriptor>;
}

/// Local memory event log (no global total order required).
pub struct MemoryEventLog {
    events: Vec<EventDescriptor>,
    /// event_id → canonical content hash (SEC-4 equivocation detection).
    seen_ids: HashMap<String, ContentHash>,
    by_object: HashMap<String, Vec<usize>>,
    by_artifact: HashMap<String, Vec<usize>>,
    subscribers: HashMap<EventType, Vec<(SubscriptionId, Handler)>>,
    next_sub: u64,
}

impl MemoryEventLog {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            seen_ids: HashMap::new(),
            by_object: HashMap::new(),
            by_artifact: HashMap::new(),
            subscribers: HashMap::new(),
            next_sub: 1,
        }
    }

    pub fn all(&self) -> &[EventDescriptor] {
        &self.events
    }

    /// Subscribe by event type (local runtime only; not part of dyn EventLog).
    pub fn subscribe<F>(&mut self, event_type: EventType, handler: F) -> SubscriptionId
    where
        F: Fn(&EventDescriptor) + Send + Sync + 'static,
    {
        let id = SubscriptionId(self.next_sub);
        self.next_sub += 1;
        self.subscribers
            .entry(event_type)
            .or_default()
            .push((id, Arc::new(handler)));
        id
    }
}

impl Default for MemoryEventLog {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSink for MemoryEventLog {
    fn append(&mut self, event: EventDescriptor) -> Result<(), EventError> {
        if event.signature.signature_value.is_empty() {
            return Err(EventError::MissingSignature);
        }
        match event.verify_canonical() {
            Ok(()) => {}
            Err(aira_object::CryptoError::MissingOrLegacy) => {
                return Err(EventError::MissingSignature);
            }
            Err(_) => {
                return Err(EventError::InvalidSignature);
            }
        }
        if payload_contains_secret(event.payload_ref.as_deref()) {
            return Err(EventError::SecretMaterial);
        }
        let id = event.event_id.as_str().to_string();
        let content_hash = event
            .canonical_content_hash()
            .map_err(|_| EventError::InvalidSignature)?;
        if let Some(stored) = self.seen_ids.get(&id) {
            if stored == &content_hash {
                // Idempotent: duplicate delivery has no additional semantic effect.
                return Ok(());
            }
            return Err(EventError::Equivocation(event.event_id.clone()));
        }

        let idx = self.events.len();
        for o in &event.object_refs {
            self.by_object
                .entry(o.as_str().to_string())
                .or_default()
                .push(idx);
        }
        for a in &event.artifact_refs {
            self.by_artifact
                .entry(a.as_str().to_string())
                .or_default()
                .push(idx);
        }

        if let Some(subs) = self.subscribers.get(&event.event_type) {
            for (_, handler) in subs {
                handler(&event);
            }
        }

        self.seen_ids.insert(id, content_hash);
        self.events.push(event);
        Ok(())
    }
}

/// Reject obvious secret material in event payload references (Issue #78).
pub fn payload_contains_secret(payload_ref: Option<&str>) -> bool {
    let Some(p) = payload_ref else {
        return false;
    };
    let lower = p.to_ascii_lowercase();
    const NEEDLES: &[&str] = &[
        "begin private key",
        "begin openssh private key",
        "ed25519_secret=",
        "secret_key=",
        "password=",
        "private_key=",
    ];
    NEEDLES.iter().any(|n| lower.contains(n))
}

impl EventLog for MemoryEventLog {
    fn mutate(&mut self, event_id: &AiraRef) -> Result<(), EventError> {
        Err(EventError::Immutable(event_id.clone()))
    }

    fn query_by_object_ref(&self, object_ref: &AiraRef) -> Vec<EventDescriptor> {
        self.by_object
            .get(object_ref.as_str())
            .map(|idxs| {
                idxs.iter()
                    .filter_map(|i| self.events.get(*i).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn query_by_artifact_ref(&self, artifact_ref: &AiraRef) -> Vec<EventDescriptor> {
        self.by_artifact
            .get(artifact_ref.as_str())
            .map(|idxs| {
                idxs.iter()
                    .filter_map(|i| self.events.get(*i).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}
