//! Trusted-mesh DHT-lite (Analyze-47).
//!
//! Durable local table of identity→addr records with Kademlia-style XOR
//! distance ranking. Announcements travel as signed `peer.dht.announce`
//! envelopes over authenticated peer links — **not** UDP/discv5.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{utc_now_rfc3339, ContentHash, Keyring, Timestamp};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::session::dial;

/// Schema tag for DHT announce payload + on-disk store.
pub const DHT_SCHEMA: &str = "aira:peer:dht:v1";

/// Protocol envelope `message_type` for DHT announce.
pub const DHT_ANNOUNCE_MESSAGE_TYPE: &str = "peer.dht.announce";

/// Default number of closest records returned by find.
pub const DHT_DEFAULT_K: usize = 8;

/// One DHT record (identity → dialable addr).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DhtRecord {
    pub identity_id: String,
    pub addr: String,
    /// sha256 hex of identity_id (no `sha256:` prefix) for XOR ranking.
    pub key_hex: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Signed announce payload (JSON in envelope `payload_ref`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DhtAnnounce {
    pub schema: String,
    pub identity_id: String,
    pub addr: String,
}

impl DhtAnnounce {
    /// Build announce for an identity/addr pair.
    pub fn new(identity_id: impl Into<String>, addr: impl Into<String>) -> Self {
        Self {
            schema: DHT_SCHEMA.into(),
            identity_id: identity_id.into(),
            addr: addr.into(),
        }
    }

    /// Fail closed on empty fields / schema mismatch.
    pub fn validate_shape(&self) -> Result<(), PeerError> {
        if self.schema != DHT_SCHEMA {
            return Err(PeerError::Protocol(format!(
                "dht schema mismatch: {}",
                self.schema
            )));
        }
        if self.identity_id.trim().is_empty() || self.addr.trim().is_empty() {
            return Err(PeerError::Protocol(
                "dht announce empty identity/addr".into(),
            ));
        }
        self.addr
            .parse::<std::net::SocketAddr>()
            .map_err(|e| PeerError::Protocol(format!("dht bad addr {}: {e}", self.addr)))?;
        crate::prime_port::validate_aira_bind(&self.addr)?;
        Ok(())
    }
}

/// Durable DHT table under `peers/dht.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDhtStore {
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default)]
    pub records: Vec<DhtRecord>,
}

fn default_schema() -> String {
    DHT_SCHEMA.into()
}

impl Default for PeerDhtStore {
    fn default() -> Self {
        Self {
            schema: DHT_SCHEMA.into(),
            records: vec![],
        }
    }
}

