//! Phase N `#244` mesh fields + Phase O `#256` honesty + Phase P `#267` freshness.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use aira_object::Keyring;
use aira_peer::{
    preferred_port, AddressBook, ReachabilityLocalState, ReachabilityStatus, RendezvousLocalState,
    StunReflexiveRecord, TransportClass,
};
use anyhow::Result;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Max age (seconds) of reachability status observation before network quality is [`DataQuality::Stale`].
pub const NETWORK_OBSERVATION_STALE_SECS: u64 = 300;

/// Max future skew (seconds) still treated as [`DataQuality::Current`] (`#279`).
/// Beyond this, future clocks are [`DataQuality::Unknown`] — not Current.
pub const NETWORK_OBSERVATION_MAX_SKEW_SECS: i64 = 300;

/// Where [`NetworkMeshSnapshot::local_bind`] came from (`#267`: config ≠ live listener).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocalBindProvenance {
    /// No bind string to display.
    #[default]
    None,
    /// Settings / CLI `peer_listen` (operator intent) — not proof a socket is listening.
    Configured,
    /// Port recorded in `reachability.json` — historical local_port, not accept-loop proof.
    ReachabilityRecord,
}

impl LocalBindProvenance {
    /// Stable label for UI / tests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Configured => "configured",
            Self::ReachabilityRecord => "reachability_record",
        }
    }
}

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

/// Freshness / availability of a projected field or section (`#256` / `#267`).
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
    /// Display bind string (configured listen or recorded reachability port).
    pub local_bind: Option<String>,
    /// Provenance of [`Self::local_bind`] — never implies a live accept loop.
    pub local_bind_provenance: LocalBindProvenance,
    /// True only when this process has proven an accept loop is listening.
    /// Desktop projection always leaves this `false` until a live listener feed exists.
    pub local_listener_proven: bool,
    pub external_observed: Option<String>,
    pub reachability_status: String,
    pub top_level: String,
    pub direct_reachability: String,
    pub relay_reachability: String,
    /// Measurement time from status-relevant reachability observation (RFC3339), if any.
    pub reachability_checked_at: Option<String>,
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
            local_bind_provenance: LocalBindProvenance::None,
            local_listener_proven: false,
            external_observed: None,
            reachability_status: "UNKNOWN".into(),
            top_level: MeshTopLevel::Unknown.as_str().into(),
            direct_reachability: "no".into(),
            relay_reachability: "no".into(),
            reachability_checked_at: None,
            rendezvous_provider: String::new(),
            rendezvous_connected: false,
            rendezvous_sequence: 0,
            address_book_count: 0,
            live_session_count: None,
        }
    }
}

/// Typed Desktop projection of authoritative runtime/store facts (`#256` / `#267`).
///
/// Not a second source of truth: each load re-reads stores. GUI state must not invent values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemSnapshot {
    /// Measurement time for network facts (status-relevant observation), or `unknown`.
    pub observed_at: String,
    /// When this projection was loaded (not the same as measurement time).
    pub loaded_at: String,
    pub network: NetworkMeshSnapshot,
    pub network_quality: DataQuality,
    /// Quality of live-session observation (usually [`DataQuality::Unknown`] until a live feed exists).
    pub live_sessions_quality: DataQuality,
}

impl SystemSnapshot {
    /// Build projection around an already-loaded mesh snapshot.
    ///
    /// `loaded_at` is the projection load clock (RFC3339 preferred; `unix:<secs>` accepted).
    /// [`Self::observed_at`] comes from mesh measurement time, not from `loaded_at`.
    pub fn from_network(network: NetworkMeshSnapshot, loaded_at: String) -> Self {
        let observed_at = network
            .reachability_checked_at
            .clone()
            .unwrap_or_else(|| "unknown".into());
        let network_quality = classify_network_quality(&network, &loaded_at);
        let live_sessions_quality = match network.live_session_count {
            Some(_) => DataQuality::Current,
            None if network.identity.is_empty() => DataQuality::Unavailable,
            None => DataQuality::Unknown,
        };
        Self {
            observed_at,
            loaded_at,
            network,
            network_quality,
            live_sessions_quality,
        }
    }

