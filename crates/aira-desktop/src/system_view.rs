//! Human-first System status conclusions (`#261` / `desktop-ux` §4).
//!
//! Honesty: UNKNOWN ≠ OFFLINE; AddressBook count ≠ live sessions; no invented model.
//! Freshness (`#267`): measurement time ≠ load time; Stale/Unknown cannot paint as Current.

use aira_desktop_runtime::{DataQuality, LifecycleStatus, SystemSnapshot};

#[cfg(test)]
use aira_desktop_runtime::NetworkMeshSnapshot;

/// Program (local AIRA process) conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramConclusion {
    Running,
    Stopped,
    Starting,
    Stopping,
    Unhealthy,
    Failed,
}

/// Model section — Desktop does not invent a selected model (`#261`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelConclusion {
    /// No authoritative model observation on this screen yet.
    NotChecked,
}

/// Connection conclusion derived from mesh top-level (never collapses UNKNOWN→OFFLINE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionConclusion {
    Direct,
    Relayed,
    OutboundOnly,
    LocalOnly,
    Unknown,
    Offline,
}

impl ConnectionConclusion {
    pub fn from_top_level(top: &str) -> Self {
        match top {
            "DIRECT" => Self::Direct,
            "RELAYED" => Self::Relayed,
            "OUTBOUND ONLY" => Self::OutboundOnly,
            "LOCAL ONLY" => Self::LocalOnly,
            "OFFLINE" => Self::Offline,
            // UNKNOWN and any unexpected label stay Unknown — not Offline.
            _ => Self::Unknown,
        }
    }
}

/// Inputs for the four System sections.
#[derive(Debug, Clone)]
pub struct SystemStatusView {
    pub program: ProgramConclusion,
    pub model: ModelConclusion,
    pub connection: ConnectionConclusion,
    pub observed_at: String,
    pub loaded_at: String,
    pub network_quality: DataQuality,
    pub live_sessions_quality: DataQuality,
    pub address_book_count: usize,
    /// `None` = not observed (must not render as zero or book size).
    pub live_session_count: Option<usize>,
    pub top_level: String,
}

impl SystemStatusView {
    pub fn from_parts(lifecycle: LifecycleStatus, system: &SystemSnapshot) -> Self {
        let program = match lifecycle {
            LifecycleStatus::Running => ProgramConclusion::Running,
            LifecycleStatus::Stopped => ProgramConclusion::Stopped,
            LifecycleStatus::Starting => ProgramConclusion::Starting,
            LifecycleStatus::Stopping => ProgramConclusion::Stopping,
            LifecycleStatus::Unhealthy => ProgramConclusion::Unhealthy,
            LifecycleStatus::Failed => ProgramConclusion::Failed,
        };
        let net = &system.network;
        Self {
            program,
            model: ModelConclusion::NotChecked,
            connection: ConnectionConclusion::from_top_level(&net.top_level),
            observed_at: system.observed_at.clone(),
            loaded_at: system.loaded_at.clone(),
            network_quality: system.network_quality,
            live_sessions_quality: system.live_sessions_quality,
            address_book_count: net.address_book_count,
            live_session_count: net.live_session_count,
            top_level: net.top_level.clone(),
        }
    }

    #[cfg(test)]
    pub fn from_mesh(
        lifecycle: LifecycleStatus,
        mesh: &NetworkMeshSnapshot,
        loaded_at: String,
    ) -> Self {
        Self::from_parts(
            lifecycle,
            &SystemSnapshot::from_network(mesh.clone(), loaded_at),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_desktop_runtime::NetworkMeshSnapshot;

    #[test]
    fn unknown_top_level_is_not_offline() {
        assert_eq!(
            ConnectionConclusion::from_top_level("UNKNOWN"),
            ConnectionConclusion::Unknown
        );
        assert_ne!(
            ConnectionConclusion::from_top_level("UNKNOWN"),
            ConnectionConclusion::Offline
        );
        assert_eq!(
            ConnectionConclusion::from_top_level("OFFLINE"),
            ConnectionConclusion::Offline
        );
    }

    #[test]
    fn address_book_not_confused_with_unobserved_sessions() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:id:test".into();
        mesh.top_level = "LOCAL ONLY".into();
        mesh.address_book_count = 3;
        mesh.live_session_count = None;
        let view = SystemStatusView::from_mesh(LifecycleStatus::Running, &mesh, "unix:1".into());
        assert_eq!(view.address_book_count, 3);
        assert_eq!(view.live_session_count, None);
        assert_eq!(view.live_sessions_quality, DataQuality::Unknown);
        assert_eq!(view.connection, ConnectionConclusion::LocalOnly);
        assert_eq!(view.model, ModelConclusion::NotChecked);
        assert_eq!(view.network_quality, DataQuality::Unknown);
        assert_eq!(view.observed_at, "unknown");
        assert_eq!(view.loaded_at, "unix:1");
    }

    #[test]
    fn stale_measurement_surfaces_on_view() {
        let mut mesh = NetworkMeshSnapshot::unavailable();
        mesh.identity = "aira:id:stale".into();
        mesh.top_level = "DIRECT".into();
        mesh.reachability_checked_at = Some("2026-09-07T10:00:00Z".into());
        let view =
            SystemStatusView::from_mesh(LifecycleStatus::Running, &mesh, "2026-09-07T12:00:00Z".into());
        assert_eq!(view.network_quality, DataQuality::Stale);
        assert_eq!(view.observed_at, "2026-09-07T10:00:00Z");
        assert_eq!(view.loaded_at, "2026-09-07T12:00:00Z");
        assert_eq!(view.connection, ConnectionConclusion::Direct);
    }

    #[test]
    fn program_maps_lifecycle() {
        let mesh = NetworkMeshSnapshot::unavailable();
        let v = SystemStatusView::from_mesh(LifecycleStatus::Stopped, &mesh, "unix:0".into());
        assert_eq!(v.program, ProgramConclusion::Stopped);
    }
}
