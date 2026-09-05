//! Rendezvous publish/query product layer (QUEUE #237 / Phase N).
//!
//! Enforces TTL, monotonic sequence, record size, and query caps before
//! delegating to a [`RendezvousProvider`]. Local `peers/rendezvous.json`
//! tracks last publish/query metadata (not a ledger mirror).
//! Live JSON-RPC dial remains out of scope (local double / mock backends).

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::Timestamp;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::PeerError;
use crate::presence::{NodePresenceRecord, PUBLIC_NETWORK_ID};
use crate::rendezvous::RendezvousProvider;

/// Default maximum presence TTL (7 days).
pub const RENDEZVOUS_MAX_TTL_SECS: u64 = 7 * 24 * 60 * 60;
/// Default minimum presence TTL (60 seconds).
pub const RENDEZVOUS_MIN_TTL_SECS: u64 = 60;
/// Max canonical JSON size admitted for publish/update.
pub const RENDEZVOUS_MAX_RECORD_BYTES: usize = 64 * 1024;
/// Cap on query_active / query_relays result length.
pub const RENDEZVOUS_MAX_QUERY_RESULTS: usize = 256;

/// Schema tag for local rendezvous service state.
pub const RENDEZVOUS_STATE_SCHEMA: &str = "aira:peer:rendezvous-state:0.1";

/// Policy knobs for publish/query admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendezvousPublishPolicy {
    pub min_ttl_secs: u64,
    pub max_ttl_secs: u64,
    pub max_record_bytes: usize,
    pub max_query_results: usize,
}

impl Default for RendezvousPublishPolicy {
    fn default() -> Self {
        Self {
            min_ttl_secs: RENDEZVOUS_MIN_TTL_SECS,
            max_ttl_secs: RENDEZVOUS_MAX_TTL_SECS,
            max_record_bytes: RENDEZVOUS_MAX_RECORD_BYTES,
            max_query_results: RENDEZVOUS_MAX_QUERY_RESULTS,
        }
    }
}

/// Wire-shaped view of an EVM contract publish (no network).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmPublishCall {
    pub identity_hash: String,
    pub sequence: u64,
    pub expires_at: String,
    pub record_bytes_hex: String,
}

/// Local service metadata under `.aira/peers/rendezvous.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendezvousLocalState {
    pub schema: String,
    pub provider: String,
    pub network_id: String,
    pub last_query: Option<String>,
    pub last_publish: Option<String>,
    pub local_sequence: u64,
    pub local_presence_hash: Option<String>,
}

impl Default for RendezvousLocalState {
    fn default() -> Self {
        Self {
            schema: RENDEZVOUS_STATE_SCHEMA.into(),
            provider: String::new(),
            network_id: PUBLIC_NETWORK_ID.into(),
            last_query: None,
            last_publish: None,
            local_sequence: 0,
            local_presence_hash: None,
        }
    }
}

impl RendezvousLocalState {
    /// Path: `<root>/peers/rendezvous.json`.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("rendezvous.json")
    }

    /// Load or default empty state.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::Io(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| PeerError::Protocol(e.to_string()))
    }

    /// Persist state (creates `peers/`).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::Io(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, format!("{json}\n")).map_err(|e| PeerError::Io(e.to_string()))?;
        Ok(())
    }
}

/// Parse RFC3339 timestamp into [`OffsetDateTime`].
fn parse_odt(s: &str) -> Result<OffsetDateTime, PeerError> {
    OffsetDateTime::parse(s.trim(), &Rfc3339)
        .map_err(|e| PeerError::Rendezvous(format!("bad timestamp {s}: {e}")))
}

/// TTL seconds between created_at and expires_at.
pub fn presence_ttl_secs(created_at: &str, expires_at: &str) -> Result<u64, PeerError> {
    let _ = Timestamp::parse(created_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
    let _ = Timestamp::parse(expires_at).map_err(|e| PeerError::Protocol(e.to_string()))?;
    let c = parse_odt(created_at)?;
    let e = parse_odt(expires_at)?;
    if e <= c {
        return Err(PeerError::Rendezvous(
            "presence expires_at must be after created_at".into(),
        ));
    }
    let secs = (e - c).whole_seconds();
    if secs < 0 {
        return Err(PeerError::Rendezvous("negative presence TTL".into()));
    }
    Ok(secs as u64)
}

/// Encode Presence as EVM publish call shape (identity_hash + bytes).
pub fn encode_evm_publish_call(record: &NodePresenceRecord) -> Result<EvmPublishCall, PeerError> {
    record.verify_canonical_signature()?;
    let bytes = serde_json::to_vec(record)?;
    Ok(EvmPublishCall {
        identity_hash: crate::evm_rendezvous::evm_identity_hash(&record.identity_ref),
        sequence: record.sequence,
        expires_at: record.expires_at.clone(),
        record_bytes_hex: hex::encode(bytes),
    })
}

/// Publish/query façade over any [`RendezvousProvider`].
pub struct RendezvousClient<'a, P: RendezvousProvider + ?Sized> {
    provider: &'a mut P,
    policy: RendezvousPublishPolicy,
    root: Option<PathBuf>,
}