impl PeerDhtStore {
    /// Path to dht.json.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("dht.json")
    }

    /// Load or empty.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self {
                schema: DHT_SCHEMA.into(),
                records: vec![],
            });
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        let mut store: Self =
            serde_json::from_str(&raw).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        if store.schema.is_empty() {
            store.schema = DHT_SCHEMA.into();
        }
        if store.schema != DHT_SCHEMA {
            return Err(PeerError::Protocol(format!(
                "dht store schema mismatch: {}",
                store.schema
            )));
        }
        Ok(store)
    }

    /// Persist store.
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PeerError::AddressBook(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Upsert a record (latest wins on same identity_id).
    pub fn upsert(
        &mut self,
        identity_id: impl Into<String>,
        addr: impl Into<String>,
        source: Option<String>,
    ) -> Result<(), PeerError> {
        let identity_id = identity_id.into();
        let addr = addr.into();
        addr.parse::<std::net::SocketAddr>()
            .map_err(|e| PeerError::Protocol(format!("dht bad addr {addr}: {e}")))?;
        crate::prime_port::validate_aira_bind(&addr)?;
        let key_hex = dht_key_hex(&identity_id);
        let updated_at = utc_now_rfc3339().map_err(|e| PeerError::AddressBook(e.to_string()))?;
        if let Some(r) = self
            .records
            .iter_mut()
            .find(|r| r.identity_id == identity_id)
        {
            r.addr = addr;
            r.key_hex = key_hex;
            r.updated_at = updated_at;
            if source.is_some() {
                r.source = source;
            }
        } else {
            self.records.push(DhtRecord {
                identity_id,
                addr,
                key_hex,
                updated_at,
                source,
            });
        }
        self.records
            .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Exact lookup by identity_id.
    pub fn get(&self, identity_id: &str) -> Option<&DhtRecord> {
        self.records.iter().find(|r| r.identity_id == identity_id)
    }

    /// Closest `k` records to `target_identity` by XOR distance on key_hex.
    pub fn closest(&self, target_identity: &str, k: usize) -> Vec<&DhtRecord> {
        let target = dht_key_bytes(target_identity);
        let mut scored: Vec<_> = self
            .records
            .iter()
            .map(|r| {
                let key = decode_key_hex(&r.key_hex).unwrap_or([0u8; 32]);
                (xor_distance(&target, &key), r)
            })
            .collect();
        scored.sort_by(|(da, a), (db, b)| match cmp_distance(da, db) {
            Ordering::Equal => a.identity_id.cmp(&b.identity_id),
            o => o,
        });
        scored.into_iter().take(k.max(1)).map(|(_, r)| r).collect()
    }
}

/// sha256(identity_id) as 32 bytes.
pub fn dht_key_bytes(identity_id: &str) -> [u8; 32] {
    decode_key_hex(&dht_key_hex(identity_id)).expect("sha256 is 32 bytes")
}

/// sha256(identity_id) as lowercase hex (no algorithm prefix).
pub fn dht_key_hex(identity_id: &str) -> String {
    let h = ContentHash::sha256_bytes(identity_id.as_bytes());
    h.as_str()
        .strip_prefix("sha256:")
        .unwrap_or(h.as_str())
        .to_ascii_lowercase()
}

fn decode_key_hex(hex_str: &str) -> Result<[u8; 32], PeerError> {
    let bytes = hex::decode(hex_str).map_err(|e| PeerError::Protocol(format!("dht key: {e}")))?;
    if bytes.len() != 32 {
        return Err(PeerError::Protocol(format!(
            "dht key must be 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Bytewise XOR of two 32-byte keys.
pub fn xor_distance(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = a[i] ^ b[i];
    }
    out
}

fn cmp_distance(a: &[u8; 32], b: &[u8; 32]) -> Ordering {
    for i in 0..32 {
        match a[i].cmp(&b[i]) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    Ordering::Equal
}

/// Apply a verified announce into the local DHT store.
pub fn apply_dht_announce(
    root: impl AsRef<Path>,
    issuer: &aira_object::AiraRef,
    announce: &DhtAnnounce,
) -> Result<(), PeerError> {
    announce.validate_shape()?;
    // Originator must match announced identity (no spoofed puts).
    if announce.identity_id != issuer.as_str() {
        return Err(PeerError::IdentityMismatch);
    }
    let root = root.as_ref();
    let mut store = PeerDhtStore::load(root)?;
    store.upsert(
        announce.identity_id.clone(),
        announce.addr.clone(),
        Some(format!("peer:{}", issuer.as_str())),
    )?;
    store.save(root)
}

/// Promote a DHT identity/addr into the authoritative address book (Analyze-57).
///
/// Preserves existing `via`. Callers must be opt-in (`--apply-book`); this never runs from
/// default DHT apply/find paths.
pub fn promote_dht_to_address_book(
    root: impl AsRef<Path>,
    identity_id: &str,
    addr: &str,
) -> Result<(), PeerError> {
    addr.parse::<std::net::SocketAddr>()
        .map_err(|e| PeerError::Protocol(format!("dht book promote bad addr {addr}: {e}")))?;
    if identity_id.trim().is_empty() {
        return Err(PeerError::Protocol(
            "dht book promote empty identity".into(),
        ));
    }
    let root = root.as_ref();
    let mut book = AddressBook::load(root)?;
    book.upsert_addr_preserve_via(identity_id, addr)?;
    book.save(root)
}

/// Opt-in exact-hit promote from local DHT into address book (Analyze-57 find `--apply-book`).
///
/// Returns `Ok(None)` when there is no exact record (closest must not be promoted).
pub fn apply_book_exact_from_dht_find(
    root: impl AsRef<Path>,
    key_ref: &str,
) -> Result<Option<(String, String)>, PeerError> {
    let root = root.as_ref();
    let store = PeerDhtStore::load(root)?;
    let Some(exact) = store.get(key_ref) else {
        return Ok(None);
    };
    let id = exact.identity_id.clone();
    let addr = exact.addr.clone();
    promote_dht_to_address_book(root, &id, &addr)?;
    Ok(Some((id, addr)))
}

/// Apply verified announce into DHT; when `apply_book`, promote to address book **first**
/// so a book failure leaves DHT unchanged (Analyze-57).
pub fn apply_dht_announce_maybe_book(
    root: impl AsRef<Path>,
    issuer: &aira_object::AiraRef,
    announce: &DhtAnnounce,
    apply_book: bool,
) -> Result<(), PeerError> {
    announce.validate_shape()?;
    if announce.identity_id != issuer.as_str() {
        return Err(PeerError::IdentityMismatch);
    }
    if apply_book {
        promote_dht_to_address_book(root.as_ref(), &announce.identity_id, &announce.addr)?;
    }
    apply_dht_announce(root, issuer, announce)
}

/// Parse `peer.dht.announce` from a verified envelope.
pub fn parse_dht_announce(env: &ProtocolEnvelope) -> Result<DhtAnnounce, PeerError> {
    if env.message_type != DHT_ANNOUNCE_MESSAGE_TYPE {
        return Err(PeerError::Protocol(format!(
            "expected {DHT_ANNOUNCE_MESSAGE_TYPE}, got {}",
            env.message_type
        )));
    }
    let raw = env
        .payload_ref
        .as_deref()
        .ok_or_else(|| PeerError::Protocol("dht announce missing payload_ref".into()))?;
    let announce: DhtAnnounce = serde_json::from_str(raw)?;
    announce.validate_shape()?;
    let expected = ContentHash::sha256_bytes(raw.as_bytes());
    if env.payload_hash != expected {
        return Err(PeerError::Protocol(
            "dht announce payload_hash mismatch".into(),
        ));
    }
    Ok(announce)
}

/// Sign a DHT announce for the local node identity (must match announce.identity_id).
pub fn make_dht_announce_envelope(
    root: impl AsRef<Path>,
    announce: &DhtAnnounce,
) -> Result<ProtocolEnvelope, PeerError> {
    announce.validate_shape()?;
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    if announce.identity_id != local_id.as_str() {
        return Err(PeerError::Protocol(
            "dht announce identity must be local node identity".into(),
        ));
    }
    let json = serde_json::to_string(announce)?;
    let hash = ContentHash::sha256_bytes(json.as_bytes());
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        aira_object::AiraRef::parse(format!("aira:message:dht-{}", hex::encode(nonce)))
            .map_err(|e| PeerError::Protocol(e.to_string()))?;
    let created = utc_now_rfc3339()?;
    ProtocolEnvelope {
        protocol_id: ProtocolId::Discovery,
        protocol_version: "0.1".into(),
        message_type: DHT_ANNOUNCE_MESSAGE_TYPE.into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id.clone(),
        target_scope: ScopeDescriptor::local("peer-dht"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(json),
        created_at: Timestamp::parse(created).map_err(|e| PeerError::Protocol(e.to_string()))?,
        expires_at: None,
        signature: ProtocolEnvelope::placeholder_signature(&local_id),
    }
    .attach_canonical_signature_with_keyring(&ring, &local_id)
    .map_err(|e| PeerError::Protocol(e.to_string()))
}

/// Announce local listen addr into DHT and fan out to address-book peers (best-effort).
pub async fn dht_announce_to_peers(
    root: impl AsRef<Path>,
    addr: &str,
) -> Result<Vec<(String, bool, Option<String>)>, PeerError> {
    let root = root.as_ref();
    let (local_id, _) = Keyring::load_node_identity(root)?;
    let announce = DhtAnnounce::new(local_id.as_str(), addr);
    announce.validate_shape()?;
    let mut store = PeerDhtStore::load(root)?;
    store.upsert(local_id.as_str(), addr, Some("local".into()))?;
    store.save(root)?;

    let env = make_dht_announce_envelope(root, &announce)?;
    let book = AddressBook::load(root)?;
    let mut out = Vec::new();
    for peer in &book.peers {
        if peer.identity_id == local_id.as_str() {
            continue;
        }
        match dial(root, &peer.identity_id).await {
            Ok(mut session) => match session.send_envelope(&env).await {
                Ok(()) => out.push((peer.identity_id.clone(), true, None)),
                Err(e) => out.push((peer.identity_id.clone(), false, Some(e.to_string()))),
            },
            Err(e) => out.push((peer.identity_id.clone(), false, Some(e.to_string()))),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn xor_distance_identity_and_ordering() {
        let a = [0u8; 32];
        let mut b = [0u8; 32];
        b[31] = 1;
        let d = xor_distance(&a, &b);
        assert_eq!(d[31], 1);
        assert_eq!(cmp_distance(&a, &a), Ordering::Equal);
        assert_eq!(cmp_distance(&a, &d), Ordering::Less);
    }

    #[test]
    fn store_upsert_and_closest() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let mut store = PeerDhtStore::default();
        store
            .upsert("aira:identity:a", "127.0.0.1:49157", Some("local".into()))
            .unwrap();
        store
            .upsert("aira:identity:b", "127.0.0.1:49171", None)
            .unwrap();
        store.save(root).unwrap();
        let loaded = PeerDhtStore::load(root).unwrap();
        assert_eq!(loaded.records.len(), 2);
        assert!(loaded.get("aira:identity:a").is_some());
        let closest = loaded.closest("aira:identity:a", 1);
        assert_eq!(closest[0].identity_id, "aira:identity:a");
    }

    #[test]
    fn announce_rejects_spoofed_identity() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let issuer = aira_object::AiraRef::parse("aira:identity:alice").unwrap();
        let bad = DhtAnnounce::new("aira:identity:bob", "127.0.0.1:49157");
        let err = apply_dht_announce(root, &issuer, &bad).unwrap_err();
        assert!(matches!(err, PeerError::IdentityMismatch));
        let non_prime = DhtAnnounce::new("aira:identity:bob", "127.0.0.1:9");
        assert!(non_prime.validate_shape().is_err());
        let composite = DhtAnnounce::new("aira:identity:bob", "127.0.0.1:50000");
        assert!(composite.validate_shape().is_err());
    }

    #[test]
    fn promote_rejects_bad_addr() {
        let dir = tempdir().unwrap();
        let err =
            promote_dht_to_address_book(dir.path(), "aira:identity:x", "not-an-addr").unwrap_err();
        assert!(err.to_string().contains("bad addr"), "{err}");
    }
}
