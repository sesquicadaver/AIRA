//! Phase S `#300`: opt-in peer dial with durable handshake evidence (Desktop path).
//!
//! Does **not** apply reachability DIRECT / invent CONNECTED from setup alone.
//! Public bind stays out of scope — callers supply an explicit dial address.
//! Phase T `#307`: evidence is last-check history; never invents `live_session_count`.
//! Phase T `#311`: candidate AddressBook upsert is rolled back if dial fails.
//! Phase U `#320`: rollback undoes **only** this dial's candidate (reload + selective
//! restore), so a parallel AddressBook upsert survives.

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{TrustStore, LOCAL_TEST_KEY_REF};
use aira_peer::{AddressBook, PeerEndpoint};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Schema id for persisted Desktop dial evidence.
pub const DIAL_SESSION_EVIDENCE_SCHEMA_ID: &str = "aira:schema:desktop:dial-session-evidence:0.1";

/// How long a confirmed dial remains shown as **last handshake history** (not a live session).
pub const DIAL_EVIDENCE_FRESH_SECS: u64 = 300;

/// Durable record of one confirmed authenticated dial (Noise XX completed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialSessionEvidence {
    pub schema: String,
    pub peer_id: String,
    pub local_id: String,
    /// Explicit dial endpoint used for TCP connect (AddressBook after upsert).
    pub bound_endpoint: String,
    pub noise_handshake_hash_hex: String,
    /// Always `OUTBOUND` for Desktop opt-in dial.
    pub direction: String,
    pub confirmed_at: String,
}

impl DialSessionEvidence {
    /// Path under node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref()
            .join("peers")
            .join("dial_session_evidence.json")
    }

    /// Load evidence if present.
    pub fn load(root: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = Self::path(root);
        if !path.is_file() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let ev: Self =
            serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        if ev.schema != DIAL_SESSION_EVIDENCE_SCHEMA_ID {
            bail!(
                "unsupported dial evidence schema {} (want {})",
                ev.schema,
                DIAL_SESSION_EVIDENCE_SCHEMA_ID
            );
        }
        Ok(Some(ev))
    }

    /// Persist evidence (creates `peers/` as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<()> {
        let path = Self::path(root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, format!("{json}\n"))
            .with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    /// One-line UI / mesh summary (never claims DIRECT reachability).
    pub fn summary_line(&self) -> String {
        let hash = if self.noise_handshake_hash_hex.len() >= 12 {
            &self.noise_handshake_hash_hex[..12]
        } else {
            self.noise_handshake_hash_hex.as_str()
        };
        format!(
            "{} @ {} · hash {hash}… · {}",
            self.peer_id, self.bound_endpoint, self.confirmed_at
        )
    }

    /// Whether evidence is fresh relative to `now` (UTC).
    pub fn is_fresh_at(&self, now: OffsetDateTime) -> bool {
        let Ok(confirmed) = OffsetDateTime::parse(self.confirmed_at.trim(), &Rfc3339) else {
            return false;
        };
        let age = (now - confirmed).whole_seconds();
        age >= 0 && (age as u64) <= DIAL_EVIDENCE_FRESH_SECS
    }
}

/// Successful opt-in dial outcome (session closed after evidence capture).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialOutcome {
    pub evidence: DialSessionEvidence,
}