impl<'a, P: RendezvousProvider + ?Sized> RendezvousClient<'a, P> {
    /// Wrap a provider with default policy (no local state root).
    pub fn new(provider: &'a mut P) -> Self {
        Self {
            provider,
            policy: RendezvousPublishPolicy::default(),
            root: None,
        }
    }

    /// Attach node root for `peers/rendezvous.json` updates.
    pub fn with_root(mut self, root: impl AsRef<Path>) -> Self {
        self.root = Some(root.as_ref().to_path_buf());
        self
    }

    /// Override admission policy.
    pub fn with_policy(mut self, policy: RendezvousPublishPolicy) -> Self {
        self.policy = policy;
        self
    }

    fn admit_record(&self, record: &NodePresenceRecord) -> Result<(), PeerError> {
        record.verify_canonical_signature()?;
        let ttl = presence_ttl_secs(&record.created_at, &record.expires_at)?;
        if ttl < self.policy.min_ttl_secs {
            return Err(PeerError::Rendezvous(format!(
                "presence TTL {ttl}s below min {}",
                self.policy.min_ttl_secs
            )));
        }
        if ttl > self.policy.max_ttl_secs {
            return Err(PeerError::Rendezvous(format!(
                "presence TTL {ttl}s above max {}",
                self.policy.max_ttl_secs
            )));
        }
        let bytes = serde_json::to_vec(record)?;
        if bytes.len() > self.policy.max_record_bytes {
            return Err(PeerError::Rendezvous(format!(
                "presence record {} bytes exceeds max {}",
                bytes.len(),
                self.policy.max_record_bytes
            )));
        }
        Ok(())
    }

    fn touch_state_after_publish(&self, record: &NodePresenceRecord) -> Result<(), PeerError> {
        let Some(root) = &self.root else {
            return Ok(());
        };
        let mut st = RendezvousLocalState::load(root)?;
        st.provider = self.provider.provider_kind().into();
        st.network_id = record.network_id.clone();
        st.last_publish = Some(record.created_at.clone());
        st.local_sequence = record.sequence;
        let bytes = serde_json::to_vec(record)?;
        st.local_presence_hash = Some(
            aira_object::ContentHash::sha256_bytes(&bytes)
                .as_str()
                .into(),
        );
        st.save(root)
    }

    fn touch_state_after_query(&self, as_of: &str) -> Result<(), PeerError> {
        let Some(root) = &self.root else {
            return Ok(());
        };
        let mut st = RendezvousLocalState::load(root)?;
        st.provider = self.provider.provider_kind().into();
        st.last_query = Some(as_of.to_string());
        st.save(root)
    }

    /// Publish with TTL/size/signature policy; encodes EVM call view for diagnostics.
    pub fn publish_presence(
        &mut self,
        record: NodePresenceRecord,
    ) -> Result<EvmPublishCall, PeerError> {
        self.admit_record(&record)?;
        let call = encode_evm_publish_call(&record)?;
        self.provider.publish_presence(record.clone())?;
        self.touch_state_after_publish(&record)?;
        Ok(call)
    }

    /// Update with policy + monotonic sequence (provider also enforces).
    pub fn update_presence(
        &mut self,
        record: NodePresenceRecord,
    ) -> Result<EvmPublishCall, PeerError> {
        self.admit_record(&record)?;
        let call = encode_evm_publish_call(&record)?;
        self.provider.update_presence(record.clone())?;
        self.touch_state_after_publish(&record)?;
        Ok(call)
    }

    /// Active peers at `as_of`, truncated to policy max.
    pub fn query_active_peers(
        &mut self,
        as_of: &str,
    ) -> Result<Vec<NodePresenceRecord>, PeerError> {
        let _ = Timestamp::parse(as_of).map_err(|e| PeerError::Protocol(e.to_string()))?;
        let mut out = self.provider.query_active_peers(as_of)?;
        if out.len() > self.policy.max_query_results {
            out.truncate(self.policy.max_query_results);
        }
        self.touch_state_after_query(as_of)?;
        Ok(out)
    }