    /// Empty projection when identity / root is unavailable.
    pub fn unavailable() -> Self {
        Self::from_network(NetworkMeshSnapshot::unavailable(), projection_now_label())
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

fn projection_now_label() -> String {
    match OffsetDateTime::now_utc().format(&Rfc3339) {
        Ok(s) => s,
        Err(_) => {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("unix:{secs}")
        }
    }
}

fn parse_clock_label(label: &str) -> Option<OffsetDateTime> {
    let label = label.trim();
    if label.is_empty() || label == "unknown" {
        return None;
    }
    if let Some(secs) = label.strip_prefix("unix:") {
        let secs: i64 = secs.parse().ok()?;
        return OffsetDateTime::from_unix_timestamp(secs).ok();
    }
    OffsetDateTime::parse(label, &Rfc3339).ok()
}

/// Classify network section quality from measurement age vs projection load clock (`#267`).
pub fn classify_network_quality(network: &NetworkMeshSnapshot, loaded_at: &str) -> DataQuality {
    if network.identity.is_empty() {
        return DataQuality::Unavailable;
    }
    let Some(checked) = network.reachability_checked_at.as_deref() else {
        return DataQuality::Unknown;
    };
    let Some(measured) = parse_clock_label(checked) else {
        return DataQuality::Unknown;
    };
    let reference = parse_clock_label(loaded_at).unwrap_or_else(OffsetDateTime::now_utc);
    let age_secs = (reference - measured).whole_seconds();
    if age_secs < 0 {
        // Mild clock skew within bound → Current; beyond skew → Unknown (#279).
        if age_secs >= -NETWORK_OBSERVATION_MAX_SKEW_SECS {
            return DataQuality::Current;
        }
        return DataQuality::Unknown;
    }
    if age_secs as u64 > NETWORK_OBSERVATION_STALE_SECS {
        DataQuality::Stale
    } else {
        DataQuality::Current
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

    let configured = peer_listen
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let (local_bind, local_bind_provenance) = if let Some(bind) = configured {
        (Some(bind), LocalBindProvenance::Configured)
    } else if let Some(port) = reach.local_port {
        (
            Some(format!("127.0.0.1:{port}")),
            LocalBindProvenance::ReachabilityRecord,
        )
    } else {
        (None, LocalBindProvenance::None)
    };
    // Desktop runtime has no in-process peer accept loop — bind display ≠ listener proof.
    let local_listener_proven = false;

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
        local_bind_provenance,
        local_listener_proven,
        external_observed,
        reachability_status: status_label(reach.status),
        top_level: top.as_str().into(),
        direct_reachability: direct_reachability.into(),
        relay_reachability: relay_reachability.into(),
        reachability_checked_at: reach.status_observation_at().map(str::to_string),
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
        projection_now_label(),
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
        assert_eq!(snap.local_bind_provenance, LocalBindProvenance::Configured);
        assert!(!snap.local_listener_proven);
        assert_eq!(
            snap.reachability_checked_at.as_deref(),
            Some("2026-09-05T12:00:00Z")
        );
        assert_eq!(snap.reachability_status, "LOCAL_ONLY");
        assert_eq!(snap.top_level, "LOCAL ONLY");
        assert_ne!(snap.top_level, "OFFLINE");
        assert_eq!(snap.direct_reachability, "no");
        assert_eq!(snap.address_book_count, 1);
        assert_eq!(snap.live_session_count, None);
    }

    #[test]
    fn reachability_port_without_config_is_not_listener_proof() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let _ = write_node(root, "mesh-bind", [65u8; 32]);
        let mut reach = ReachabilityLocalState::default();
        reach
            .mark_local_bind(49157, "2026-09-07T12:00:00Z")
            .unwrap();
        reach.save(root).unwrap();

        let snap = load_network_mesh_snapshot(root, None).unwrap();
        assert_eq!(snap.local_bind.as_deref(), Some("127.0.0.1:49157"));
        assert_eq!(
            snap.local_bind_provenance,
            LocalBindProvenance::ReachabilityRecord
        );
        assert!(!snap.local_listener_proven);
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
        // Identity present but no reachability measurement → Unknown, not Current.
        assert_eq!(sys.network_quality, DataQuality::Unknown);
        assert_eq!(sys.observed_at, "unknown");
        assert!(!sys.loaded_at.is_empty());
        assert_ne!(sys.observed_at, sys.loaded_at);
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
        assert_eq!(sys.observed_at, "unknown");
        assert_eq!(sys.loaded_at, "unix:0");
    }

    #[test]
    fn fresh_checked_at_is_current_and_differs_from_loaded_at() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:identity:fresh".into();
        mesh.reachability_checked_at = Some("2026-09-07T12:00:00Z".into());
        mesh.top_level = "LOCAL ONLY".into();
        let sys = SystemSnapshot::from_network(mesh, "2026-09-07T12:02:00Z".into());
        assert_eq!(sys.network_quality, DataQuality::Current);
        assert_eq!(sys.observed_at, "2026-09-07T12:00:00Z");
        assert_eq!(sys.loaded_at, "2026-09-07T12:02:00Z");
        assert_ne!(sys.observed_at, sys.loaded_at);
    }

