//! AddressBook promotion from Presence (QUEUE #240 / Phase N).
//!
//! Valid Presence + local trust policy → `peers/address_book.json` only.
//! Never auto-upserts TrustStore (`DISCOVERED ≠ TRUSTED`). Relay product
//! integration remains `#241`.

use std::path::Path;

use aira_object::{AiraRef, TrustStore, LOCAL_TEST_KEY_REF};

use crate::address_book::{AddressBook, PeerEndpoint};
use crate::error::PeerError;
use crate::presence::NodePresenceRecord;
use crate::prime_port::validate_aira_bind;

/// True when identity is in TrustStore entries and not revoked.
pub fn trust_policy_allows(store: &TrustStore, identity_id: &str) -> bool {
    let id = identity_id.trim();
    if id.is_empty() || id == LOCAL_TEST_KEY_REF {
        return false;
    }
    if store.is_revoked(id) {
        return false;
    }
    store.entries.iter().any(|e| e.identity_id == id)
}

/// Pick dial target from Presence: prefer `tcp-peer` direct, else first relay.
///
/// Returns `(addr, via)` where `via` is set for relay-only advertisements.
pub fn dial_target_from_presence(
    presence: &NodePresenceRecord,
) -> Result<(String, Option<String>), PeerError> {
    for ep in &presence.direct_endpoints {
        if ep.transport == "tcp-peer" {
            let addr = format!("{}:{}", ep.host.trim(), ep.port);
            validate_aira_bind(&addr)?;
            return Ok((addr, None));
        }
    }
    if let Some(rel) = presence.relay_endpoints.first() {
        validate_aira_bind(&rel.relay_endpoint)?;
        AiraRef::parse(&rel.relay_identity_ref).map_err(|e| PeerError::Protocol(e.to_string()))?;
        return Ok((
            rel.relay_endpoint.clone(),
            Some(rel.relay_identity_ref.clone()),
        ));
    }
    Err(PeerError::AddressBook(
        "presence has no tcp-peer direct or relay endpoint to promote".into(),
    ))
}

