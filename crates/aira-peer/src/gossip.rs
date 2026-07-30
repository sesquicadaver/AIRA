//! Trust-delta gossip fanout (Analyze-43 / Analyze-53).
//!
//! Forwards the **original** signed envelope to address-book peers. Each local
//! node relays a given `message_id` at most once (durable seen log).
//! Analyze-53: never forward when `subject_id ≠ issuer` (self-sovereign only).

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use aira_protocol::ProtocolEnvelope;

use crate::address_book::AddressBook;
use crate::discovery::{DiscoverySource, PeerDiscoveryStore};
use crate::error::PeerError;
use crate::session::dial;
use crate::trust_delta::{parse_trust_delta, TRUST_DELTA_MESSAGE_TYPE};

/// Max retained message ids in the seen log.
pub const GOSSIP_SEEN_CAP: usize = 512;

/// Durable dedupe log under `peers/gossip_seen.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GossipSeenLog {
    #[serde(default)]
    pub message_ids: VecDeque<String>,
}

impl GossipSeenLog {
    /// Path to gossip_seen.json.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("gossip_seen.json")
    }

    /// Load or empty.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Persist.
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::AddressBook(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PeerError::AddressBook(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::AddressBook(e.to_string()))
    }

    /// Returns `true` if this id was **already** seen (caller should not relay again).
    ///
    /// If new, inserts and persists (capped).
    pub fn check_and_mark(&mut self, message_id: &str) -> bool {
        if self.message_ids.iter().any(|id| id == message_id) {
            return true;
        }
        self.message_ids.push_back(message_id.to_string());
        while self.message_ids.len() > GOSSIP_SEEN_CAP {
            self.message_ids.pop_front();
        }
        false
    }
}

/// Outcome of one gossip forward attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipForwardResult {
    pub peer_id: String,
    pub ok: bool,
    pub error: Option<String>,
    pub skipped: bool,
}

/// Mark message seen; return whether it was a duplicate.
pub fn gossip_mark_seen(root: impl AsRef<Path>, message_id: &str) -> Result<bool, PeerError> {
    let root = root.as_ref();
    let mut log = GossipSeenLog::load(root)?;
    let dup = log.check_and_mark(message_id);
    log.save(root)?;
    Ok(dup)
}

/// Forward a trust-delta envelope to all address-book peers except `exclude_peer_id`.
///
/// Best-effort. Does not re-sign. Skips when `message_id` already seen (after mark),
/// or when the delta is not self-sovereign (`subject_id ≠ issuer`, Analyze-53).
/// Records discovery sightings for the envelope issuer as `source=gossip`.
pub async fn gossip_forward_trust_delta(
    root: impl AsRef<Path>,
    env: &ProtocolEnvelope,
    exclude_peer_id: &str,
) -> Result<Vec<GossipForwardResult>, PeerError> {
    let root = root.as_ref();
    if env.message_type != TRUST_DELTA_MESSAGE_TYPE {
        return Err(PeerError::Protocol(format!(
            "gossip expects {TRUST_DELTA_MESSAGE_TYPE}, got {}",
            env.message_type
        )));
    }
    // Fail-closed parse; refuse to fan out garbage or third-party CRL (A-52/A-53).
    let delta = parse_trust_delta(env)?;
    let self_sovereign = delta.subject_id.trim() == env.issuer_identity.as_str();

    let msg_id = env.message_id.as_str();
    if gossip_mark_seen(root, msg_id)? {
        return Ok(vec![GossipForwardResult {
            peer_id: "*".into(),
            ok: true,
            error: None,
            skipped: true,
        }]);
    }
    if !self_sovereign {
        return Ok(vec![GossipForwardResult {
            peer_id: "*".into(),
            ok: true,
            error: Some("non-self-sovereign trust-delta".into()),
            skipped: true,
        }]);
    }

    // Record originator as gossip-learned (addr unknown unless in book).
    let book = AddressBook::load(root)?;
    let issuer = env.issuer_identity.as_str();
    let issuer_addr = book
        .peers
        .iter()
        .find(|p| p.identity_id == issuer)
        .map(|p| p.addr.clone());
    let _ = PeerDiscoveryStore::record_and_save(
        root,
        issuer,
        issuer_addr,
        Some(exclude_peer_id.to_string()),
        DiscoverySource::Gossip,
    );

    let mut out = Vec::new();
    for peer in &book.peers {
        if peer.identity_id == exclude_peer_id || peer.identity_id == issuer {
            continue;
        }
        // Do not send back to ourselves if somehow listed.
        let peer_id = peer.identity_id.clone();
        match dial(root, &peer_id).await {
            Ok(mut session) => match session.send_relayed_trust_delta(env).await {
                Ok(()) => {
                    let _ = PeerDiscoveryStore::record_and_save(
                        root,
                        &peer_id,
                        Some(peer.addr.clone()),
                        Some(exclude_peer_id.to_string()),
                        DiscoverySource::Direct,
                    );
                    out.push(GossipForwardResult {
                        peer_id,
                        ok: true,
                        error: None,
                        skipped: false,
                    });
                }
                Err(e) => out.push(GossipForwardResult {
                    peer_id,
                    ok: false,
                    error: Some(e.to_string()),
                    skipped: false,
                }),
            },
            Err(e) => out.push(GossipForwardResult {
                peer_id,
                ok: false,
                error: Some(e.to_string()),
                skipped: false,
            }),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn gossip_seen_dedupes_and_caps() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let mut log = GossipSeenLog::default();
        assert!(!log.check_and_mark("m1"));
        assert!(log.check_and_mark("m1"));
        log.save(root).unwrap();
        let loaded = GossipSeenLog::load(root).unwrap();
        assert!(loaded.message_ids.iter().any(|id| id == "m1"));

        let mut big = GossipSeenLog::default();
        for i in 0..(GOSSIP_SEEN_CAP + 3) {
            big.check_and_mark(&format!("id-{i}"));
        }
        assert_eq!(big.message_ids.len(), GOSSIP_SEEN_CAP);
        assert!(!big.message_ids.iter().any(|id| id == "id-0"));
    }
}
