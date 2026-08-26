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

impl EventDescriptor {
    /// Sign over canonical JSON of this descriptor without the top-level `signature`.
    pub fn attach_canonical_signature(mut self) -> Result<Self, aira_object::CryptoError> {
        let v =
            serde_json::to_value(&self).map_err(|e| aira_object::CryptoError::Io(e.to_string()))?;
        self.signature = aira_object::sign_canonical_descriptor(&self.producer_identity, &v)?;
        Ok(self)
    }

    /// Tenant-isolated sign over the same canonical message as [`Self::attach_canonical_signature`].
    pub fn attach_canonical_signature_for_tenant(
        mut self,
        tenant_csu: &AiraRef,
    ) -> Result<Self, aira_object::CryptoError> {
        let v =
            serde_json::to_value(&self).map_err(|e| aira_object::CryptoError::Io(e.to_string()))?;
        let msg = aira_object::descriptor_signing_message(&v)?;
        self.signature =
            aira_object::signature_for_tenant(tenant_csu, &self.producer_identity, &msg)?;
        Ok(self)
    }

    /// Verify Ed25519 over canonical descriptor hash. No LOCAL_TEST domain fallback.
    pub fn verify_canonical(&self) -> Result<(), aira_object::CryptoError> {
        aira_object::verify_producer_signature_binding(&self.producer_identity, &self.signature)?;
        let v =
            serde_json::to_value(self).map_err(|e| aira_object::CryptoError::Io(e.to_string()))?;
        aira_object::verify_canonical_descriptor(&self.signature, &v)
    }
}
