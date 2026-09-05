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
    /// Optional trusted relay identity used as courier (Analyze-44).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
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
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PeerError::AddressBook(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Insert or replace by identity_id (clears `via`). Fail-closed on non-`P_AIRA` ports.
    pub fn upsert(
        &mut self,
        identity_id: impl Into<String>,
        addr: impl Into<String>,
    ) -> Result<(), PeerError> {
        self.upsert_via(identity_id, addr, None)
    }

    /// Insert or replace by identity_id, optionally setting courier relay.
    pub fn upsert_via(
        &mut self,
        identity_id: impl Into<String>,
        addr: impl Into<String>,
        via: Option<String>,
    ) -> Result<(), PeerError> {
        let identity_id = identity_id.into();
        let addr = addr.into();
        Self::validate_addr_string(&addr)?;
        if let Some(p) = self.peers.iter_mut().find(|p| p.identity_id == identity_id) {
            p.addr = addr;
            p.via = via;
        } else {
            self.peers.push(PeerEndpoint {
                identity_id,
                addr,
                via,
            });
        }
        self.peers.sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Update/insert dial addr while preserving an existing `via` (Analyze-57 DHT promote).
    ///
    /// New peers get `via = None`. Unlike [`Self::upsert`], this never clears courier routing.
    pub fn upsert_addr_preserve_via(
        &mut self,
        identity_id: impl Into<String>,
        addr: impl Into<String>,
    ) -> Result<(), PeerError> {
        let identity_id = identity_id.into();
        let addr = addr.into();
        Self::validate_addr_string(&addr)?;
        if let Some(p) = self.peers.iter_mut().find(|p| p.identity_id == identity_id) {
            p.addr = addr;
        } else {
            self.peers.push(PeerEndpoint {
                identity_id,
                addr,
                via: None,
            });
        }
        self.peers.sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Lookup socket address for identity.
    pub fn resolve(&self, identity_id: &str) -> Result<SocketAddr, PeerError> {
        let ep = self
            .peers
            .iter()
            .find(|p| p.identity_id == identity_id)
            .ok_or_else(|| PeerError::AddressBook(format!("unknown peer {identity_id}")))?;
        let addr: SocketAddr = ep
            .addr
            .parse()
            .map_err(|e| PeerError::AddressBook(format!("bad addr {}: {e}", ep.addr)))?;
        crate::prime_port::validate_aira_port(addr.port())?;
        Ok(addr)
    }

    fn validate_addr_string(addr: &str) -> Result<(), PeerError> {
        crate::prime_port::validate_aira_bind(addr).map(|_| ())
    }

    /// Courier relay identity for `identity_id`, if configured.
    pub fn via_of(&self, identity_id: &str) -> Option<&str> {
        self.peers
            .iter()
            .find(|p| p.identity_id == identity_id)
            .and_then(|p| p.via.as_deref())
    }

    /// Map view for tests / diagnostics.
    pub fn as_map(&self) -> BTreeMap<String, String> {
        self.peers
            .iter()
            .map(|p| (p.identity_id.clone(), p.addr.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_addr_preserve_via_keeps_courier() {
        let mut book = AddressBook::default();
        book.upsert_via(
            "aira:identity:a",
            "127.0.0.1:49157",
            Some("aira:identity:relay".into()),
        )
        .unwrap();
        book.upsert_addr_preserve_via("aira:identity:a", "127.0.0.1:49171")
            .unwrap();
        let ep = book
            .peers
            .iter()
            .find(|p| p.identity_id == "aira:identity:a")
            .unwrap();
        assert_eq!(ep.addr, "127.0.0.1:49171");
        assert_eq!(ep.via.as_deref(), Some("aira:identity:relay"));
    }

    #[test]
    fn upsert_clears_via_unlike_preserve() {
        let mut book = AddressBook::default();
        book.upsert_via(
            "aira:identity:a",
            "127.0.0.1:49157",
            Some("aira:identity:relay".into()),
        )
        .unwrap();
        book.upsert("aira:identity:a", "127.0.0.1:49171").unwrap();
        assert!(book.peers[0].via.is_none());
    }

    #[test]
    fn upsert_addr_preserve_via_inserts_new() {
        let mut book = AddressBook::default();
        book.upsert_addr_preserve_via("aira:identity:b", "127.0.0.1:49157")
            .unwrap();
        assert_eq!(book.peers.len(), 1);
        assert!(book.peers[0].via.is_none());
    }

    #[test]
    fn upsert_rejects_non_prime_port() {
        let mut book = AddressBook::default();
        let err = book
            .upsert("aira:identity:a", "127.0.0.1:9797")
            .unwrap_err();
        assert!(err.to_string().contains("Prime Private Port Invariant"));
    }
}