    /// Query one identity (no truncation).
    pub fn query_identity(
        &mut self,
        identity_ref: &str,
    ) -> Result<Option<NodePresenceRecord>, PeerError> {
        let got = self.provider.query_identity(identity_ref)?;
        // last_query stamp without requiring as_of — use identity lookup time marker
        if let Some(root) = &self.root {
            let mut st = RendezvousLocalState::load(root)?;
            st.provider = self.provider.provider_kind().into();
            st.last_query = Some(identity_ref.to_string());
            st.save(root)?;
        }
        Ok(got)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, Keyring};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::evm_rendezvous::EvmRendezvousProvider;
    use crate::presence::{
        empty_capabilities_hash, PresenceDirectEndpoint, PresenceDraft, PresenceReachability,
    };
    use crate::rendezvous::MockRendezvousProvider;

    fn write_node(root: &Path, name: &str, seed: [u8; 32]) -> (AiraRef, String) {
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
        let (loaded, _): (AiraRef, Keyring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded, id_ref);
        (id_ref, pub_hex)
    }

    fn signed(
        root: &Path,
        id: &AiraRef,
        pk: &str,
        seq: u64,
        created: &str,
        expires: &str,
    ) -> NodePresenceRecord {
        NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pk.into(),
            sequence: seq,
            created_at: created.into(),
            expires_at: expires.into(),
            direct_endpoints: vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: "127.0.0.1".into(),
                port: 49157,
                reachability_state: PresenceReachability::Unknown,
                observed_at: created.into(),
            }],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(root)
        .unwrap()
    }

    #[test]
    fn publish_query_update_with_ttl_and_state() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pk) = write_node(root, "pq-alice", [61u8; 32]);
        let mut mock = MockRendezvousProvider::new();
        let mut client = RendezvousClient::new(&mut mock).with_root(root);
        let rec = signed(
            root,
            &id,
            &pk,
            1,
            "2026-09-05T12:00:00Z",
            "2026-09-06T12:00:00Z",
        );
        let call = client.publish_presence(rec).unwrap();
        assert!(call.identity_hash.starts_with("sha256:"));
        assert!(!call.record_bytes_hex.is_empty());
        let active = client.query_active_peers("2026-09-05T18:00:00Z").unwrap();
        assert_eq!(active.len(), 1);
        let updated = signed(
            root,
            &id,
            &pk,
            2,
            "2026-09-05T12:00:00Z",
            "2026-09-07T12:00:00Z",
        );
        client.update_presence(updated).unwrap();
        let st = RendezvousLocalState::load(root).unwrap();
        assert_eq!(st.local_sequence, 2);
        assert_eq!(st.provider, "mock");
        assert!(st.last_publish.is_some());
        assert!(st.last_query.is_some());
        assert!(RendezvousLocalState::path(root).is_file());
    }

    #[test]
    fn rejects_ttl_out_of_bounds() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pk) = write_node(root, "pq-ttl", [62u8; 32]);
        let mut mock = MockRendezvousProvider::new();
        let mut client = RendezvousClient::new(&mut mock);
        let too_short = signed(
            root,
            &id,
            &pk,
            1,
            "2026-09-05T12:00:00Z",
            "2026-09-05T12:00:30Z",
        );
        assert!(client.publish_presence(too_short).is_err());
        let too_long = signed(
            root,
            &id,
            &pk,
            1,
            "2026-09-05T12:00:00Z",
            "2026-10-20T12:00:00Z",
        );
        assert!(client.publish_presence(too_long).is_err());
    }

    #[test]
    fn works_over_evm_local_double() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (id, pk) = write_node(root, "pq-evm", [63u8; 32]);
        let mut evm = EvmRendezvousProvider::local_double();
        let mut client = RendezvousClient::new(&mut evm).with_root(root);
        let rec = signed(
            root,
            &id,
            &pk,
            1,
            "2026-09-05T12:00:00Z",
            "2026-09-06T12:00:00Z",
        );
        client.publish_presence(rec).unwrap();
        assert_eq!(
            client
                .query_identity(id.as_str())
                .unwrap()
                .unwrap()
                .sequence,
            1
        );
        let st = RendezvousLocalState::load(root).unwrap();
        assert_eq!(st.provider, "evm");
    }
}
