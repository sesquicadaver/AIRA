//! RendezvousProvider abstraction (QUEUE #235 / Phase N).
//!
//! Ledger-agnostic discovery API. Implementations live outside `aira-core`.
//! EVM adapter: `#236`. Deeper TTL/sequence ledger rules: `#237`.
//! `DISCOVERED ≠ TRUSTED`: provider I/O must not upsert TrustStore.

use std::collections::HashMap;

use aira_object::Timestamp;

use crate::error::PeerError;
use crate::presence::NodePresenceRecord;

/// Stable kind id for adapters (`mock`, later `evm`, …).
pub const RENDEZVOUS_KIND_MOCK: &str = "mock";

/// Ledger-agnostic rendezvous substrate (no EVM/chain types in this trait).
pub trait RendezvousProvider: Send {
    /// First publish of a signed presence for an identity.
    fn publish_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError>;

    /// Replace an existing presence (monotonic sequence expected by adapters).
    fn update_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError>;

    /// Drop identity if expired at `as_of` (RFC3339), or force-remove when `force`.
    fn remove_or_expire_presence(
        &mut self,
        identity_ref: &str,
        as_of: &str,
        force: bool,
    ) -> Result<bool, PeerError>;

    /// All non-expired presence records at `as_of`.
    fn query_active_peers(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError>;

    /// Latest stored record for `identity_ref` (may be expired).
    fn query_identity(&self, identity_ref: &str) -> Result<Option<NodePresenceRecord>, PeerError>;

    /// Active peers that advertise at least one relay endpoint.
    fn query_relays(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError>;

    /// Adapter kind (`mock`, …). Never a ledger brand inside Core.
    fn provider_kind(&self) -> &'static str;
}

/// In-memory deterministic double for CI (no network, no chain).
#[derive(Debug, Default, Clone)]
pub struct MockRendezvousProvider {
    by_identity: HashMap<String, NodePresenceRecord>,
}

impl MockRendezvousProvider {
    /// Empty mock ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored identities (tests / diagnostics).
    pub fn len(&self) -> usize {
        self.by_identity.len()
    }

    /// True when no identities are stored.
    pub fn is_empty(&self) -> bool {
        self.by_identity.is_empty()
    }

    fn admit(record: &NodePresenceRecord) -> Result<(), PeerError> {
        record.verify_canonical_signature()?;
        Ok(())
    }

    fn is_active(record: &NodePresenceRecord, as_of: &str) -> Result<bool, PeerError> {
        let as_of_ts = Timestamp::parse(as_of).map_err(|e| PeerError::Rendezvous(e.to_string()))?;
        let exp = Timestamp::parse(&record.expires_at)
            .map_err(|e| PeerError::Rendezvous(e.to_string()))?;
        Ok(exp.as_str() > as_of_ts.as_str())
    }
}

impl RendezvousProvider for MockRendezvousProvider {
    fn publish_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        Self::admit(&record)?;
        let key = record.identity_ref.clone();
        if self.by_identity.contains_key(&key) {
            return Err(PeerError::Rendezvous(format!(
                "presence already published for {key}; use update_presence"
            )));
        }
        self.by_identity.insert(key, record);
        Ok(())
    }

    fn update_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        Self::admit(&record)?;
        let key = record.identity_ref.clone();
        let Some(prev) = self.by_identity.get(&key) else {
            return Err(PeerError::Rendezvous(format!(
                "presence missing for {key}; use publish_presence"
            )));
        };
        if record.sequence <= prev.sequence {
            return Err(PeerError::Rendezvous(format!(
                "presence sequence must increase (have {}, got {})",
                prev.sequence, record.sequence
            )));
        }
        self.by_identity.insert(key, record);
        Ok(())
    }

    fn remove_or_expire_presence(
        &mut self,
        identity_ref: &str,
        as_of: &str,
        force: bool,
    ) -> Result<bool, PeerError> {
        let Some(rec) = self.by_identity.get(identity_ref) else {
            return Ok(false);
        };
        if force || !Self::is_active(rec, as_of)? {
            self.by_identity.remove(identity_ref);
            return Ok(true);
        }
        Ok(false)
    }

    fn query_active_peers(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        let mut out = Vec::new();
        for rec in self.by_identity.values() {
            if Self::is_active(rec, as_of)? {
                out.push(rec.clone());
            }
        }
        out.sort_by(|a, b| a.identity_ref.cmp(&b.identity_ref));
        Ok(out)
    }

    fn query_identity(&self, identity_ref: &str) -> Result<Option<NodePresenceRecord>, PeerError> {
        Ok(self.by_identity.get(identity_ref).cloned())
    }

    fn query_relays(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        let mut out = Vec::new();
        for rec in self.query_active_peers(as_of)? {
            if !rec.relay_endpoints.is_empty() {
                out.push(rec);
            }
        }
        Ok(out)
    }

    fn provider_kind(&self) -> &'static str {
        RENDEZVOUS_KIND_MOCK
    }
}

