//! Trust-delta messages over authenticated peer links (Analyze-36).

use std::path::Path;

use aira_object::{
    sync_trust_verifiers, ContentHash, Keyring, Timestamp, TrustStore, LOCAL_TEST_KEY_REF,
};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::PeerError;

/// Schema / domain tag for trust-delta payload JSON.
pub const TRUST_DELTA_SCHEMA: &str = "aira:peer:trust-delta:v1";

/// Protocol envelope `message_type` for trust-delta.
pub const TRUST_DELTA_MESSAGE_TYPE: &str = "peer.trust.delta";

/// Trust-delta operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustDeltaOp {
    Revoke,
    Rotate,
    Unrevoke,
}

impl TrustDeltaOp {
    /// Parse CLI / wire string.
    pub fn parse(s: &str) -> Result<Self, PeerError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "revoke" => Ok(Self::Revoke),
            "rotate" => Ok(Self::Rotate),
            "unrevoke" => Ok(Self::Unrevoke),
            other => Err(PeerError::Protocol(format!(
                "unknown trust-delta op: {other}"
            ))),
        }
    }
}

/// Signed trust-delta payload (JSON in envelope `payload_ref`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustDelta {
    pub schema: String,
    pub op: TrustDeltaOp,
    /// Subject identity (revoke/unrevoke) or old identity (rotate).
    pub subject_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Successor identity (rotate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_id: Option<String>,
    /// Successor Ed25519 pubkey hex (rotate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_pubkey_hex: Option<String>,
    /// Optional dual-key grace end RFC3339 UTC (rotate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_until: Option<String>,
}

impl TrustDelta {
    /// Build a revoke announcement.
    pub fn revoke(subject_id: impl Into<String>, reason: Option<String>) -> Self {
        Self {
            schema: TRUST_DELTA_SCHEMA.into(),
            op: TrustDeltaOp::Revoke,
            subject_id: subject_id.into(),
            reason,
            new_id: None,
            new_pubkey_hex: None,
            grace_until: None,
        }
    }

    /// Build an unrevoke announcement (CRL clear only).
    pub fn unrevoke(subject_id: impl Into<String>) -> Self {
        Self {
            schema: TRUST_DELTA_SCHEMA.into(),
            op: TrustDeltaOp::Unrevoke,
            subject_id: subject_id.into(),
            reason: None,
            new_id: None,
            new_pubkey_hex: None,
            grace_until: None,
        }
    }

    /// Build a rotate announcement.
    pub fn rotate(
        old_id: impl Into<String>,
        new_id: impl Into<String>,
        new_pubkey_hex: impl Into<String>,
        reason: Option<String>,
        grace_until: Option<String>,
    ) -> Self {
        Self {
            schema: TRUST_DELTA_SCHEMA.into(),
            op: TrustDeltaOp::Rotate,
            subject_id: old_id.into(),
            reason,
            new_id: Some(new_id.into()),
            new_pubkey_hex: Some(new_pubkey_hex.into()),
            grace_until,
        }
    }

    /// Canonical JSON bytes used for `payload_hash`.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PeerError> {
        serde_json::to_vec(self).map_err(|e| PeerError::Protocol(e.to_string()))
    }

    /// Validate schema + op-specific required fields (no TrustStore mutation).
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != TRUST_DELTA_SCHEMA {
            return Err(PeerError::Protocol(format!(
                "unsupported trust-delta schema: {}",
                self.schema
            )));
        }
        let subject = self.subject_id.trim();
        if subject.is_empty() {
            return Err(PeerError::Protocol("trust-delta subject_id empty".into()));
        }
        aira_object::AiraRef::parse(subject)
            .map_err(|e| PeerError::Protocol(format!("subject_id: {e}")))?;
        match self.op {
            TrustDeltaOp::Revoke | TrustDeltaOp::Unrevoke => Ok(()),
            TrustDeltaOp::Rotate => {
                let new_id = self
                    .new_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| PeerError::Protocol("rotate requires new_id".into()))?;
                aira_object::AiraRef::parse(new_id)
                    .map_err(|e| PeerError::Protocol(format!("new_id: {e}")))?;
                let pk = self
                    .new_pubkey_hex
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| PeerError::Protocol("rotate requires new_pubkey_hex".into()))?;
                if pk.len() != 64 {
                    return Err(PeerError::Protocol(
                        "new_pubkey_hex must be 64 hex chars".into(),
                    ));
                }
                Ok(())
            }
        }
    }
}

