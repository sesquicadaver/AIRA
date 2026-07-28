//! Trusted relay hub + deliver envelopes (Analyze-44).
//!
//! NAT-bound peers keep an outbound session registered on a relay. Senders
//! wrap the original signed envelope in `peer.relay.deliver`; the hub forwards
//! the **inner** envelope to the live target session (courier model).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use aira_object::{ContentHash, Keyring, Timestamp};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::session::{dial, AuthenticatedPeer};

/// Schema tag for relay-deliver payload JSON.
pub const RELAY_DELIVER_SCHEMA: &str = "aira:peer:relay-deliver:v1";

/// Protocol envelope `message_type` for relay deliver.
pub const RELAY_DELIVER_MESSAGE_TYPE: &str = "peer.relay.deliver";

/// Signed relay-deliver payload (JSON in envelope `payload_ref`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelayDeliver {
    pub schema: String,
    pub target_id: String,
    pub inner: ProtocolEnvelope,
}

impl RelayDeliver {
    /// Build a deliver payload wrapping an already-signed inner envelope.
    pub fn wrap(target_id: impl Into<String>, inner: ProtocolEnvelope) -> Self {
        Self {
            schema: RELAY_DELIVER_SCHEMA.into(),
            target_id: target_id.into(),
            inner,
        }
    }

    /// Fail closed on empty target / wrong schema / inner issuer mismatch.
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != RELAY_DELIVER_SCHEMA {
            return Err(PeerError::Protocol(format!(
                "relay-deliver schema mismatch: {}",
                self.schema
            )));
        }
        if self.target_id.trim().is_empty() {
            return Err(PeerError::Protocol("relay-deliver empty target_id".into()));
        }
        if self.inner.signature.key_ref != self.inner.issuer_identity {
            return Err(PeerError::IdentityMismatch);
        }
        Ok(())
    }
}

/// Parse `peer.relay.deliver` from an envelope.
pub fn parse_relay_deliver(env: &ProtocolEnvelope) -> Result<RelayDeliver, PeerError> {
    if env.message_type != RELAY_DELIVER_MESSAGE_TYPE {
        return Err(PeerError::Protocol(format!(
            "expected {RELAY_DELIVER_MESSAGE_TYPE}, got {}",
            env.message_type
        )));
    }
    let raw = env
        .payload_ref
        .as_deref()
        .ok_or_else(|| PeerError::Protocol("relay-deliver missing payload_ref".into()))?;
    let deliver: RelayDeliver = serde_json::from_str(raw)?;
    deliver.validate_shape()?;
    let hash = ContentHash::sha256_bytes(raw.as_bytes());
    if hash != env.payload_hash {
        return Err(PeerError::Protocol(
            "relay-deliver payload_hash mismatch".into(),
        ));
    }
    Ok(deliver)
}

/// Sign a `peer.relay.deliver` envelope from the local node.
pub fn make_relay_deliver_envelope(
    root: impl AsRef<Path>,
    target_id: &str,
    inner: ProtocolEnvelope,
) -> Result<ProtocolEnvelope, PeerError> {
    let root = root.as_ref();
    let deliver = RelayDeliver::wrap(target_id, inner);
    deliver.validate_shape()?;
    let payload = serde_json::to_string(&deliver)?;
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let hash = ContentHash::sha256_bytes(payload.as_bytes());
    let signature = ring
        .sign(&local_id, hash.as_str().as_bytes())
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        aira_object::AiraRef::parse(format!("aira:message:relay-{}", hex::encode(nonce)))
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let created = aira_object::utc_now_rfc3339()?;
    Ok(ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: RELAY_DELIVER_MESSAGE_TYPE.into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id,
        target_scope: ScopeDescriptor::local("peer-relay"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(payload),
        created_at: Timestamp::parse(created).map_err(|e| PeerError::Protocol(e.to_string()))?,
        expires_at: None,
        signature,
    })
}

/// In-memory live session routes for a relay node.
#[derive(Clone, Default)]
pub struct RelayHub {
    routes: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<ProtocolEnvelope>>>>,
}

impl RelayHub {
    /// Create an empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `peer_id` and return the inbound queue for envelopes to forward to them.
    pub fn register(
        &self,
        peer_id: impl Into<String>,
    ) -> mpsc::UnboundedReceiver<ProtocolEnvelope> {
        let (tx, rx) = mpsc::unbounded_channel();
        let id = peer_id.into();
        if let Ok(mut map) = self.routes.lock() {
            map.insert(id, tx);
        }
        rx
    }

    /// Drop registration (session closed).
    pub fn unregister(&self, peer_id: &str) {
        if let Ok(mut map) = self.routes.lock() {
            map.remove(peer_id);
        }
    }