/// Kind id for durable local-file rendezvous (CLI / Desktop offline substrate).
pub const RENDEZVOUS_KIND_LOCAL_FILE: &str = "local-file";

/// Schema for `peers/rendezvous_ledger.json`.
pub const RENDEZVOUS_LEDGER_SCHEMA: &str = "aira:peer:rendezvous-ledger:0.1";

/// Durable local rendezvous ledger (file-backed mock semantics; no chain).
#[derive(Debug, Clone)]
pub struct LocalFileRendezvousProvider {
    root: std::path::PathBuf,
    inner: MockRendezvousProvider,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RendezvousLedgerFile {
    schema: String,
    records: Vec<NodePresenceRecord>,
}

impl LocalFileRendezvousProvider {
    /// Load from `<root>/peers/rendezvous_ledger.json` (or empty).
    pub fn open(root: impl AsRef<std::path::Path>) -> Result<Self, PeerError> {
        let root = root.as_ref().to_path_buf();
        let path = Self::path(&root);
        let mut inner = MockRendezvousProvider::new();
        if path.exists() {
            let raw = std::fs::read_to_string(&path).map_err(|e| PeerError::Io(e.to_string()))?;
            let file: RendezvousLedgerFile =
                serde_json::from_str(&raw).map_err(|e| PeerError::Rendezvous(e.to_string()))?;
            if file.schema != RENDEZVOUS_LEDGER_SCHEMA {
                return Err(PeerError::Rendezvous(format!(
                    "rendezvous ledger schema mismatch: {}",
                    file.schema
                )));
            }
            for rec in file.records {
                rec.verify_canonical_signature()?;
                inner.by_identity.insert(rec.identity_ref.clone(), rec);
            }
        }
        Ok(Self { root, inner })
    }

    /// Path to durable ledger.
    pub fn path(root: impl AsRef<std::path::Path>) -> std::path::PathBuf {
        root.as_ref().join("peers").join("rendezvous_ledger.json")
    }

    fn persist(&self) -> Result<(), PeerError> {
        let path = Self::path(&self.root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PeerError::Io(e.to_string()))?;
        }
        let mut records: Vec<_> = self.inner.by_identity.values().cloned().collect();
        records.sort_by(|a, b| a.identity_ref.cmp(&b.identity_ref));
        let file = RendezvousLedgerFile {
            schema: RENDEZVOUS_LEDGER_SCHEMA.into(),
            records,
        };
        let json = serde_json::to_string_pretty(&file)?;
        std::fs::write(&path, format!("{json}\n")).map_err(|e| PeerError::Io(e.to_string()))?;
        Ok(())
    }
}

