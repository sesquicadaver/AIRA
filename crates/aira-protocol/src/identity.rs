//! Local AIRA-ID Identity Descriptor (Issue #74).

use aira_object::{local_test_signature, AiraRef, Signature, Timestamp};
use serde::{Deserialize, Serialize};

use crate::envelope::{mvp_timestamp, ProtocolError};

/// Identity kinds from Schema Pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdentityType {
    User,
    Csu,
    Federation,
    Service,
    Organization,
}

/// Public key material entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKeyEntry {
    pub key_id: String,
    pub algorithm: String,
    pub public_key_material: String,
    pub valid_from: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
}

/// Schema-aligned Identity Descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityDescriptor {
    pub identity_id: AiraRef,
    pub identity_type: IdentityType,
    pub public_keys: Vec<PublicKeyEntry>,
    #[serde(default)]
    pub trust_anchors: Vec<AiraRef>,
    pub policy_refs: Vec<AiraRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_hash: Option<String>,
    pub signature: Signature,
}

impl IdentityDescriptor {
    /// Build a local user identity with one Ed25519 public key hex.
    pub fn local_user(identity_id: &str, public_key_hex: &str) -> Result<Self, ProtocolError> {
        let identity_id =
            AiraRef::parse(identity_id).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let policy = AiraRef::parse("aira:policy:default")
            .map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let desc = Self {
            identity_id: identity_id.clone(),
            identity_type: IdentityType::User,
            public_keys: vec![PublicKeyEntry {
                key_id: "local-ed25519".into(),
                algorithm: "ed25519".into(),
                public_key_material: public_key_hex.into(),
                valid_from: mvp_timestamp(),
                valid_to: None,
            }],
            trust_anchors: vec![],
            policy_refs: vec![policy],
            metadata_hash: None,
            signature: local_test_signature(identity_id.as_str().as_bytes()),
        };
        if desc.public_keys.is_empty() || desc.public_keys[0].public_key_material.is_empty() {
            return Err(ProtocolError::Schema("public key required".into()));
        }
        if aira_object::verify_ed25519(&desc.signature, desc.identity_id.as_str().as_bytes())
            .is_err()
        {
            return Err(ProtocolError::InvalidSignature);
        }
        Ok(desc)
    }
}
