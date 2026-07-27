//! Static peer address book (no DHT / registry).

use std::collections::BTreeMap;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::PeerError;

/// One dialable peer endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEndpoint {
    pub identity_id: String,
    pub addr: String,
}

/// Local static address book under `peers/address_book.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressBook {
    #[serde(default)]
    pub peers: Vec<PeerEndpoint>,
}

impl AddressBook {
    /// Path to address book for a node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("address_book.json")
    }

    /// Load or return empty book.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Persist address book (creates `peers/` as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Insert or replace by identity_id.
    pub fn upsert(&mut self, identity_id: impl Into<String>, addr: impl Into<String>) {
        let identity_id = identity_id.into();
        let addr = addr.into();
        if let Some(p) = self.peers.iter_mut().find(|p| p.identity_id == identity_id) {
            p.addr = addr;
        } else {
            self.peers.push(PeerEndpoint { identity_id, addr });
        }
        self.peers
            .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
    }

    /// Lookup socket address for identity.
    pub fn resolve(&self, identity_id: &str) -> Result<SocketAddr, PeerError> {
        let ep = self
            .peers
            .iter()
            .find(|p| p.identity_id == identity_id)
            .ok_or_else(|| PeerError::AddressBook(format!("unknown peer {identity_id}")))?;
        ep.addr
            .parse()
            .map_err(|e| PeerError::AddressBook(format!("bad addr {}: {e}", ep.addr)))
    }

    /// Map view for tests / diagnostics.
    pub fn as_map(&self) -> BTreeMap<String, String> {
        self.peers
            .iter()
            .map(|p| (p.identity_id.clone(), p.addr.clone()))
            .collect()
    }
}
