//! Peer ping envelope helper (shared by CLI and tests).

use aira_object::{ContentHash, Keyring, Timestamp};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::PeerError;

/// Build a signed `peer.ping` envelope from the local node identity.
///
/// Signature is over `payload_hash.as_str().as_bytes()` (strict wire verify in
/// [`crate::AuthenticatedPeer::recv_envelope`]).
pub fn make_peer_ping(
    root: impl AsRef<std::path::Path>,
    text: &str,
) -> Result<ProtocolEnvelope, PeerError> {
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let hash = ContentHash::sha256_bytes(text.as_bytes());
    let signature = ring
        .sign(&local_id, hash.as_str().as_bytes())
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        aira_object::AiraRef::parse(format!("aira:message:peer-{}", hex::encode(nonce)))
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let created = aira_object::utc_now_rfc3339()?;
    Ok(ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: "peer.ping".into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id,
        target_scope: ScopeDescriptor::local("peer-cli"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(text.to_string()),
        created_at: Timestamp::parse(created).map_err(|e| PeerError::Protocol(e.to_string()))?,
        expires_at: None,
        signature,
    })
}
