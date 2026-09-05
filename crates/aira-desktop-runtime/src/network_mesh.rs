//! Phase N `#244`: Network mesh snapshot for Desktop (orchestrates peer APIs only).

use std::path::Path;

use aira_object::Keyring;
use aira_peer::{
    preferred_port, AddressBook, ReachabilityLocalState, ReachabilityStatus,
    RendezvousLocalState, StunReflexiveRecord, TransportClass,
};
use anyhow::Result;

/// Operator-facing top-level mesh banner (TZ §35).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshTopLevel {
    Direct,
    Relayed,
    OutboundOnly,
    Offline,
}

impl MeshTopLevel {
    /// Stable English label for UI / tests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "DIRECT",
            Self::Relayed => "RELAYED",
            Self::OutboundOnly => "OUTBOUND ONLY",
            Self::Offline => "OFFLINE",
        }
    }

    /// Map local reachability status to the coarse banner.
    pub fn from_reachability(status: ReachabilityStatus) -> Self {
        match status {
            ReachabilityStatus::DirectReachable => Self::Direct,
            ReachabilityStatus::RelayOnly => Self::Relayed,
            ReachabilityStatus::OutboundOnly => Self::OutboundOnly,
            ReachabilityStatus::Unknown
            | ReachabilityStatus::LocalOnly
            | ReachabilityStatus::Offline => Self::Offline,
        }
    }
}

/// Read-only Network tab fields (Identity, port, reachability, rendezvous, peers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkMeshSnapshot {
    pub identity: String,
    pub preferred_port: Option<u16>,
    pub local_bind: Option<String>,
    pub external_observed: Option<String>,
    pub reachability_status: String,
    pub top_level: String,
    pub direct_reachability: String,
    pub relay_reachability: String,
    pub rendezvous_provider: String,
    pub rendezvous_connected: bool,
    pub rendezvous_sequence: u64,
    pub peer_count: usize,
}

impl NetworkMeshSnapshot {
    /// Empty / unavailable snapshot (no identity yet).
    pub fn unavailable() -> Self {
        Self {
            identity: String::new(),
            preferred_port: None,
            local_bind: None,
            external_observed: None,
            reachability_status: "UNKNOWN".into(),
            top_level: MeshTopLevel::Offline.as_str().into(),
            direct_reachability: "no".into(),
            relay_reachability: "no".into(),
            rendezvous_provider: String::new(),
            rendezvous_connected: false,
            rendezvous_sequence: 0,
            peer_count: 0,
        }
    }
}

fn status_label(status: ReachabilityStatus) -> String {
    match status {
        ReachabilityStatus::Unknown => "UNKNOWN".into(),
        ReachabilityStatus::LocalOnly => "LOCAL_ONLY".into(),
        ReachabilityStatus::DirectReachable => "DIRECT_REACHABLE".into(),
        ReachabilityStatus::RelayOnly => "RELAY_ONLY".into(),
        ReachabilityStatus::OutboundOnly => "OUTBOUND_ONLY".into(),
        ReachabilityStatus::Offline => "OFFLINE".into(),
    }
}

/// Load Network mesh fields from node root + optional configured peer listen.
pub fn load_network_mesh_snapshot(
    root: impl AsRef<Path>,
    peer_listen: Option<&str>,
) -> Result<NetworkMeshSnapshot> {
    let root = root.as_ref();
    let identity = match Keyring::load_node_identity(root) {
        Ok((id, _)) => id.as_str().to_string(),
        Err(_) => {
            return Ok(NetworkMeshSnapshot::unavailable());
        }
    };

    let preferred = preferred_port(&identity, TransportClass::TcpPeer);
    let reach = ReachabilityLocalState::load(root)?;
    let top = MeshTopLevel::from_reachability(reach.status);

    let local_bind = peer_listen
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            reach
                .local_port
                .map(|p| format!("127.0.0.1:{p}"))
        });

    let external_observed = reach
        .observed_endpoint
        .clone()
        .or_else(|| StunReflexiveRecord::load(root).ok().map(|r| r.addr));

    let rv = RendezvousLocalState::load(root)?;
    let rendezvous_connected = !rv.provider.is_empty() && rv.local_sequence > 0;

    let book = AddressBook::load(root)?;
    let peer_count = book.peers.len();

    let direct_reachability = if reach.status == ReachabilityStatus::DirectReachable {
        "yes"
    } else {
        "no"
    };
    let relay_reachability = if reach.status == ReachabilityStatus::RelayOnly
        || !reach.relay_routes.is_empty()
    {
        "yes"
    } else {
        "no"
    };

    Ok(NetworkMeshSnapshot {
        identity,
        preferred_port: Some(preferred),
        local_bind,
        external_observed,
        reachability_status: status_label(reach.status),
        top_level: top.as_str().into(),
        direct_reachability: direct_reachability.into(),
        relay_reachability: relay_reachability.into(),
        rendezvous_provider: rv.provider,
        rendezvous_connected,
        rendezvous_sequence: rv.local_sequence,
        peer_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef};
    use aira_peer::{RelayRouteRecord};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    fn write_node(root: &Path, name: &str, seed: [u8; 32]) -> AiraRef {
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
        id_ref
    }

    #[test]
    fn snapshot_reads_port_reachability_peers() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let id = write_node(root, "mesh-a", [61u8; 32]);
        let mut reach = ReachabilityLocalState::default();
        reach
            .mark_local_bind(49157, "2026-09-05T12:00:00Z")
            .unwrap();
        reach.save(root).unwrap();
        let mut book = AddressBook::default();
        book.upsert("aira:identity:peer-b", "127.0.0.1:49169")
            .unwrap();
        book.save(root).unwrap();

        let snap =
            load_network_mesh_snapshot(root, Some("127.0.0.1:49157")).unwrap();
        assert_eq!(snap.identity, id.as_str());
        assert_eq!(snap.preferred_port, Some(preferred_port(id.as_str(), TransportClass::TcpPeer)));
        assert_eq!(snap.local_bind.as_deref(), Some("127.0.0.1:49157"));
        assert_eq!(snap.reachability_status, "LOCAL_ONLY");
        assert_eq!(snap.top_level, "OFFLINE");
        assert_eq!(snap.direct_reachability, "no");
        assert_eq!(snap.peer_count, 1);
    }

    #[test]
    fn relay_only_maps_to_relayed_banner() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let _ = write_node(root, "mesh-b", [62u8; 32]);
        let mut reach = ReachabilityLocalState::default();
        reach
            .apply_direct_failed(
                "2026-09-05T12:00:00Z",
                vec![RelayRouteRecord {
                    relay_identity_ref: "aira:identity:relay".into(),
                    relay_endpoint: "127.0.0.1:49171".into(),
                    reservation_id: Some("r1".into()),
                }],
                true,
            )
            .unwrap();
        reach.save(root).unwrap();
        let snap = load_network_mesh_snapshot(root, None).unwrap();
        assert_eq!(snap.top_level, "RELAYED");
        assert_eq!(snap.relay_reachability, "yes");
        assert_eq!(snap.reachability_status, "RELAY_ONLY");
    }
}
