//! Ab ovo discovery → trust → dial orchestration (QUEUE #245 / Phase N).
//!
//! Zero-knowledge path: empty AddressBook on B; A publishes Presence to a shared
//! ledger; B queries, records DISCOVERED, admits trust, promotes AddressBook, dials.
//! NAT/relay Noise remains `#246`. Does not auto-trust from the ledger.

use std::path::Path;

use aira_object::TrustStore;

use crate::address_book::{AddressBook, PeerEndpoint};
use crate::discovery::{DiscoverySource, PeerDiscoveryStore};
use crate::error::PeerError;
use crate::presence::NodePresenceRecord;
use crate::presence_promote::promote_presence_to_address_book;
use crate::rendezvous::RendezvousProvider;

/// Record a validated Presence as DISCOVERED (discovery journal only).
///
/// Never writes TrustStore or AddressBook.
pub fn record_discovered_presence(
    root: impl AsRef<Path>,
    presence: &NodePresenceRecord,
) -> Result<(), PeerError> {
    presence.verify_canonical_signature()?;
    let addr = presence
        .direct_endpoints
        .iter()
        .find(|e| e.transport == "tcp-peer")
        .map(|e| format!("{}:{}", e.host.trim(), e.port));
    PeerDiscoveryStore::record_and_save(
        root,
        presence.identity_ref.as_str(),
        addr,
        Some("rendezvous".into()),
        DiscoverySource::Rendezvous,
    )
}

/// Explicit trust admission (operator / policy). Does not promote AddressBook.
pub fn admit_peer_trust(
    root: impl AsRef<Path>,
    identity_id: &str,
    public_key_hex: &str,
) -> Result<(), PeerError> {
    let root = root.as_ref();
    let mut trust = TrustStore::load(root).map_err(|e| PeerError::Crypto(e.to_string()))?;
    trust
        .upsert(identity_id, public_key_hex)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    trust
        .save(root)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    Ok(())
}

