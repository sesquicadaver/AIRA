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
