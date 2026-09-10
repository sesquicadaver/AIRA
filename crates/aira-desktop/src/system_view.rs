//! Human-first System status conclusions (`#261` / `desktop-ux` §4).
//!
//! Honesty: UNKNOWN ≠ OFFLINE; AddressBook count ≠ live sessions; no invented model.
//! Freshness (`#267`): measurement time ≠ load time; Stale/Unknown cannot paint as Current.
//! Model triple (`#269`): selected ≠ ready ≠ used-in-result.

use aira_desktop_runtime::{
    DataQuality, LifecycleStatus, ModelFact, ModelTripleConclusion, ModelTripleSnapshot,
    SystemSnapshot,
};

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

/// Model section — three independent facts (`#269`) + executor honesty (`#319`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelConclusion {
    pub selected: ModelFact,
    pub ready: bool,
    pub ready_detail: String,
    pub used: ModelFact,
    pub summary: ModelTripleConclusion,
    /// Staff executor kind (`mock` / `process`); activate-ready ≠ process.
    pub executor_kind: String,
}

impl ModelConclusion {
    pub fn from_triple(t: &ModelTripleSnapshot) -> Self {
        Self {
            selected: t.selected.clone(),
            ready: t.ready,
            ready_detail: t.ready_detail.clone(),
            used: t.used.clone(),
            summary: ModelTripleConclusion::from_triple(t),
            executor_kind: t.executor_kind.clone(),
        }
    }
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
    pub fn from_parts(
        lifecycle: LifecycleStatus,
        system: &SystemSnapshot,
        model: &ModelTripleSnapshot,
    ) -> Self {
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
            model: ModelConclusion::from_triple(model),
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
            &ModelTripleSnapshot::undefined(),
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
        assert_eq!(view.model.summary, ModelTripleConclusion::NoneSelected);
        assert_eq!(view.model.used, ModelFact::Undefined);
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
        let view = SystemStatusView::from_mesh(
            LifecycleStatus::Running,
            &mesh,
            "2026-09-07T12:00:00Z".into(),
        );
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

    #[test]
    fn model_triple_used_not_copied_from_selected() {
        let mut triple = ModelTripleSnapshot::undefined();
        triple.selected = ModelFact::Value("aira:model:selected".into());
        triple.ready = true;
        triple.used = ModelFact::Value("aira:model:used".into());
        let c = ModelConclusion::from_triple(&triple);
        assert_eq!(c.selected.as_display(), "aira:model:selected");
        assert_eq!(c.used.as_display(), "aira:model:used");
        assert_ne!(c.selected, c.used);
        assert_eq!(c.summary, ModelTripleConclusion::UsedInResult);
    }
}