/// Opt-in dial: require trust + **explicit** address, trial-upsert AddressBook, dial,
/// persist evidence.
///
/// Phase T `#311` / Phase U `#320`: the AddressBook write is a **candidate** for this
/// attempt. On dial failure only that candidate is undone (reload book, restore or
/// remove this peer_id when still equal to the candidate). Parallel upserts of other
/// peers are preserved. On success the candidate remains dial authority.
/// Does not mutate reachability DIRECT state. Fail-closed on empty identity/addr or
/// untrusted peer.
pub fn run_opt_in_peer_dial(
    root: impl AsRef<Path>,
    peer_identity_id: &str,
    explicit_addr: &str,
) -> Result<DialOutcome> {
    let root = root.as_ref();
    let peer_id = peer_identity_id.trim();
    let addr = explicit_addr.trim();
    if peer_id.is_empty() {
        bail!("peer identity required for opt-in dial");
    }
    if peer_id == LOCAL_TEST_KEY_REF {
        bail!("cannot dial local test key ref");
    }
    if addr.is_empty() {
        bail!("explicit dial address required (setup/listen alone is not a dial target)");
    }

    let trust = TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    if trust.is_revoked(peer_id) {
        bail!("peer {peer_id} is revoked");
    }
    if !trust.entries.iter().any(|e| e.identity_id == peer_id) {
        bail!("peer {peer_id} is not trusted — import invite / admit trust first");
    }

    // Prior endpoint for this peer only (`#320` selective rollback).
    let prior_book = AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    let prior_endpoint = prior_book.endpoint_of(peer_id).cloned();
    let mut book = prior_book;
    book.upsert(peer_id, addr)
        .map_err(|e| anyhow::anyhow!("address book upsert: {e}"))?;
    book.save(root)
        .map_err(|e| anyhow::anyhow!("address book save: {e}"))?;

    let root_buf = root.to_path_buf();
    let peer_owned = peer_id.to_string();
    let session = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("tokio runtime for peer dial")?
        .block_on(async move { aira_peer::dial(&root_buf, &peer_owned).await })
    {
        Ok(s) => s,
        Err(e) => {
            if let Err(rb) = rollback_own_dial_candidate(root, peer_id, addr, prior_endpoint) {
                return Err(anyhow::anyhow!(
                    "dial failed: {e}; address book candidate rollback failed: {rb}"
                ));
            }
            return Err(anyhow::anyhow!(
                "dial failed: {e} (address book candidate rolled back)"
            ));
        }
    };

    let confirmed_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let evidence = DialSessionEvidence {
        schema: DIAL_SESSION_EVIDENCE_SCHEMA_ID.into(),
        peer_id: session.peer_id.as_str().to_string(),
        local_id: session.local_id.as_str().to_string(),
        bound_endpoint: session.bound_endpoint().to_string(),
        noise_handshake_hash_hex: session.noise_handshake_hash_hex(),
        direction: "OUTBOUND".into(),
        confirmed_at,
    };
    // Drop session (TCP close) — evidence is the durable proof for Desktop projection.
    drop(session);
    evidence.save(root)?;
    Ok(DialOutcome { evidence })
}

