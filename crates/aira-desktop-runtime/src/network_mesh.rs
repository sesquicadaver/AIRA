//! Phase N `#244` mesh fields + Phase O `#256` honesty projection.

use std::path::Path;

use aira_object::Keyring;
use aira_peer::{
    preferred_port, AddressBook, ReachabilityLocalState, ReachabilityStatus, RendezvousLocalState,
    StunReflexiveRecord, TransportClass,
};
use anyhow::Result;

/// Operator-facing top-level mesh banner (TZ §35; honesty `#256`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshTopLevel {
    Direct,
    Relayed,
    OutboundOnly,
    LocalOnly,
    Unknown,
    Offline,
}

impl MeshTopLevel {
    /// Stable English label for UI / tests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "DIRECT",
            Self::Relayed => "RELAYED",
            Self::OutboundOnly => "OUTBOUND ONLY",
            Self::LocalOnly => "LOCAL ONLY",
            Self::Unknown => "UNKNOWN",
            Self::Offline => "OFFLINE",
        }
    }

    /// Map local reachability status to the coarse banner without collapsing unknowns.
    pub fn from_reachability(status: ReachabilityStatus) -> Self {
        match status {
            ReachabilityStatus::DirectReachable => Self::Direct,
            ReachabilityStatus::RelayOnly => Self::Relayed,
            ReachabilityStatus::OutboundOnly => Self::OutboundOnly,
            ReachabilityStatus::LocalOnly => Self::LocalOnly,
            ReachabilityStatus::Unknown => Self::Unknown,
            ReachabilityStatus::Offline => Self::Offline,
        }
    }
}

/// Freshness / availability of a projected field or section (`#256`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataQuality {
    /// Observation matches current authoritative reads for this load.
    Current,
    /// Last known value may be older than the section's freshness threshold.
    Stale,
    /// Value is intentionally unknown (not the same as zero / offline).
    Unknown,
    /// Source could not be read (e.g. no identity yet).
    Unavailable,
}

impl DataQuality {
    /// Stable label for tests / technical details.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
            Self::Unavailable => "unavailable",
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
    /// Local publish metadata present (provider + sequence) — not a live global session.
    pub rendezvous_connected: bool,
    pub rendezvous_sequence: u64,
    /// Entries in AddressBook (dial authority). Never treat as live sessions.
    pub address_book_count: usize,
    /// Authenticated live sessions known to this process.
    /// `None` = not observed from Desktop runtime (do not display as `0` or book size).
    pub live_session_count: Option<usize>,
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
            top_level: MeshTopLevel::Unknown.as_str().into(),
            direct_reachability: "no".into(),
            relay_reachability: "no".into(),
            rendezvous_provider: String::new(),
            rendezvous_connected: false,
            rendezvous_sequence: 0,
            address_book_count: 0,
            live_session_count: None,
        }
    }
}

/// Typed Desktop projection of authoritative runtime/store facts (`#256`).
///
/// Not a second source of truth: each load re-reads stores. GUI state must not invent values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemSnapshot {
    /// RFC3339 observation time for this projection load (UTC).
    pub observed_at: String,
    pub network: NetworkMeshSnapshot,
    pub network_quality: DataQuality,
    /// Quality of live-session observation (usually [`DataQuality::Unknown`] until a live feed exists).
    pub live_sessions_quality: DataQuality,
}

impl SystemSnapshot {
    /// Build projection around an already-loaded mesh snapshot.
    pub fn from_network(network: NetworkMeshSnapshot, observed_at: String) -> Self {
        let network_quality = if network.identity.is_empty() {
            DataQuality::Unavailable
        } else {
            DataQuality::Current
        };
        let live_sessions_quality = match network.live_session_count {
            Some(_) => DataQuality::Current,
            None if network.identity.is_empty() => DataQuality::Unavailable,
            None => DataQuality::Unknown,
        };
        Self {
            observed_at,
            network,
            network_quality,
            live_sessions_quality,
        }
    }

