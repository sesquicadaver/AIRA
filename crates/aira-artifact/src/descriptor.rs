//! Artifact descriptor types (Schema Pack §7).

use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use serde::{Deserialize, Serialize};

/// Artifact type enum from Schema Pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    VerifiedResultArtifact,
    ReadySolutionArtifact,
    KnowledgeArtifact,
    EvidenceArtifact,
    BestCurrentHypothesisArtifact,
    NegativeResultArtifact,
    OpenResearchArtifact,
    OperationalArtifact,
    ResearchArtifact,
    PolicyArtifact,
    ContextArtifact,
    ExecutionArtifact,
    ConformanceArtifact,
    CustomArtifact,
}

/// Immutable artifact descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDescriptor {
    pub artifact_id: AiraRef,
    pub artifact_type: ArtifactType,
    pub schema_version: String,
    pub content_hash: ContentHash,
    pub content_ref: String,
    pub producer_identity: AiraRef,
    pub provenance_refs: Vec<AiraRef>,
    pub dependency_refs: Vec<AiraRef>,
    pub policy_refs: Vec<AiraRef>,
    pub signature: Signature,
    pub created_at: Timestamp,
}

impl ArtifactDescriptor {
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