/// Promote a verified, trusted Presence into the authoritative AddressBook.
///
/// Fail-closed: invalid signature, untrusted/revoked identity, or empty endpoints.
/// Does **not** call `TrustStore::upsert`.
pub fn promote_presence_to_address_book(
    root: impl AsRef<Path>,
    presence: &NodePresenceRecord,
) -> Result<PeerEndpoint, PeerError> {
    presence.verify_canonical_signature()?;
    let root = root.as_ref();
    let trust = TrustStore::load(root).map_err(|e| PeerError::Crypto(e.to_string()))?;
    if !trust_policy_allows(&trust, &presence.identity_ref) {
        return Err(PeerError::Untrusted(presence.identity_ref.clone()));
    }
    let (addr, via) = dial_target_from_presence(presence)?;
    let mut book = AddressBook::load(root)?;
    book.upsert_via(&presence.identity_ref, &addr, via)?;
    book.save(root)?;
    let ep = book
        .peers
        .iter()
        .find(|p| p.identity_id == presence.identity_ref)
        .cloned()
        .ok_or_else(|| PeerError::AddressBook("promote failed to persist peer".into()))?;
    Ok(ep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, Keyring};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::presence::{
        empty_capabilities_hash, PresenceDirectEndpoint, PresenceDraft, PresenceReachability,
        PresenceRelayEndpoint,
    };

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

    fn signed_presence(
        peer_root: &Path,
        id: &AiraRef,
        pk: &str,
        relays: Vec<PresenceRelayEndpoint>,
    ) -> NodePresenceRecord {
        NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pk.into(),
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: "127.0.0.1".into(),
                port: 49157,
                reachability_state: PresenceReachability::Unknown,
                observed_at: "2026-09-05T12:00:00Z".into(),
            }],
            relay_endpoints: relays,
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(peer_root)
        .unwrap()
    }

    #[test]
    fn promote_trusted_presence_into_book() {
        let local = tempdir().unwrap();
        let peer = tempdir().unwrap();
        let (pid, ppk) = write_node(peer.path(), "promo-peer", [91u8; 32]);
        let _ = write_node(local.path(), "promo-local", [92u8; 32]);
        let mut trust = TrustStore::load(local.path()).unwrap();
        trust.upsert(pid.as_str(), &ppk).unwrap();
        trust.save(local.path()).unwrap();

        let presence = signed_presence(peer.path(), &pid, &ppk, vec![]);
        let ep = promote_presence_to_address_book(local.path(), &presence).unwrap();
        assert_eq!(ep.identity_id, pid.as_str());
        assert_eq!(ep.addr, "127.0.0.1:49157");
        let book = AddressBook::load(local.path()).unwrap();
        assert_eq!(
            book.resolve(pid.as_str()).unwrap().to_string(),
            "127.0.0.1:49157"
        );
    }

    #[test]
    fn rejects_untrusted_and_does_not_auto_trust() {
        let local = tempdir().unwrap();
        let peer = tempdir().unwrap();
        let (pid, ppk) = write_node(peer.path(), "stranger", [93u8; 32]);
        let _ = write_node(local.path(), "host", [94u8; 32]);
        let before = TrustStore::load(local.path()).unwrap();
        let presence = signed_presence(peer.path(), &pid, &ppk, vec![]);
        let err = promote_presence_to_address_book(local.path(), &presence).unwrap_err();
        assert!(matches!(err, PeerError::Untrusted(_)));
        assert!(AddressBook::load(local.path()).unwrap().peers.is_empty());
        let after = TrustStore::load(local.path()).unwrap();
        assert!(!after.entries.iter().any(|e| e.identity_id == pid.as_str()));
        assert_eq!(before.entries.len(), after.entries.len());
    }

    #[test]
    fn rejects_revoked_even_if_still_in_entries_removed() {
        let local = tempdir().unwrap();
        let peer = tempdir().unwrap();
        let (pid, ppk) = write_node(peer.path(), "revoked-peer", [95u8; 32]);
        let _ = write_node(local.path(), "host2", [96u8; 32]);
        let mut trust = TrustStore::load(local.path()).unwrap();
        trust.upsert(pid.as_str(), &ppk).unwrap();
        trust.revoke(pid.as_str(), Some("test")).unwrap();
        trust.save(local.path()).unwrap();
        let presence = signed_presence(peer.path(), &pid, &ppk, vec![]);
        assert!(promote_presence_to_address_book(local.path(), &presence).is_err());
        assert!(AddressBook::load(local.path()).unwrap().peers.is_empty());
    }

    #[test]
    fn relay_only_presence_sets_via() {
        let local = tempdir().unwrap();
        let peer = tempdir().unwrap();
        let (pid, ppk) = write_node(peer.path(), "relayed", [97u8; 32]);
        let _ = write_node(local.path(), "host3", [98u8; 32]);
        let mut trust = TrustStore::load(local.path()).unwrap();
        trust.upsert(pid.as_str(), &ppk).unwrap();
        trust.save(local.path()).unwrap();

        // Presence with only relay: craft by clearing directs after draft — easier to build custom
        let mut presence = NodePresenceRecord::draft(PresenceDraft {
            identity_ref: pid.as_str().into(),
            identity_public_key: ppk.clone(),
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![],
            relay_endpoints: vec![PresenceRelayEndpoint {
                relay_identity_ref: "aira:identity:hub".into(),
                relay_endpoint: "127.0.0.1:49171".into(),
                reservation_id: "r1".into(),
                expires_at: "2026-09-12T12:00:00Z".into(),
            }],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(peer.path())
        .unwrap();
        let _ = &mut presence;
        let ep = promote_presence_to_address_book(local.path(), &presence).unwrap();
        assert_eq!(ep.via.as_deref(), Some("aira:identity:hub"));
        assert_eq!(ep.addr, "127.0.0.1:49171");
    }
}