    /// Empty projection when identity / root is unavailable.
    pub fn unavailable() -> Self {
        Self::from_network(
            NetworkMeshSnapshot::unavailable(),
            chrono_like_now_fallback(),
        )
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

fn chrono_like_now_fallback() -> String {
    // Prefer std-only timestamp when chrono is unavailable in this crate.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
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
        .or_else(|| reach.local_port.map(|p| format!("127.0.0.1:{p}")));

    let external_observed = reach
        .observed_endpoint
        .clone()
        .or_else(|| StunReflexiveRecord::load(root).ok().map(|r| r.addr));

    let rv = RendezvousLocalState::load(root)?;
    let rendezvous_connected = !rv.provider.is_empty() && rv.local_sequence > 0;

    let book = AddressBook::load(root)?;
    let address_book_count = book.peers.len();
    // Desktop runtime has no in-process peer accept loop; live sessions are unknown here.
    let live_session_count = None;

    let direct_reachability = if reach.status == ReachabilityStatus::DirectReachable {
        "yes"
    } else {
        "no"
    };
    let relay_reachability =
        if reach.status == ReachabilityStatus::RelayOnly || !reach.relay_routes.is_empty() {
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
        address_book_count,
        live_session_count,
    })
}

/// Load [`SystemSnapshot`] projection (mesh + quality markers).
pub fn load_system_snapshot(
    root: impl AsRef<Path>,
    peer_listen: Option<&str>,
) -> Result<SystemSnapshot> {
    let network = load_network_mesh_snapshot(root, peer_listen)?;
    Ok(SystemSnapshot::from_network(
        network,
        chrono_like_now_fallback(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef};
    use aira_peer::RelayRouteRecord;
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

        let snap = load_network_mesh_snapshot(root, Some("127.0.0.1:49157")).unwrap();
        assert_eq!(snap.identity, id.as_str());
        assert_eq!(
            snap.preferred_port,
            Some(preferred_port(id.as_str(), TransportClass::TcpPeer))
        );
        assert_eq!(snap.local_bind.as_deref(), Some("127.0.0.1:49157"));
        assert_eq!(snap.reachability_status, "LOCAL_ONLY");
        assert_eq!(snap.top_level, "LOCAL ONLY");
        assert_ne!(snap.top_level, "OFFLINE");
        assert_eq!(snap.direct_reachability, "no");
        assert_eq!(snap.address_book_count, 1);
        assert_eq!(snap.live_session_count, None);
    }

    #[test]
    fn unknown_reachability_is_not_offline_banner() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let _ = write_node(root, "mesh-u", [63u8; 32]);
        // Default reachability state is UNKNOWN with no bind.
        let snap = load_network_mesh_snapshot(root, None).unwrap();
        assert_eq!(snap.reachability_status, "UNKNOWN");
        assert_eq!(snap.top_level, "UNKNOWN");
        assert_ne!(snap.top_level, "OFFLINE");
        assert_eq!(
            MeshTopLevel::from_reachability(ReachabilityStatus::Unknown),
            MeshTopLevel::Unknown
        );
        assert_eq!(
            MeshTopLevel::from_reachability(ReachabilityStatus::Offline),
            MeshTopLevel::Offline
        );
    }

    #[test]
    fn address_book_peers_are_not_live_sessions() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let _ = write_node(root, "mesh-book", [64u8; 32]);
        let mut book = AddressBook::default();
        book.upsert("aira:identity:peer-c", "127.0.0.1:49157")
            .unwrap();
        book.upsert("aira:identity:peer-d", "127.0.0.1:49169")
            .unwrap();
        book.save(root).unwrap();

        let sys = load_system_snapshot(root, None).unwrap();
        assert_eq!(sys.network.address_book_count, 2);
        assert_eq!(sys.network.live_session_count, None);
        assert_eq!(sys.live_sessions_quality, DataQuality::Unknown);
        assert_eq!(sys.network_quality, DataQuality::Current);
        assert!(!sys.observed_at.is_empty());
    }

    #[test]
    fn unavailable_identity_projects_unknown_not_offline() {
        let dir = tempdir().unwrap();
        let snap = load_network_mesh_snapshot(dir.path(), None).unwrap();
        assert!(snap.identity.is_empty());
        assert_eq!(snap.top_level, "UNKNOWN");
        assert_eq!(snap.live_session_count, None);
        let sys = SystemSnapshot::from_network(snap, "unix:0".into());
        assert_eq!(sys.network_quality, DataQuality::Unavailable);
        assert_eq!(sys.live_sessions_quality, DataQuality::Unavailable);
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
