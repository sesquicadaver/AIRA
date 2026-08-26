//! Trusted relay hub + deliver envelopes (Analyze-44 / Analyze-58 durable registry).
//!
//! NAT-bound peers keep an outbound session registered on a relay. Senders
//! wrap the original signed envelope in `peer.relay.deliver`; the hub forwards
//! the **inner** envelope to the live target session (courier model).
//!
//! Analyze-58: durable membership metadata in `peers/relay_hub.json` (not live
//! sessions). Optional TTL prunes only **offline** rows.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use aira_object::{utc_now_rfc3339, ContentHash, Keyring, Timestamp};
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

/// Schema tag for durable relay hub registry (Analyze-58).
pub const RELAY_HUB_REGISTRY_SCHEMA: &str = "aira:peer:relay-hub:v1";

/// Recommended TTL when operators enable prune (days).
pub const RELAY_HUB_TTL_DAYS_RECOMMENDED: u64 = 31;

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
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        aira_object::AiraRef::parse(format!("aira:message:relay-{}", hex::encode(nonce)))
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let created = utc_now_rfc3339()?;
    ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: RELAY_DELIVER_MESSAGE_TYPE.into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id.clone(),
        target_scope: ScopeDescriptor::local("peer-relay"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(payload),
        created_at: Timestamp::parse(created).map_err(|e| PeerError::Protocol(e.to_string()))?,
        expires_at: None,
        signature: ProtocolEnvelope::placeholder_signature(&local_id),
    }
    .attach_canonical_signature_with_keyring(&ring, &local_id)
    .map_err(|e| PeerError::Protocol(e.to_string()))
}

/// One durable hub membership row (Analyze-58).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayHubEntry {
    pub identity_id: String,
    /// RFC3339 UTC (human / docs).
    pub last_seen: String,
    /// Unix seconds for TTL math without extra time deps.
    pub last_seen_unix: u64,
    pub online: bool,
}

/// Durable relay hub registry under `peers/relay_hub.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayHubRegistry {
    #[serde(default = "default_registry_schema")]
    pub schema: String,
    #[serde(default)]
    pub entries: Vec<RelayHubEntry>,
}

fn default_registry_schema() -> String {
    RELAY_HUB_REGISTRY_SCHEMA.into()
}

impl Default for RelayHubRegistry {
    fn default() -> Self {
        Self {
            schema: RELAY_HUB_REGISTRY_SCHEMA.into(),
            entries: vec![],
        }
    }
}

impl RelayHubRegistry {
    /// Path to durable registry.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("relay_hub.json")
    }

    /// Load or empty; fail closed on schema mismatch / corrupt JSON.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        let store: Self =
            serde_json::from_str(&raw).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        if store.schema != RELAY_HUB_REGISTRY_SCHEMA {
            return Err(PeerError::Protocol(format!(
                "relay hub registry schema mismatch: {}",
                store.schema
            )));
        }
        Ok(store)
    }

    /// Persist registry (creates `peers/` as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PeerError::AddressBook(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Upsert online=true for `identity_id`.
    pub fn mark_online(&mut self, identity_id: &str) -> Result<(), PeerError> {
        self.upsert(identity_id, true)
    }

    /// Upsert online=false for `identity_id` (keep history).
    pub fn mark_offline(&mut self, identity_id: &str) -> Result<(), PeerError> {
        self.upsert(identity_id, false)
    }

    fn upsert(&mut self, identity_id: &str, online: bool) -> Result<(), PeerError> {
        if identity_id.trim().is_empty() {
            return Err(PeerError::Protocol("relay hub empty identity".into()));
        }
        let last_seen = utc_now_rfc3339().map_err(|e| PeerError::AddressBook(e.to_string()))?;
        let last_seen_unix = unix_now_secs();
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| e.identity_id == identity_id)
        {
            e.last_seen = last_seen;
            e.last_seen_unix = last_seen_unix;
            e.online = online;
        } else {
            self.entries.push(RelayHubEntry {
                identity_id: identity_id.into(),
                last_seen,
                last_seen_unix,
                online,
            });
        }
        self.entries
            .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Drop **offline** rows older than `ttl_days`. Never removes `online:true`.
    pub fn prune_offline_older_than(&mut self, ttl_days: u64, now_unix: u64) -> usize {
        let ttl_secs = ttl_days.saturating_mul(86_400);
        let before = self.entries.len();
        self.entries.retain(|e| {
            if e.online {
                return true;
            }
            now_unix.saturating_sub(e.last_seen_unix) < ttl_secs
        });
        before.saturating_sub(self.entries.len())
    }

    /// Lookup durable entry.
    pub fn get(&self, identity_id: &str) -> Option<&RelayHubEntry> {
        self.entries.iter().find(|e| e.identity_id == identity_id)
    }
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Load registry, optionally prune offline TTL, apply `f`, save. Fail closed.
///
/// Serialized across threads so concurrent relay sessions cannot clobber the JSON file.
pub fn with_relay_hub_registry<R>(
    root: impl AsRef<Path>,
    ttl_days: Option<u64>,
    f: impl FnOnce(&mut RelayHubRegistry) -> Result<R, PeerError>,
) -> Result<R, PeerError> {
    let _guard = REGISTRY_FILE_LOCK
        .lock()
        .map_err(|_| PeerError::Protocol("relay hub registry lock poisoned".into()))?;
    let root = root.as_ref();
    let mut reg = RelayHubRegistry::load(root)?;
    if let Some(days) = ttl_days {
        reg.prune_offline_older_than(days, unix_now_secs());
    }
    let out = f(&mut reg)?;
    if let Some(days) = ttl_days {
        reg.prune_offline_older_than(days, unix_now_secs());
    }
    reg.save(root)?;
    Ok(out)
}