    /// Queue an inner envelope for a registered peer.
    pub fn deliver(&self, target_id: &str, inner: ProtocolEnvelope) -> Result<(), PeerError> {
        let map = self
            .routes
            .lock()
            .map_err(|_| PeerError::Protocol("relay hub lock poisoned".into()))?;
        let tx = map.get(target_id).ok_or_else(|| {
            PeerError::Protocol(format!("relay target not registered: {target_id}"))
        })?;
        tx.send(inner)
            .map_err(|_| PeerError::Protocol(format!("relay target session closed: {target_id}")))
    }

    /// Snapshot of currently registered peer ids (diagnostics / tests).
    pub fn registered(&self) -> Vec<String> {
        self.routes
            .lock()
            .map(|m| {
                let mut ids: Vec<_> = m.keys().cloned().collect();
                ids.sort();
                ids
            })
            .unwrap_or_default()
    }
}

/// Serve one authenticated peer on a relay hub until disconnect.
///
/// Handles inbound `peer.relay.deliver` and flushes queued outbound inners.
pub async fn serve_relay_peer(
    hub: RelayHub,
    mut peer: AuthenticatedPeer,
) -> Result<(), PeerError> {
    let peer_id = peer.peer_id.as_str().to_string();
    let mut rx = hub.register(peer_id.clone());
    let result = async {
        loop {
            tokio::select! {
                biased;
                maybe = rx.recv() => {
                    match maybe {
                        Some(inner) => {
                            peer.send_relayed_envelope(&inner).await?;
                        }
                        None => break,
                    }
                }
                env = peer.recv_envelope() => {
                    let env = env?;
                    if env.message_type == RELAY_DELIVER_MESSAGE_TYPE {
                        let deliver = parse_relay_deliver(&env)?;
                        hub.deliver(&deliver.target_id, deliver.inner)?;
                    }
                    // Non-deliver envelopes on the relay socket are ignored (courier only).
                }
            }
        }
        Ok::<(), PeerError>(())
    }
    .await;
    hub.unregister(&peer_id);
    result
}

/// Send a locally signed envelope to `peer_id`, honoring address-book `via`.
///
/// When `via` is set, dials the relay and sends `peer.relay.deliver` instead of
/// connecting to the target directly.
pub async fn send_envelope_to_peer(
    root: impl AsRef<Path>,
    peer_id: &str,
    envelope: &ProtocolEnvelope,
) -> Result<(), PeerError> {
    let root = root.as_ref();
    let book = AddressBook::load(root)?;
    if let Some(via) = book.via_of(peer_id) {
        let via = via.to_string();
        let outer = make_relay_deliver_envelope(root, peer_id, envelope.clone())?;
        let mut session = dial(root, &via).await?;
        session.send_envelope(&outer).await
    } else {
        let mut session = dial(root, peer_id).await?;
        session.send_envelope(envelope).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::AiraRef;
    use aira_object::Signature;

    fn dummy_inner(issuer: &str) -> ProtocolEnvelope {
        let id = AiraRef::parse(issuer).unwrap();
        ProtocolEnvelope {
            protocol_id: ProtocolId::Identity,
            protocol_version: "0.1".into(),
            message_type: "peer.ping".into(),
            message_id: AiraRef::parse("aira:message:relay-unit").unwrap(),
            correlation_id: None,
            causal_refs: vec![],
            issuer_identity: id.clone(),
            target_scope: ScopeDescriptor::local("t"),
            policy_refs: vec![],
            payload_hash: ContentHash::sha256_bytes(b"z"),
            payload_ref: Some("z".into()),
            created_at: Timestamp::parse("2020-01-01T00:00:00Z").unwrap(),
            expires_at: None,
            signature: Signature {
                algorithm: "Ed25519".into(),
                key_ref: id,
                signature_value: "11".repeat(64),
            },
        }
    }

    #[test]
    fn deliver_rejects_bad_schema_and_empty_target() {
        let mut d = RelayDeliver::wrap("aira:identity:c", dummy_inner("aira:identity:a"));
        d.schema = "bad".into();
        assert!(d.validate_shape().is_err());
        let mut d = RelayDeliver::wrap("  ", dummy_inner("aira:identity:a"));
        d.schema = RELAY_DELIVER_SCHEMA.into();
        assert!(d.validate_shape().is_err());
    }

    #[test]
    fn hub_register_deliver_unregister() {
        let hub = RelayHub::new();
        let mut rx = hub.register("aira:identity:c");
        assert_eq!(hub.registered(), vec!["aira:identity:c".to_string()]);
        let ping = dummy_inner("aira:identity:a");
        hub.deliver("aira:identity:c", ping.clone()).unwrap();
        let got = rx.try_recv().unwrap();
        assert_eq!(got.message_id, ping.message_id);
        hub.unregister("aira:identity:c");
        assert!(hub.deliver("aira:identity:c", ping).is_err());
    }
}
