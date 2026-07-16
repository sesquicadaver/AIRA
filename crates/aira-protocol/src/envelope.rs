//! Protocol Envelope / Response types (Issue #71).

use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported Book II protocol identifiers (local C2 subset emphasizes EP/AP/ID/DP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolId {
    #[serde(rename = "AIRA-EP")]
    Event,
    #[serde(rename = "AIRA-AP")]
    Artifact,
    #[serde(rename = "AIRA-ID")]
    Identity,
    #[serde(rename = "AIRA-DP")]
    Discovery,
    #[serde(rename = "AIRA-CAP")]
    Capability,
    #[serde(rename = "AIRA-CRP")]
    Capsule,
    #[serde(rename = "AIRA-FED")]
    Federation,
    #[serde(rename = "AIRA-SET")]
    Settlement,
}

impl ProtocolId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Event => "AIRA-EP",
            Self::Artifact => "AIRA-AP",
            Self::Identity => "AIRA-ID",
            Self::Discovery => "AIRA-DP",
            Self::Capability => "AIRA-CAP",
            Self::Capsule => "AIRA-CRP",
            Self::Federation => "AIRA-FED",
            Self::Settlement => "AIRA-SET",
        }
    }
}

/// Minimal scope descriptor (Schema Pack common).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeDescriptor {
    pub scope_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ScopeDescriptor {
    pub fn local(description: impl Into<String>) -> Self {
        Self {
            scope_type: "local".into(),
            description: Some(description.into()),
        }
    }
}

/// Protocol response status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolStatus {
    Accepted,
    Rejected,
    Deferred,
    RequiresPolicy,
    RequiresEvidence,
    UnsupportedVersion,
    UnsupportedCapability,
    InvalidSignature,
    InvalidArtifact,
    InvariantViolation,
}

/// Envelope validation / adapter errors.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid signature")]
    InvalidSignature,
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(String),
    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("duplicate message: {0}")]
    Duplicate(AiraRef),
    #[error("invalid artifact: {0}")]
    InvalidArtifact(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("schema: {0}")]
    Schema(String),
}

/// Book II Protocol Envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolEnvelope {
    pub protocol_id: ProtocolId,
    pub protocol_version: String,
    pub message_type: String,
    pub message_id: AiraRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub causal_refs: Vec<AiraRef>,
    pub issuer_identity: AiraRef,
    pub target_scope: ScopeDescriptor,
    pub policy_refs: Vec<AiraRef>,
    pub payload_hash: ContentHash,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_ref: Option<String>,
    pub created_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub signature: Signature,
}

impl ProtocolEnvelope {
    /// Reject empty / missing signature material.
    pub fn validate_signature(&self) -> Result<(), ProtocolError> {
        if self.signature.signature_value.trim().is_empty() {
            return Err(ProtocolError::InvalidSignature);
        }
        Ok(())
    }
}

/// Protocol response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolResponse {
    pub message_id: AiraRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    pub status: ProtocolStatus,
    #[serde(default)]
    pub reason_refs: Vec<AiraRef>,
    pub created_at: Timestamp,
    pub signature: Signature,
}

/// Local MVP signature helper.
pub fn local_signature() -> Signature {
    Signature {
        algorithm: "ed25519".into(),
        key_ref: AiraRef::parse("aira:identity:local-test").expect("ref"),
        signature_value: "TESTSIG".into(),
    }
}

/// Local MVP identity ref.
pub fn local_identity() -> AiraRef {
    AiraRef::parse("aira:identity:local-test").expect("ref")
}

/// Fixed MVP timestamp.
pub fn mvp_timestamp() -> Timestamp {
    Timestamp::parse("2026-07-10T12:00:00Z").expect("ts")
}