    #[test]
    fn old_checked_at_is_stale() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:identity:stale".into();
        mesh.reachability_checked_at = Some("2026-09-07T10:00:00Z".into());
        mesh.top_level = "DIRECT".into();
        let sys = SystemSnapshot::from_network(mesh, "2026-09-07T12:00:00Z".into());
        assert_eq!(sys.network_quality, DataQuality::Stale);
        assert_eq!(sys.observed_at, "2026-09-07T10:00:00Z");
        assert_ne!(sys.loaded_at, sys.observed_at);
    }

    #[test]
    fn missing_checked_at_is_unknown_not_current() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:identity:nocheck".into();
        mesh.top_level = "DIRECT".into();
        let q = classify_network_quality(&mesh, "2026-09-07T12:00:00Z");
        assert_eq!(q, DataQuality::Unknown);
    }

    #[test]
    fn future_clock_within_skew_is_current() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:identity:skew".into();
        // 60s ahead of reference — within NETWORK_OBSERVATION_MAX_SKEW_SECS.
        mesh.reachability_checked_at = Some("2026-09-07T12:01:00Z".into());
        let q = classify_network_quality(&mesh, "2026-09-07T12:00:00Z");
        assert_eq!(q, DataQuality::Current);
    }

    #[test]
    fn future_clock_beyond_skew_is_unknown_not_current() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:identity:far-future".into();
        // 10 minutes ahead — beyond 300s skew.
        mesh.reachability_checked_at = Some("2026-09-07T12:10:00Z".into());
        let q = classify_network_quality(&mesh, "2026-09-07T12:00:00Z");
        assert_eq!(q, DataQuality::Unknown);
    }

    #[test]
    fn local_bind_does_not_make_stale_direct_current() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let _ = write_node(root, "mesh-bind-direct", [66u8; 32]);
        let mut reach = ReachabilityLocalState::default();
        reach.status = ReachabilityStatus::DirectReachable;
        reach.verified_endpoint = Some("127.0.0.1:49157".into());
        reach.probe_evidence = Some("ch-1".into());
        reach.external_checked_at = Some("2026-09-07T10:00:00Z".into());
        reach.checked_at = Some("2026-09-07T10:00:00Z".into());
        reach.local_port = Some(49157);
        reach
            .mark_local_bind(49157, "2026-09-07T12:00:00Z")
            .unwrap();
        reach.save(root).unwrap();

        let snap = load_network_mesh_snapshot(root, None).unwrap();
        assert_eq!(snap.top_level, "DIRECT");
        assert_eq!(
            snap.reachability_checked_at.as_deref(),
            Some("2026-09-07T10:00:00Z"),
            "status observation must stay external, not local bind time"
        );
        let sys = SystemSnapshot::from_network(snap, "2026-09-07T12:00:00Z".into());
        assert_eq!(sys.network_quality, DataQuality::Stale);
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
        assert_eq!(
            snap.reachability_checked_at.as_deref(),
            Some("2026-09-05T12:00:00Z")
        );
    }
}