impl RendezvousProvider for LocalFileRendezvousProvider {
    fn publish_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        self.inner.publish_presence(record)?;
        self.persist()
    }

    fn update_presence(&mut self, record: NodePresenceRecord) -> Result<(), PeerError> {
        self.inner.update_presence(record)?;
        self.persist()
    }

    fn remove_or_expire_presence(
        &mut self,
        identity_ref: &str,
        as_of: &str,
        force: bool,
    ) -> Result<bool, PeerError> {
        let removed = self
            .inner
            .remove_or_expire_presence(identity_ref, as_of, force)?;
        if removed {
            self.persist()?;
        }
        Ok(removed)
    }

    fn query_active_peers(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        self.inner.query_active_peers(as_of)
    }

    fn query_identity(&self, identity_ref: &str) -> Result<Option<NodePresenceRecord>, PeerError> {
        self.inner.query_identity(identity_ref)
    }

    fn query_relays(&self, as_of: &str) -> Result<Vec<NodePresenceRecord>, PeerError> {
        self.inner.query_relays(as_of)
    }

    fn provider_kind(&self) -> &'static str {
        RENDEZVOUS_KIND_LOCAL_FILE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{
        ensure_trust_defaults, sign_with_key, AiraRef, Keyring, TrustStore, LOCAL_TEST_KEY_REF,
    };
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::presence::{
        empty_capabilities_hash, PresenceDirectEndpoint, PresenceDraft, PresenceReachability,
        PresenceRelayEndpoint,
    };

    fn write_node(
        root: &std::path::Path,
        name: &str,
        seed: [u8; 32],
    ) -> (AiraRef, String, Keyring) {
        let paths = NodePaths::new(root);
        fs::create_dir_all(paths.identity_dir()).unwrap();
        let sk = SigningKey::from_bytes(&seed);
        let pub_hex = hex::encode(sk.verifying_key().to_bytes());
        let id = format!("aira:identity:{name}");
        let id_ref = AiraRef::parse(&id).unwrap();
        fs::write(
            paths.identity_key(),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        let sig = sign_with_key(id_ref.clone(), &sk, id.as_bytes());
        let desc = serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": name,
            "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": sig
        });
        fs::write(
            paths.identity_json(),
            serde_json::to_string_pretty(&desc).unwrap(),
        )
        .unwrap();
        let _ = ensure_trust_defaults(root).unwrap();
        let (loaded_id, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded_id, id_ref);
        (id_ref, pub_hex, ring)
    }

    fn sample_direct() -> PresenceDirectEndpoint {
        PresenceDirectEndpoint {
            transport: "tcp-peer".into(),
            host: "127.0.0.1".into(),
            port: 49157,
            reachability_state: PresenceReachability::Unknown,
            observed_at: "2026-09-05T12:00:00Z".into(),
        }
    }

    fn signed_presence(
        root: &std::path::Path,
        id: &AiraRef,
        pub_hex: &str,
        sequence: u64,
        relays: Vec<PresenceRelayEndpoint>,
    ) -> NodePresenceRecord {
        NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pub_hex.into(),
            sequence,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![sample_direct()],
            relay_endpoints: relays,
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap()
    }

    #[test]
    fn mock_publish_query_update_expire() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pub_hex, _) = write_node(root, "rv-alice", [41u8; 32]);
        let mut provider: Box<dyn RendezvousProvider> = Box::new(MockRendezvousProvider::new());
        assert_eq!(provider.provider_kind(), RENDEZVOUS_KIND_MOCK);

        let rec = signed_presence(root, &id, &pub_hex, 1, vec![]);
        provider.publish_presence(rec.clone()).unwrap();
        assert!(provider.publish_presence(rec.clone()).is_err());

        let got = provider.query_identity(id.as_str()).unwrap().unwrap();
        assert_eq!(got.sequence, 1);
        let active = provider.query_active_peers("2026-09-06T00:00:00Z").unwrap();
        assert_eq!(active.len(), 1);

        let updated = signed_presence(
            root,
            &id,
            &pub_hex,
            2,
            vec![PresenceRelayEndpoint {
                relay_identity_ref: "aira:identity:relay".into(),
                relay_endpoint: "127.0.0.1:49157".into(),
                reservation_id: "r1".into(),
                expires_at: "2026-09-12T12:00:00Z".into(),
            }],
        );
        provider.update_presence(updated).unwrap();
        assert_eq!(
            provider.query_relays("2026-09-06T00:00:00Z").unwrap().len(),
            1
        );

        assert!(!provider
            .remove_or_expire_presence(id.as_str(), "2026-09-06T00:00:00Z", false)
            .unwrap());
        assert!(provider
            .remove_or_expire_presence(id.as_str(), "2026-09-13T00:00:00Z", false)
            .unwrap());
        assert!(provider.query_identity(id.as_str()).unwrap().is_none());
    }

    #[test]
    fn mock_rejects_unsigned_and_does_not_trust() {
        let alice_dir = tempdir().unwrap();
        let bob_dir = tempdir().unwrap();
        let (alice_id, alice_pk, _) = write_node(alice_dir.path(), "rv-alice2", [42u8; 32]);
        let (_bob_id, _, _) = write_node(bob_dir.path(), "rv-bob", [44u8; 32]);

        let mut draft = NodePresenceRecord::draft(PresenceDraft {
            identity_ref: alice_id.as_str().into(),
            identity_public_key: alice_pk.clone(),
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![sample_direct()],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap();
        draft.signature.signature_value.clear();

        let mut provider = MockRendezvousProvider::new();
        assert!(provider.publish_presence(draft).is_err());
        assert!(provider.is_empty());

        let signed = signed_presence(alice_dir.path(), &alice_id, &alice_pk, 1, vec![]);
        provider.publish_presence(signed).unwrap();
        let discovered = provider
            .query_identity(alice_id.as_str())
            .unwrap()
            .expect("discovered");
        discovered.verify_canonical_signature().unwrap();

        // Bob's TrustStore must not gain Alice from discovery alone.
        let bob_trust = TrustStore::load(bob_dir.path()).unwrap();
        assert!(!bob_trust
            .entries
            .iter()
            .any(|e| e.identity_id == alice_id.as_str()));
        assert_ne!(alice_id.as_str(), LOCAL_TEST_KEY_REF);
    }

    #[test]
    fn mock_update_requires_higher_sequence() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pub_hex, _) = write_node(root, "rv-seq", [43u8; 32]);
        let mut provider = MockRendezvousProvider::new();
        let rec = signed_presence(root, &id, &pub_hex, 1, vec![]);
        provider.publish_presence(rec).unwrap();
        let same = signed_presence(root, &id, &pub_hex, 1, vec![]);
        assert!(provider.update_presence(same).is_err());
    }

    #[test]
    fn local_file_ledger_survives_reopen() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pub_hex, _) = write_node(root, "rv-file", [45u8; 32]);
        let rec = signed_presence(root, &id, &pub_hex, 1, vec![]);
        {
            let mut p = LocalFileRendezvousProvider::open(root).unwrap();
            p.publish_presence(rec).unwrap();
        }
        let p2 = LocalFileRendezvousProvider::open(root).unwrap();
        let got = p2.query_identity(id.as_str()).unwrap().unwrap();
        assert_eq!(got.sequence, 1);
        assert!(LocalFileRendezvousProvider::path(root).is_file());
        assert_eq!(p2.provider_kind(), RENDEZVOUS_KIND_LOCAL_FILE);
    }
}