/// Undo only this dial's candidate on the live book (`#320` / RFC-0205).
///
/// Reloads from disk. If `peer_id` still holds `candidate_addr`, restores `prior`
/// or removes the entry. Otherwise leaves the book unchanged (another writer won).
pub(crate) fn rollback_own_dial_candidate(
    root: &Path,
    peer_id: &str,
    candidate_addr: &str,
    prior: Option<PeerEndpoint>,
) -> Result<()> {
    let mut book = AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    let still_candidate = book
        .endpoint_of(peer_id)
        .is_some_and(|ep| ep.addr == candidate_addr);
    if !still_candidate {
        return Ok(());
    }
    match prior {
        Some(ep) => book
            .upsert_via(ep.identity_id, ep.addr, ep.via)
            .map_err(|e| anyhow::anyhow!("{e}"))?,
        None => {
            book.remove(peer_id);
        }
    }
    book.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

/// Load fresh dial evidence for mesh projection (`None` if missing or stale).
pub fn load_fresh_dial_evidence(root: impl AsRef<Path>) -> Result<Option<DialSessionEvidence>> {
    let Some(ev) = DialSessionEvidence::load(root)? else {
        return Ok(None);
    };
    if ev.is_fresh_at(OffsetDateTime::now_utc()) {
        Ok(Some(ev))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{sign_with_key, AiraRef};
    use aira_peer::{accept, admit_peer_trust, listen_available_loopback};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

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
            "created_at": "2026-09-09T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": sig
        });
        fs::write(
            paths.identity_json(),
            serde_json::to_string_pretty(&desc).unwrap(),
        )
        .unwrap();
        aira_object::ensure_trust_defaults(root).unwrap();
        (id_ref, pub_hex)
    }

    #[test]
    fn empty_addr_is_fail_closed() {
        let dir = tempdir().unwrap();
        let err = run_opt_in_peer_dial(dir.path(), "aira:identity:x", "  ").unwrap_err();
        assert!(err.to_string().contains("explicit dial address"));
    }

    #[test]
    fn untrusted_peer_is_fail_closed() {
        let dir = tempdir().unwrap();
        let _ = write_node(dir.path(), "local", [11u8; 32]);
        let err = run_opt_in_peer_dial(dir.path(), "aira:identity:foreign", "127.0.0.1:49157")
            .unwrap_err();
        assert!(err.to_string().contains("not trusted"));
    }

    #[tokio::test]
    async fn opt_in_dial_persists_evidence_without_direct_status() {
        let target = tempdir().unwrap();
        let probe = tempdir().unwrap();
        let (tid, tpk) = write_node(target.path(), "dial-tgt", [91u8; 32]);
        let (pid, ppk) = write_node(probe.path(), "dial-prb", [92u8; 32]);
        admit_peer_trust(target.path(), pid.as_str(), &ppk).unwrap();
        admit_peer_trust(probe.path(), tid.as_str(), &tpk).unwrap();

        let (listener, addr) = listen_available_loopback().await.unwrap();
        let endpoint = format!("127.0.0.1:{}", addr.port());
        let root_t = target.path().to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_t).await });

        let outcome = tokio::task::spawn_blocking({
            let probe = probe.path().to_path_buf();
            let tid = tid.as_str().to_string();
            let endpoint = endpoint.clone();
            move || run_opt_in_peer_dial(&probe, &tid, &endpoint)
        })
        .await
        .unwrap()
        .unwrap();
        let _ = accept_task.await.unwrap().unwrap();

        assert_eq!(outcome.evidence.peer_id, tid.as_str());
        assert_eq!(outcome.evidence.direction, "OUTBOUND");
        assert!(!outcome.evidence.noise_handshake_hash_hex.is_empty());
        assert!(DialSessionEvidence::path(probe.path()).is_file());

        let loaded = DialSessionEvidence::load(probe.path()).unwrap().unwrap();
        assert_eq!(
            loaded.noise_handshake_hash_hex,
            outcome.evidence.noise_handshake_hash_hex
        );
        assert!(loaded.is_fresh_at(OffsetDateTime::now_utc()));

        // Must not invent DIRECT reachability from dial alone.
        let reach = aira_peer::ReachabilityLocalState::load(probe.path()).unwrap();
        assert_ne!(reach.status, aira_peer::ReachabilityStatus::DirectReachable);

        let snap = crate::load_network_mesh_snapshot(probe.path(), None).unwrap();
        // `#307`: closed dial is history, not a live session.
        assert_eq!(snap.live_session_count, None);
        assert!(snap
            .last_confirmed_handshake
            .as_deref()
            .unwrap()
            .contains(tid.as_str()));
        assert_ne!(snap.top_level, "DIRECT");
        let sys = crate::load_system_snapshot(probe.path(), None).unwrap();
        assert_eq!(
            sys.live_sessions_quality,
            crate::DataQuality::Unknown,
            "handshake history must not mark live sessions Current"
        );
    }

    #[test]
    fn fresh_dial_evidence_is_history_not_live_count() {
        let dir = tempdir().unwrap();
        let _ = write_node(dir.path(), "hist", [33u8; 32]);
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
        let ev = DialSessionEvidence {
            schema: DIAL_SESSION_EVIDENCE_SCHEMA_ID.into(),
            peer_id: "aira:identity:peer".into(),
            local_id: "aira:identity:hist".into(),
            bound_endpoint: "127.0.0.1:49157".into(),
            noise_handshake_hash_hex: "abcd".repeat(8),
            direction: "OUTBOUND".into(),
            confirmed_at: now,
        };
        ev.save(dir.path()).unwrap();
        let snap = crate::load_network_mesh_snapshot(dir.path(), None).unwrap();
        assert_eq!(snap.live_session_count, None);
        assert!(snap.last_confirmed_handshake.is_some());
    }

    /// `#311` / `#320`: failed trial dial must not leave candidate B as AddressBook authority.
    #[test]
    fn failed_dial_restores_prior_address_book() {
        let dir = tempdir().unwrap();
        let (_lid, _) = write_node(dir.path(), "probe", [41u8; 32]);
        let peer = "aira:identity:peer311";
        admit_peer_trust(dir.path(), peer, &"aa".repeat(32)).unwrap();

        let known = "127.0.0.1:49157";
        let mut book = AddressBook::default();
        book.upsert(peer, known).unwrap();
        book.save(dir.path()).unwrap();

        // Unreachable candidate (no listener); still a valid P_AIRA port.
        let candidate = "127.0.0.1:49171";
        let err = run_opt_in_peer_dial(dir.path(), peer, candidate).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("dial failed"), "{msg}");
        assert!(
            msg.contains("address book candidate rolled back"),
            "rollback should be explicit in error: {msg}"
        );

        let restored = AddressBook::load(dir.path()).unwrap();
        assert_eq!(
            restored.as_map().get(peer).map(String::as_str),
            Some(known),
            "known address A must survive failed candidate B"
        );
        assert!(
            !DialSessionEvidence::path(dir.path()).is_file(),
            "failed dial must not write handshake evidence"
        );
    }

    /// `#311` / `#320`: first-time peer with no prior entry is removed after failed dial.
    #[test]
    fn failed_dial_removes_new_candidate_when_no_prior() {
        let dir = tempdir().unwrap();
        let _ = write_node(dir.path(), "probe", [42u8; 32]);
        let peer = "aira:identity:peer311new";
        admit_peer_trust(dir.path(), peer, &"bb".repeat(32)).unwrap();

        let err = run_opt_in_peer_dial(dir.path(), peer, "127.0.0.1:49171").unwrap_err();
        assert!(err
            .to_string()
            .contains("address book candidate rolled back"));

        let book = AddressBook::load(dir.path()).unwrap();
        assert!(
            !book.as_map().contains_key(peer),
            "failed first dial must not leave orphan AddressBook entry"
        );
    }

    /// `#320`: full-snapshot rollback must not erase a parallel peer upsert.
    #[test]
    fn selective_rollback_preserves_parallel_peer_upsert() {
        let dir = tempdir().unwrap();
        let peer = "aira:identity:peer320dial";
        let known = "127.0.0.1:49157";
        let candidate = "127.0.0.1:49171";
        let parallel_id = "aira:identity:peer320parallel";
        let parallel_addr = "127.0.0.1:49177";

        let prior = PeerEndpoint {
            identity_id: peer.into(),
            addr: known.into(),
            via: None,
        };
        let mut book = AddressBook::default();
        book.upsert(peer, known).unwrap();
        book.save(dir.path()).unwrap();

        // Dial candidate on disk (as after trial upsert).
        book.upsert(peer, candidate).unwrap();
        book.save(dir.path()).unwrap();

        // Parallel writer while dial would be in flight.
        book.upsert(parallel_id, parallel_addr).unwrap();
        book.save(dir.path()).unwrap();

        rollback_own_dial_candidate(dir.path(), peer, candidate, Some(prior)).unwrap();

        let book = AddressBook::load(dir.path()).unwrap();
        assert_eq!(
            book.as_map().get(peer).map(String::as_str),
            Some(known),
            "dial peer must restore known addr"
        );
        assert_eq!(
            book.as_map().get(parallel_id).map(String::as_str),
            Some(parallel_addr),
            "parallel upsert must survive selective rollback"
        );
    }

    /// `#320`: if another writer already replaced the candidate, do not clobber.
    #[test]
    fn selective_rollback_skips_when_candidate_already_replaced() {
        let dir = tempdir().unwrap();
        let peer = "aira:identity:peer320race";
        let known = "127.0.0.1:49157";
        let candidate = "127.0.0.1:49171";
        let other = "127.0.0.1:49177";
        let prior = PeerEndpoint {
            identity_id: peer.into(),
            addr: known.into(),
            via: None,
        };
        let mut book = AddressBook::default();
        book.upsert(peer, other).unwrap();
        book.save(dir.path()).unwrap();

        rollback_own_dial_candidate(dir.path(), peer, candidate, Some(prior)).unwrap();
        let book = AddressBook::load(dir.path()).unwrap();
        assert_eq!(
            book.as_map().get(peer).map(String::as_str),
            Some(other),
            "must not overwrite a concurrent non-candidate addr"
        );
    }
}
