//! Minimal Policy Gate implementation.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aira_event::{EventDescriptor, EventSink, EventType};
use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use thiserror::Error;

use crate::types::{PolicyDecision, PolicyDecisionKind, PolicyQuery};

/// Policy errors.
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("event emit failed: {0}")]
    Event(String),
}

static GATE_NONCE_SEQ: AtomicU64 = AtomicU64::new(1);

/// Allocate a process-unique gate nonce when callers do not supply [`PolicyGate::with_run_nonce`].
fn allocate_gate_nonce() -> String {
    let n = GATE_NONCE_SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}{n:x}")
}

/// Minimal local Policy Gate.
pub struct PolicyGate {
    allowed_actions: HashSet<String>,
    require_actions: HashSet<String>,
    signer: Signature,
    /// Bound into `aira:event:policy{run_nonce}_{seq}` (#317).
    run_nonce: String,
    seq: u64,
}

impl PolicyGate {
    pub fn new(signer: Signature) -> Self {
        Self {
            allowed_actions: HashSet::new(),
            require_actions: HashSet::new(),
            signer,
            run_nonce: allocate_gate_nonce(),
            seq: 1,
        }
    }

    /// Align policy event ids with a plane/submit run nonce (flow path).
    pub fn with_run_nonce(mut self, run_nonce: impl Into<String>) -> Self {
        self.run_nonce = run_nonce.into();
        self
    }

    pub fn allow_action(&mut self, action: impl Into<String>) {
        self.allowed_actions.insert(action.into());
    }

    pub fn require_action(&mut self, action: impl Into<String>) {
        self.require_actions.insert(action.into());
    }

    /// Evaluate policy; optionally append `PolicyEvaluated` event.
    pub fn check(
        &mut self,
        query: PolicyQuery,
        events: Option<&mut dyn EventSink>,
    ) -> Result<PolicyDecision, PolicyError> {
        let decision = if self.require_actions.contains(&query.action) {
            PolicyDecisionKind::Require
        } else if self.allowed_actions.contains(&query.action) {
            PolicyDecisionKind::Allow
        } else {
            // Unknown controlled action defaults to DENY.
            PolicyDecisionKind::Deny
        };

        let result = PolicyDecision {
            decision,
            requirements: vec![],
            reason_refs: vec![AiraRef::parse("aira:policy:default").unwrap()],
            signature: self.signer.clone(),
        };

        if let Some(log) = events {
            // Unique per gate/run (#317): never reuse `aira:event:policy{N}` across submits.
            let id = format!("aira:event:policy{}_{}", self.run_nonce, self.seq);
            self.seq += 1;
            let ev = EventDescriptor {
                event_id: AiraRef::parse(&id).map_err(|e| PolicyError::Event(e.to_string()))?,
                event_type: EventType::PolicyEvaluated,
                schema_version: "0.1".into(),
                producer_identity: AiraRef::parse("aira:identity:local-test")
                    .map_err(|e| PolicyError::Event(e.to_string()))?,
                causal_refs: vec![],
                object_refs: query.object_refs.clone(),
                artifact_refs: query.artifact_refs.clone(),
                policy_refs: result.reason_refs.clone(),
                payload_hash: ContentHash::parse(
                    "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                )
                .map_err(|e| PolicyError::Event(e.to_string()))?,
                payload_ref: None,
                created_at: Timestamp::parse("2026-07-10T12:00:00Z")
                    .map_err(|e| PolicyError::Event(e.to_string()))?,
                signature: self.signer.clone(),
            }
            .attach_canonical_signature()
            .map_err(|e| PolicyError::Event(e.to_string()))?;
            log.append(ev)
                .map_err(|e| PolicyError::Event(e.to_string()))?;
        }

        Ok(result)
    }
}
