//! NAT/relay integration: inbound-blocked peers via courier hub (QUEUE #246).
//!
//! Models TZ §48: both endpoints cannot accept inbound; they discover a trusted
//! relay and complete an encrypted peer payload through `RelayHub` courier.
//! End-to-end Noise remains peer↔relay (and signed inner envelope peer↔peer);
//! hubs never verify inner payloads. Ab ovo direct path stays `#245`.

use std::path::Path;

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::reachability_state::RelayRouteRecord;
use crate::relay_integrate::{plan_dial_path, DialPathInput, DialPathStep};

/// Configure AddressBook so `peer_id` is only reachable via `relay_id` at `relay_addr`.
///
/// The peer's own `direct_addr` is a non-listening placeholder (inbound blocked).
pub fn configure_inbound_blocked_via_relay(
    root: impl AsRef<Path>,
    peer_id: &str,
    blocked_direct_addr: &str,
    relay_id: &str,
    relay_addr: &str,
) -> Result<(), PeerError> {
    let root = root.as_ref();
    crate::prime_port::validate_aira_bind(blocked_direct_addr)?;
    crate::prime_port::validate_aira_bind(relay_addr)?;
    let mut book = AddressBook::load(root)?;
    book.upsert(relay_id, relay_addr)?;
    book.upsert_via(peer_id, blocked_direct_addr, Some(relay_id.to_string()))?;
    book.save(root)?;
    Ok(())
}