/// Query ledger for `identity_ref`, verify, mark DISCOVERED; if trusted, promote book.
///
/// Fail-closed if AddressBook already had the peer when `require_empty_book` is true.
pub fn discover_admit_promote(
    local_root: impl AsRef<Path>,
    provider: &dyn RendezvousProvider,
    identity_ref: &str,
    as_of: &str,
    require_empty_book: bool,
) -> Result<(NodePresenceRecord, PeerEndpoint), PeerError> {
    let local_root = local_root.as_ref();
    if require_empty_book {
        let book = AddressBook::load(local_root)?;
        if book.peers.iter().any(|p| p.identity_id == identity_ref) {
            return Err(PeerError::Protocol(
                "ab ovo: peer already in AddressBook before discovery".into(),
            ));
        }
    }

    let presence = provider
        .query_identity(identity_ref)?
        .or_else(|| {
            provider
                .query_active_peers(as_of)
                .ok()
                .and_then(|list| list.into_iter().find(|p| p.identity_ref == identity_ref))
        })
        .ok_or_else(|| PeerError::Protocol(format!("ab ovo: {identity_ref} not in ledger")))?;

    presence.verify_canonical_signature()?;
    record_discovered_presence(local_root, &presence)?;

    let trust = TrustStore::load(local_root).map_err(|e| PeerError::Crypto(e.to_string()))?;
    if !crate::presence_promote::trust_policy_allows(&trust, &presence.identity_ref) {
        return Err(PeerError::Untrusted(presence.identity_ref.clone()));
    }

    let ep = promote_presence_to_address_book(local_root, &presence)?;
    Ok((presence, ep))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, Keyring};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::presence::{
        empty_capabilities_hash, PresenceDirectEndpoint, PresenceDraft, PresenceReachability,
    };
    use crate::rendezvous::LocalFileRendezvousProvider;
    use crate::{accept, dial, listen_available_loopback};

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
        host: &str,
        port: u16,
    ) -> NodePresenceRecord {
        NodePresenceRecord::draft(PresenceDraft {
            identity_ref: id.as_str().into(),
            identity_public_key: pk.into(),
            sequence: 1,
            created_at: "2026-09-05T12:00:00Z".into(),
            expires_at: "2026-09-12T12:00:00Z".into(),
            direct_endpoints: vec![PresenceDirectEndpoint {
                transport: "tcp-peer".into(),
                host: host.into(),
                port,
                reachability_state: PresenceReachability::Unknown,
                observed_at: "2026-09-05T12:00:00Z".into(),
            }],
            relay_endpoints: vec![],
            capabilities_hash: empty_capabilities_hash(),
        })
        .unwrap()
        .sign_for_node_root(peer_root)
        .unwrap()
    }

    #[test]
    fn discovery_without_trust_fills_journal_not_book() {
        let ledger = tempdir().unwrap();
        let a = tempdir().unwrap();
        let b = tempdir().unwrap();
        let (ida, pka) = write_node(a.path(), "ab-a", [71u8; 32]);
        let _ = write_node(b.path(), "ab-b", [72u8; 32]);
        let mut provider = LocalFileRendezvousProvider::open(ledger.path()).unwrap();
        let presence = signed_presence(a.path(), &ida, &pka, "127.0.0.1", 49157);
        provider.publish_presence(presence).unwrap();

        assert!(AddressBook::load(b.path()).unwrap().peers.is_empty());
        let err = discover_admit_promote(
            b.path(),
            &provider,
            ida.as_str(),
            "2026-09-05T12:00:00Z",
            true,
        )
        .unwrap_err();
        assert!(matches!(err, PeerError::Untrusted(_)));
        assert!(AddressBook::load(b.path()).unwrap().peers.is_empty());
        let disc = PeerDiscoveryStore::load(b.path()).unwrap();
        assert!(disc
            .peers
            .iter()
            .any(|p| p.identity_id == ida.as_str() && p.source == DiscoverySource::Rendezvous));
        assert!(!TrustStore::load(b.path())
            .unwrap()
            .entries
            .iter()
            .any(|e| e.identity_id == ida.as_str()));
    }

    #[tokio::test]
    async fn ab_ovo_publish_discover_trust_dial_noise() {
        let ledger = tempdir().unwrap();
        let a = tempdir().unwrap();
        let b = tempdir().unwrap();
        let (ida, pka) = write_node(a.path(), "ovo-a", [73u8; 32]);
        let (idb, pkb) = write_node(b.path(), "ovo-b", [74u8; 32]);

        assert!(!LocalFileRendezvousProvider::path(ledger.path()).exists());
        assert!(AddressBook::load(b.path()).unwrap().peers.is_empty());

        let (listener, addr) = listen_available_loopback().await.unwrap();
        let port = addr.port();

        let mut provider = LocalFileRendezvousProvider::open(ledger.path()).unwrap();
        let presence = signed_presence(a.path(), &ida, &pka, "127.0.0.1", port);
        provider.publish_presence(presence).unwrap();

        let pre = discover_admit_promote(
            b.path(),
            &provider,
            ida.as_str(),
            "2026-09-05T13:00:00Z",
            true,
        );
        assert!(matches!(pre, Err(PeerError::Untrusted(_))));
        assert!(AddressBook::load(b.path()).unwrap().peers.is_empty());

        admit_peer_trust(b.path(), ida.as_str(), &pka).unwrap();
        admit_peer_trust(a.path(), idb.as_str(), &pkb).unwrap();

        let (_pres, ep) = discover_admit_promote(
            b.path(),
            &provider,
            ida.as_str(),
            "2026-09-05T13:00:00Z",
            false,
        )
        .unwrap();
        assert_eq!(ep.identity_id, ida.as_str());
        assert_eq!(ep.addr, format!("127.0.0.1:{port}"));

        let root_a = a.path().to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_a).await });
        let client = dial(b.path(), ida.as_str()).await.unwrap();
        let server = accept_task.await.unwrap().unwrap();
        assert_eq!(client.peer_id, ida);
        assert_eq!(server.peer_id, idb);
    }
}
