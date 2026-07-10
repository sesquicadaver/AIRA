//! Event descriptor types (Schema Pack §8).

use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use serde::{Deserialize, Serialize};

/// Event types used by MVP / C0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    ProblemSubmitted,
    ContextResolved,
    ReductionCompleted,
    CapsuleCreated,
    CapsuleBound,
    CapsuleCompleted,
    CapsuleFailed,
    VerificationCompleted,
    VerificationFailed,
    ResultPublished,
    ArtifactPublished,
    ArtifactResolved,
    ArtifactInvalid,
    ArtifactSuperseded,
    CapabilityRegistered,
    PolicyEvaluated,
    CSURegistered,
    CSUSuspended,
    CSUFailed,
    InvariantViolation,
    FailureEvidenceCreated,
    ResearchArtifactCreated,
    ArtifactPromotionCandidate,
    CustomEvent,
}

/// Append-only event descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventDescriptor {
    pub event_id: AiraRef,
    pub event_type: EventType,
    pub schema_version: String,
    pub producer_identity: AiraRef,
    pub causal_refs: Vec<AiraRef>,
    pub object_refs: Vec<AiraRef>,
    pub artifact_refs: Vec<AiraRef>,
    pub policy_refs: Vec<AiraRef>,
    pub payload_hash: ContentHash,
    pub payload_ref: Option<String>,
    pub created_at: Timestamp,
    pub signature: Signature,
}