/// Dial plan when direct/NAT are unavailable: only relay steps remain.
pub fn plan_inbound_blocked_relay_path(
    relays: Vec<RelayRouteRecord>,
) -> Result<Vec<DialPathStep>, PeerError> {
    plan_dial_path(DialPathInput {
        direct_addr: None,
        nat_observed_addr: None,
        relays,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::{init_node, NodePaths};
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, TrustStore};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::rendezvous::LocalFileRendezvousProvider;
    use crate::session::{accept, dial, listen_available_loopback};
    use crate::trust_delta::{
        apply_trust_delta, make_trust_delta_envelope, parse_trust_delta, TrustDelta,
    };
    use crate::{send_envelope_to_peer, serve_relay_peer, RelayHub};

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
        (id_ref, pub_hex)
    }

    fn mutual_trust(
        a_root: &Path,
        a_id: &str,
        a_pub: &str,
        b_root: &Path,
        b_id: &str,
        b_pub: &str,
    ) {
        let mut ta = TrustStore::load(a_root).unwrap();
        ta.upsert(b_id, b_pub).unwrap();
        ta.save(a_root).unwrap();
        let mut tb = TrustStore::load(b_root).unwrap();
        tb.upsert(a_id, a_pub).unwrap();
        tb.save(b_root).unwrap();
    }

    #[test]
    fn inbound_blocked_plan_is_relay_only() {
        let steps = plan_inbound_blocked_relay_path(vec![
            RelayRouteRecord {
                relay_identity_ref: "aira:identity:r1".into(),
                relay_endpoint: "127.0.0.1:49171".into(),
                reservation_id: Some("res-1".into()),
            },
            RelayRouteRecord {
                relay_identity_ref: "aira:identity:r2".into(),
                relay_endpoint: "127.0.0.1:49177".into(),
                reservation_id: Some("res-2".into()),
            },
        ])
        .unwrap();
        assert_eq!(steps.len(), 2);
        assert!(steps.iter().all(|s| s.kind() == "relay"));
        assert!(plan_inbound_blocked_relay_path(vec![]).is_err());
    }

    #[tokio::test]
    async fn both_inbound_blocked_relay_courier_noise_succeeds() {
        // Shared empty ledger (discovery substrate); no preconfigured A↔B book.
        let ledger = tempdir().unwrap();
        assert!(!LocalFileRendezvousProvider::path(ledger.path()).exists());
        let _provider = LocalFileRendezvousProvider::open(ledger.path()).unwrap();

        let dir_a = tempdir().unwrap();
        let dir_b = tempdir().unwrap();
        let dir_r = tempdir().unwrap();
        let root_a = dir_a.path();
        let root_b = dir_b.path();
        let root_r = dir_r.path();
        init_node(root_a).unwrap();
        init_node(root_b).unwrap();
        init_node(root_r).unwrap();
        let (id_a, pub_a) = write_node(root_a, "nat-a", [81u8; 32]);
        let (id_b, pub_b) = write_node(root_b, "nat-b", [82u8; 32]);
        let (id_r, pub_r) = write_node(root_r, "nat-r", [83u8; 32]);

        mutual_trust(root_a, id_a.as_str(), &pub_a, root_r, id_r.as_str(), &pub_r);
        mutual_trust(root_b, id_b.as_str(), &pub_b, root_r, id_r.as_str(), &pub_r);
        // Inner trust for courier payload (DISCOVERED≠TRUSTED; explicit admit).
        let mut ta = TrustStore::load(root_a).unwrap();
        ta.upsert(id_b.as_str(), &pub_b).unwrap();
        ta.save(root_a).unwrap();
        let mut tb = TrustStore::load(root_b).unwrap();
        tb.upsert(id_a.as_str(), &pub_a).unwrap();
        tb.save(root_b).unwrap();

        assert!(AddressBook::load(root_a).unwrap().peers.is_empty());
        assert!(AddressBook::load(root_b).unwrap().peers.is_empty());

        let hub = RelayHub::new();
        let (listener, addr_r) = listen_available_loopback().await.unwrap();
        let relay_addr = addr_r.to_string();

        // Inbound blocked: peer direct slots are non-listening prime placeholders.
        configure_inbound_blocked_via_relay(
            root_a,
            id_b.as_str(),
            "127.0.0.1:49169",
            id_r.as_str(),
            &relay_addr,
        )
        .unwrap();
        configure_inbound_blocked_via_relay(
            root_b,
            id_a.as_str(),
            "127.0.0.1:49157",
            id_r.as_str(),
            &relay_addr,
        )
        .unwrap();

        // Direct dial of the blocked peer must fail (nothing listening).
        let direct_err = dial(root_a, id_b.as_str()).await;
        assert!(direct_err.is_err(), "direct must fail when inbound blocked");

        let hub_accept = hub.clone();
        let root_r2 = root_r.to_path_buf();
        let accept_task = tokio::spawn(async move {
            for _ in 0..2 {
                let peer = accept(&listener, &root_r2).await.unwrap();
                let hub_c = hub_accept.clone();
                let root_peer = root_r2.clone();
                tokio::spawn(async move {
                    let _ = serve_relay_peer(hub_c, peer, &root_peer, None).await;
                });
            }
        });

        let root_b2 = root_b.to_path_buf();
        let id_r_s = id_r.as_str().to_string();
        let hold = tokio::spawn(async move {
            let mut peer = dial(&root_b2, &id_r_s).await.unwrap();
            let env = peer.recv_envelope_allow_relayed().await.unwrap();
            let delta = parse_trust_delta(&env).unwrap();
            apply_trust_delta(&root_b2, &env.issuer_identity, &delta).unwrap();
            delta
        });

        for _ in 0..50 {
            if hub.registered().iter().any(|id| id == id_b.as_str()) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert!(
            hub.registered().iter().any(|id| id == id_b.as_str()),
            "B not registered on relay: {:?}",
            hub.registered()
        );

        let delta = TrustDelta::revoke(id_a.as_str(), Some("nat-relay".into()));
        let env = make_trust_delta_envelope(root_a, &delta).unwrap();
        send_envelope_to_peer(root_a, id_b.as_str(), &env)
            .await
            .unwrap();

        let applied = hold.await.unwrap();
        assert_eq!(applied.subject_id, id_a.as_str());
        assert!(TrustStore::load(root_b).unwrap().is_revoked(id_a.as_str()));
        let _ = accept_task.await;
    }
}