fn refuse_protected(id: &str, local_id: &str) -> Result<(), PeerError> {
    let id = id.trim();
    if id == LOCAL_TEST_KEY_REF {
        return Err(PeerError::Protocol(
            "trust-delta refuses local-test identity".into(),
        ));
    }
    if id == local_id {
        return Err(PeerError::Protocol(
            "trust-delta refuses local node identity".into(),
        ));
    }
    Ok(())
}

/// Apply a validated trust-delta into local `trust.json` (fail-closed).
///
/// `issuer` must be the authenticated peer that signed the envelope. Does not
/// re-check the envelope signature — caller must use [`crate::AuthenticatedPeer::recv_envelope`].
pub fn apply_trust_delta(
    root: impl AsRef<Path>,
    issuer: &aira_object::AiraRef,
    delta: &TrustDelta,
) -> Result<(), PeerError> {
    let root = root.as_ref();
    delta.validate_shape()?;
    let (local_id, _) = Keyring::load_node_identity(root)?;
    let local = local_id.as_str();

    // Issuer must already be trusted (and not revoked) at apply time.
    let trust_check = TrustStore::load(root)?;
    if trust_check.is_revoked(issuer.as_str()) {
        return Err(PeerError::Revoked(issuer.as_str().into()));
    }
    if !trust_check
        .entries
        .iter()
        .any(|e| e.identity_id == issuer.as_str())
    {
        return Err(PeerError::Untrusted(issuer.as_str().into()));
    }

    refuse_protected(delta.subject_id.trim(), local)?;
    if let Some(new_id) = delta.new_id.as_deref() {
        refuse_protected(new_id, local)?;
    }

    let mut store = TrustStore::load(root)?;
    match delta.op {
        TrustDeltaOp::Revoke => {
            store.revoke(delta.subject_id.trim(), delta.reason.as_deref())?;
        }
        TrustDeltaOp::Unrevoke => {
            store.unrevoke(delta.subject_id.trim())?;
        }
        TrustDeltaOp::Rotate => {
            let new_id = delta.new_id.as_deref().unwrap().trim();
            let pk = delta.new_pubkey_hex.as_deref().unwrap().trim();
            store.rotate(
                delta.subject_id.trim(),
                new_id,
                pk,
                delta.reason.as_deref(),
                delta.grace_until.as_deref(),
            )?;
        }
    }
    store.save(root)?;
    let _ = sync_trust_verifiers(root)?;
    Ok(())
}

/// Parse trust-delta from a verified envelope.
pub fn parse_trust_delta(env: &ProtocolEnvelope) -> Result<TrustDelta, PeerError> {
    if env.message_type != TRUST_DELTA_MESSAGE_TYPE {
        return Err(PeerError::Protocol(format!(
            "expected {TRUST_DELTA_MESSAGE_TYPE}, got {}",
            env.message_type
        )));
    }
    let raw = env
        .payload_ref
        .as_deref()
        .ok_or_else(|| PeerError::Protocol("trust-delta missing payload_ref".into()))?;
    let delta: TrustDelta = serde_json::from_str(raw)?;
    delta.validate_shape()?;
    let expected = ContentHash::sha256_bytes(raw.as_bytes());
    if env.payload_hash != expected {
        return Err(PeerError::Protocol(
            "trust-delta payload_hash mismatch".into(),
        ));
    }
    Ok(delta)
}

/// Build a signed `peer.trust.delta` envelope from the local node identity.
pub fn make_trust_delta_envelope(
    root: impl AsRef<Path>,
    delta: &TrustDelta,
) -> Result<ProtocolEnvelope, PeerError> {
    delta.validate_shape()?;
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let json = String::from_utf8(delta.canonical_bytes()?)
        .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let hash = ContentHash::sha256_bytes(json.as_bytes());
    let signature = ring
        .sign(&local_id, hash.as_str().as_bytes())
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        aira_object::AiraRef::parse(format!("aira:message:trust-delta-{}", hex::encode(nonce)))
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let created = aira_object::utc_now_rfc3339()?;
    Ok(ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: TRUST_DELTA_MESSAGE_TYPE.into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id,
        target_scope: ScopeDescriptor::local("peer-trust-delta"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(json),
        created_at: Timestamp::parse(created).map_err(|e| PeerError::Protocol(e.to_string()))?,
        expires_at: None,
        signature,
    })
}
