//! C0 Invariant Checker (Issue #34).

use aira_artifact::ArtifactDescriptor;
use aira_event::{EventDescriptor, EventSink, EventType};
use aira_object::{AiraRef, ContentHash, ObjectDescriptor, Signature, Timestamp};
use aira_policy::{PolicyDecisionKind, PolicyGate, PolicyQuery};

use crate::error::{CoreError, InvariantViolation};
use crate::store::ObjectStore;

/// Checks C0 invariants and emits `InvariantViolation` events when needed.
pub struct InvariantChecker {
    producer: AiraRef,
    signer: Signature,
    seq: u64,
}

impl InvariantChecker {
    pub fn new(producer: AiraRef, signer: Signature) -> Self {
        Self {
            producer,
            signer,
            seq: 1,
        }
    }

    /// Record object immutability violation + event.
    pub fn on_object_mutation_attempt(
        &mut self,
        object_id: &AiraRef,
        events: &mut dyn EventSink,
    ) -> Result<(), CoreError> {
        self.emit(
            events,
            object_id,
            InvariantViolation::ObjectImmutability {
                object_id: object_id.clone(),
            },
        )
    }

    /// Record artifact immutability violation + event.
    pub fn on_artifact_mutation_attempt(
        &mut self,
        artifact_id: &AiraRef,
        events: &mut dyn EventSink,
    ) -> Result<(), CoreError> {
        self.emit(
            events,
            artifact_id,
            InvariantViolation::ArtifactImmutability {
                artifact_id: artifact_id.clone(),
            },
        )
    }

    /// Event must carry a non-empty signature value.
    pub fn check_event_signature(
        &mut self,
        event: &EventDescriptor,
        events: &mut dyn EventSink,
    ) -> Result<(), CoreError> {
        if event.signature.signature_value.is_empty() {
            return self.emit(
                events,
                &event.event_id,
                InvariantViolation::MissingEventSignature {
                    event_id: event.event_id.clone(),
                },
            );
        }
        Ok(())
    }

    /// Policy-before-action: evaluate gate; DENY → invariant + event.
    pub fn check_policy_before_action(
        &mut self,
        gate: &mut PolicyGate,
        query: PolicyQuery,
        events: &mut dyn EventSink,
    ) -> Result<PolicyDecisionKind, CoreError> {
        let subject = query.subject.clone();
        let decision = gate
            .check(query, Some(events))
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        if decision.decision == PolicyDecisionKind::Deny {
            // `emit` always returns `Err(CoreError::Invariant(_))` after recording the event.
            self.emit(
                events,
                &subject,
                InvariantViolation::PolicyDenied {
                    subject: subject.clone(),
                },
            )?;
        }
        Ok(decision.decision)
    }

    /// Object create/open equality check.
    pub fn assert_object_unchanged(
        store: &impl ObjectStore,
        handle: &aira_object::Handle,
        expected: &ObjectDescriptor,
    ) -> Result<(), CoreError> {
        let got = store.open(handle)?;
        if &got != expected {
            return Err(CoreError::Invariant(
                InvariantViolation::ObjectImmutability {
                    object_id: expected.object_id.clone(),
                },
            ));
        }
        Ok(())
    }

    /// Artifact payload matches descriptor hash.
    pub fn assert_artifact_hash(
        desc: &ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<(), CoreError> {
        let actual = ContentHash::sha256_bytes(payload);
        if actual != desc.content_hash {
            return Err(CoreError::Invariant(
                InvariantViolation::ArtifactImmutability {
                    artifact_id: desc.artifact_id.clone(),
                },
            ));
        }
        Ok(())
    }

    fn emit(
        &mut self,
        events: &mut dyn EventSink,
        subject: &AiraRef,
        violation: InvariantViolation,
    ) -> Result<(), CoreError> {
        let id = format!("aira:event:inv{}", self.seq);
        self.seq += 1;
        let ev = EventDescriptor {
            event_id: AiraRef::parse(&id).map_err(|e| CoreError::Storage(e.to_string()))?,
            event_type: EventType::InvariantViolation,
            schema_version: "0.1".into(),
            producer_identity: self.producer.clone(),
            causal_refs: vec![],
            object_refs: vec![subject.clone()],
            artifact_refs: vec![],
            policy_refs: vec![],
            payload_hash: ContentHash::parse(
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )
            .map_err(|e| CoreError::Storage(e.to_string()))?,
            payload_ref: None,
            created_at: Timestamp::parse("2026-07-10T12:00:00Z")
                .map_err(|e| CoreError::Storage(e.to_string()))?,
            signature: self.signer.clone(),
        };
        events
            .append(ev)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Err(CoreError::Invariant(violation))
    }
}