/// Process-wide lock for durable registry read-modify-write.
static REGISTRY_FILE_LOCK: Mutex<()> = Mutex::new(());

/// In-memory live session routes for a relay node.
#[derive(Clone, Default)]
pub struct RelayHub {
    routes: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<ProtocolEnvelope>>>>,
}

impl RelayHub {
    /// Create an empty hub (live routes only; durable registry is separate).
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

    /// Snapshot of currently registered peer ids (live only).
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
/// Persists durable online/offline metadata under `root` (Analyze-58). Optional
/// `ttl_days` prunes stale **offline** rows on each write.
pub async fn serve_relay_peer(
    hub: RelayHub,
    mut peer: AuthenticatedPeer,
    root: impl AsRef<Path>,
    ttl_days: Option<u64>,
) -> Result<(), PeerError> {
    let root = root.as_ref();
    let peer_id = peer.peer_id.as_str().to_string();
    // Durable first: if persist fails, never leave a live route.
    with_relay_hub_registry(root, ttl_days, |reg| reg.mark_online(&peer_id))?;
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
    // Always attempt offline mark; prefer session error if both fail.
    let offline = with_relay_hub_registry(root, ttl_days, |reg| reg.mark_offline(&peer_id));
    match (result, offline) {
        (Err(e), _) => Err(e),
        (Ok(()), Err(e)) => Err(e),
        (Ok(()), Ok(())) => Ok(()),
    }
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
    use tempfile::tempdir;

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

    #[test]
    fn registry_survives_restart_and_marks_offline() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        with_relay_hub_registry(root, None, |r| r.mark_online("aira:identity:c")).unwrap();
        let loaded = RelayHubRegistry::load(root).unwrap();
        assert!(loaded.get("aira:identity:c").unwrap().online);
        // "restart": new load, no live routes
        assert!(RelayHub::new().registered().is_empty());
        with_relay_hub_registry(root, None, |r| r.mark_offline("aira:identity:c")).unwrap();
        let again = RelayHubRegistry::load(root).unwrap();
        assert!(!again.get("aira:identity:c").unwrap().online);
    }

    #[test]
    fn ttl_prunes_stale_offline_only() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let mut reg = RelayHubRegistry::default();
        let now = 1_700_000_000u64;
        reg.entries.push(RelayHubEntry {
            identity_id: "aira:identity:old".into(),
            last_seen: "2020-01-01T00:00:00Z".into(),
            last_seen_unix: now - 40 * 86_400,
            online: false,
        });
        reg.entries.push(RelayHubEntry {
            identity_id: "aira:identity:live".into(),
            last_seen: "2020-01-01T00:00:00Z".into(),
            last_seen_unix: now - 40 * 86_400,
            online: true,
        });
        reg.entries.push(RelayHubEntry {
            identity_id: "aira:identity:fresh".into(),
            last_seen: "2024-01-01T00:00:00Z".into(),
            last_seen_unix: now - 2 * 86_400,
            online: false,
        });
        let n = reg.prune_offline_older_than(31, now);
        assert_eq!(n, 1);
        assert!(reg.get("aira:identity:old").is_none());
        assert!(reg.get("aira:identity:live").unwrap().online);
        assert!(reg.get("aira:identity:fresh").is_some());
        reg.save(root).unwrap();
        assert_eq!(RelayHubRegistry::load(root).unwrap().entries.len(), 2);
    }

    #[test]
    fn ttl_none_retains_stale_offline() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let mut reg = RelayHubRegistry::default();
        let now = 1_700_000_000u64;
        reg.entries.push(RelayHubEntry {
            identity_id: "aira:identity:ancient".into(),
            last_seen: "2019-01-01T00:00:00Z".into(),
            last_seen_unix: now - 400 * 86_400,
            online: false,
        });
        reg.save(root).unwrap();
        with_relay_hub_registry(root, None, |_| Ok(())).unwrap();
        assert!(RelayHubRegistry::load(root)
            .unwrap()
            .get("aira:identity:ancient")
            .is_some());
    }

    #[test]
    fn registry_rejects_bad_schema() {
        let dir = tempdir().unwrap();
        let path = RelayHubRegistry::path(dir.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{\"schema\":\"bad\",\"entries\":[]}\n").unwrap();
        let err = RelayHubRegistry::load(dir.path()).unwrap_err();
        assert!(err.to_string().contains("schema mismatch"), "{err}");
    }
}
