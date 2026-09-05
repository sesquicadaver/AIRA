//! Durable peer discovery journal (Analyze-43).
//!
//! Observational memory of peers we have seen — **not** a DHT and not the dial source
//! (dial still uses [`crate::AddressBook`]).

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::utc_now_rfc3339;
use serde::{Deserialize, Serialize};

use crate::error::PeerError;

/// How we learned about a peer sighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    Direct,
    Gossip,
    /// Observed via rendezvous ledger Presence (DISCOVERED; not TRUSTED).
    Rendezvous,
}

/// One durable discovery record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryEntry {
    pub identity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
    pub last_seen: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learned_from: Option<String>,
    pub source: DiscoverySource,
}

/// Local discovery journal under `peers/discovery.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDiscoveryStore {
    #[serde(default)]
    pub peers: Vec<DiscoveryEntry>,
}

impl PeerDiscoveryStore {
    /// Path to discovery.json for a node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("discovery.json")
    }

    /// Load or empty store.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Persist store (creates `peers/` as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PeerError::AddressBook(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Upsert a sighting (latest wins on same identity_id).
    pub fn record(
        &mut self,
        identity_id: impl Into<String>,
        addr: Option<String>,
        learned_from: Option<String>,
        source: DiscoverySource,
    ) -> Result<(), PeerError> {
        let identity_id = identity_id.into();
        let last_seen = utc_now_rfc3339().map_err(|e| PeerError::AddressBook(e.to_string()))?;
        if let Some(e) = self.peers.iter_mut().find(|e| e.identity_id == identity_id) {
            e.last_seen = last_seen;
            e.source = source;
            if addr.is_some() {
                e.addr = addr;
            }
            if learned_from.is_some() {
                e.learned_from = learned_from;
            }
        } else {
            self.peers.push(DiscoveryEntry {
                identity_id,
                addr,
                last_seen,
                learned_from,
                source,
            });
        }
        self.peers.sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Convenience: load → record → save.
    pub fn record_and_save(
        root: impl AsRef<Path>,
        identity_id: impl Into<String>,
        addr: Option<String>,
        learned_from: Option<String>,
        source: DiscoverySource,
    ) -> Result<(), PeerError> {
        let root = root.as_ref();
        let mut store = Self::load(root)?;
        store.record(identity_id, addr, learned_from, source)?;
        store.save(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn discovery_record_persists_and_upserts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        PeerDiscoveryStore::record_and_save(
            root,
            "aira:identity:a",
            Some("127.0.0.1:1".into()),
            Some("aira:identity:b".into()),
            DiscoverySource::Gossip,
        )
        .unwrap();
        PeerDiscoveryStore::record_and_save(
            root,
            "aira:identity:a",
            Some("127.0.0.1:2".into()),
            None,
            DiscoverySource::Direct,
        )
        .unwrap();
        let store = PeerDiscoveryStore::load(root).unwrap();
        assert_eq!(store.peers.len(), 1);
        assert_eq!(store.peers[0].addr.as_deref(), Some("127.0.0.1:2"));
        assert_eq!(store.peers[0].source, DiscoverySource::Direct);
        assert_eq!(
            store.peers[0].learned_from.as_deref(),
            Some("aira:identity:b")
        );
    }
}
