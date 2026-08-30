//! Protocol Envelope / Response types (Issue #71).

use aira_object::{
    descriptor_signing_message, sign_canonical_descriptor, verify_canonical_descriptor, AiraRef,
    ContentHash, Keyring, Signature, Timestamp,
};
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
    Equivocation,
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
    #[error("envelope expired")]
    Expired,
    #[error("envelope clock skew")]
    ClockSkew,
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
    /// Placeholder signature (stripped from canonical hash before sign).
    pub fn placeholder_signature(issuer: &AiraRef) -> Signature {
        Signature {
            algorithm: "ed25519".into(),
            key_ref: issuer.clone(),
            signature_value: String::new(),
        }
    }

    /// Canonical sign over full descriptor; `signature.key_ref` must equal `issuer_identity`.
    pub fn attach_canonical_signature(mut self) -> Result<Self, ProtocolError> {
        let issuer = self.issuer_identity.clone();
        let v = serde_json::to_value(&self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        self.signature =
            sign_canonical_descriptor(&issuer, &v).map_err(|_| ProtocolError::InvalidSignature)?;
        Ok(self)
    }

    /// Canonical sign with an explicit node keyring (peer / CLI paths).
    pub fn attach_canonical_signature_with_keyring(
        mut self,
        ring: &Keyring,
        issuer: &AiraRef,
    ) -> Result<Self, ProtocolError> {
        if self.issuer_identity != *issuer {
            return Err(ProtocolError::InvalidSignature);
        }
        let v = serde_json::to_value(&self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let msg = descriptor_signing_message(&v).map_err(|_| ProtocolError::InvalidSignature)?;
        self.signature = ring
            .sign(issuer, &msg)
            .map_err(|_| ProtocolError::InvalidSignature)?;
        Ok(self)
    }

    /// Verify canonical descriptor signature via process keyring.
    pub fn validate_signature(&self) -> Result<(), ProtocolError> {
        if self.signature.key_ref != self.issuer_identity {
            return Err(ProtocolError::InvalidSignature);
        }
        let v = serde_json::to_value(self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        verify_canonical_descriptor(&self.signature, &v)
            .map_err(|_| ProtocolError::InvalidSignature)
    }

    /// Verify canonical descriptor signature via a supplied verifying keyring.
    pub fn validate_signature_with_keyring(&self, ring: &Keyring) -> Result<(), ProtocolError> {
        if self.signature.key_ref != self.issuer_identity {
            return Err(ProtocolError::InvalidSignature);
        }
        let v = serde_json::to_value(self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let msg = descriptor_signing_message(&v).map_err(|_| ProtocolError::InvalidSignature)?;
        ring.verify(&self.signature, &msg)
            .map_err(|_| ProtocolError::InvalidSignature)
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

impl ProtocolResponse {
    pub fn placeholder_signature(issuer: &AiraRef) -> Signature {
        ProtocolEnvelope::placeholder_signature(issuer)
    }

    pub fn attach_canonical_signature(mut self, issuer: &AiraRef) -> Result<Self, ProtocolError> {
        let v = serde_json::to_value(&self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        self.signature =
            sign_canonical_descriptor(issuer, &v).map_err(|_| ProtocolError::InvalidSignature)?;
        Ok(self)
    }

    pub fn attach_canonical_signature_with_keyring(
        mut self,
        ring: &Keyring,
        issuer: &AiraRef,
    ) -> Result<Self, ProtocolError> {
        let v = serde_json::to_value(&self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let msg = descriptor_signing_message(&v).map_err(|_| ProtocolError::InvalidSignature)?;
        self.signature = ring
            .sign(issuer, &msg)
            .map_err(|_| ProtocolError::InvalidSignature)?;
        Ok(self)
    }

    pub fn validate_signature(&self, issuer: &AiraRef) -> Result<(), ProtocolError> {
        if self.signature.key_ref != *issuer {
            return Err(ProtocolError::InvalidSignature);
        }
        let v = serde_json::to_value(self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        verify_canonical_descriptor(&self.signature, &v)
            .map_err(|_| ProtocolError::InvalidSignature)
    }

    pub fn validate_signature_with_keyring(
        &self,
        ring: &Keyring,
        issuer: &AiraRef,
    ) -> Result<(), ProtocolError> {
        if self.signature.key_ref != *issuer {
            return Err(ProtocolError::InvalidSignature);
        }
        let v = serde_json::to_value(self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let msg = descriptor_signing_message(&v).map_err(|_| ProtocolError::InvalidSignature)?;
        ring.verify(&self.signature, &msg)
            .map_err(|_| ProtocolError::InvalidSignature)
    }
}

/// Local MVP signature helper (real Ed25519 over domain message).
pub fn local_signature() -> Signature {
    aira_object::local_test_signature(aira_object::LOCAL_TEST_DOMAIN_MSG)
}

/// Local MVP identity ref.
pub fn local_identity() -> AiraRef {
    AiraRef::parse("aira:identity:local-test").expect("ref")
}

/// Fixed MVP timestamp.
pub fn mvp_timestamp() -> Timestamp {
    Timestamp::parse("2026-07-10T12:00:00Z").expect("ts")
}
