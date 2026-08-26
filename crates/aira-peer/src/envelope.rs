//! Peer ping envelope helper (shared by CLI and tests).

use aira_object::{ContentHash, Keyring, Timestamp};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::PeerError;

/// Build a signed `peer.ping` envelope from the local node identity.
///
/// Signature is canonical over the full envelope descriptor (SEC-2).
pub fn make_peer_ping(
    root: impl AsRef<std::path::Path>,
    text: &str,
) -> Result<ProtocolEnvelope, PeerError> {
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let hash = ContentHash::sha256_bytes(text.as_bytes());
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        aira_object::AiraRef::parse(format!("aira:message:peer-{}", hex::encode(nonce)))
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let created = aira_object::utc_now_rfc3339()?;
    ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: "peer.ping".into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id.clone(),
        target_scope: ScopeDescriptor::local("peer-cli"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(text.to_string()),
        created_at: Timestamp::parse(created).map_err(|e| PeerError::Protocol(e.to_string()))?,
        expires_at: None,
        signature: ProtocolEnvelope::placeholder_signature(&local_id),
    }
    .attach_canonical_signature_with_keyring(&ring, &local_id)
    .map_err(|e| PeerError::Protocol(e.to_string()))
}
