//! Settings apply lifecycle (`#262` / `#268` / `desktop-ux` §5).
//!
//! Disk-saved values and runtime-applied values are tracked separately.
//! Applied values come only from confirmed runtime (Start/attach/status).
//! When unconfirmed → [`SettingsApplyPhase::Undefined`], never fake Applied from settings.

use aira_desktop_runtime::{
    DesktopSettings, LifecycleStatus, NetworkProfile, PidRecordView, StartOutcome,
};

/// Coarse apply phase shown on the Settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsApplyPhase {
    /// No confirmed running node yet — do not claim Applied.
    Undefined,
    /// Saved values match what the confirmed runtime is using.
    Applied,
    /// Saved to disk; confirmed runtime still uses previous network/listen values.
    RestartNeeded,
}

/// Subset of settings that bind into a started node / peer listen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedRuntimeSettings {
    pub network_profile: NetworkProfile,
    pub peer_listen: Option<String>,
    pub relay_ttl_days: Option<u32>,
    pub http_listen: String,
}

impl AppliedRuntimeSettings {
    /// Build from disk settings — only valid after a Start that used those settings.
    pub fn from_settings(s: &DesktopSettings) -> Self {
        Self {
            network_profile: s.network_profile,
            peer_listen: s.peer_listen.clone(),
            relay_ttl_days: s.relay_ttl_days,
            http_listen: s.http_listen.clone(),
        }
    }

    /// Confirmed values after a successful Start / attach outcome (`#268`).
    ///
    /// Listens come from the outcome; profile/TTL are those used for the successful start.
    pub fn from_start_outcome(outcome: &StartOutcome, used: &DesktopSettings) -> Self {
        let mut applied = Self::from_settings(used);
        applied.http_listen = outcome.listen.clone();
        if used.network_profile.requires_peer_listen() {
            applied.peer_listen = outcome
                .peer_listen
                .clone()
                .or_else(|| used.peer_listen.clone());
        } else {
            applied.peer_listen = None;
            applied.relay_ttl_days = None;
        }
        if !used.network_profile.is_relay_profile() {
            applied.relay_ttl_days = None;
        }
        applied
    }

    /// Project confirmed applied values from a live status record (`#268`).
    ///
    /// Returns `None` when lifecycle is not Running or the pid record is missing.
    pub fn from_status(lifecycle: LifecycleStatus, record: Option<&PidRecordView>) -> Option<Self> {
        if !matches!(lifecycle, LifecycleStatus::Running) {
            return None;
        }
        let rec = record?;
        match (rec.peer_listen.as_ref(), rec.peer_network_profile) {
            (Some(pl), Some(profile)) => Some(Self {
                network_profile: profile,
                peer_listen: Some(pl.clone()),
                relay_ttl_days: rec.peer_relay_ttl_days,
                http_listen: rec.listen.clone(),
            }),
            _ => Some(Self {
                network_profile: NetworkProfile::P0,
                peer_listen: None,
                relay_ttl_days: None,
                http_listen: rec.listen.clone(),
            }),
        }
    }

    /// True when saved settings differ in fields that need node restart to take effect.
    pub fn differs_restart_relevant(&self, saved: &DesktopSettings) -> bool {
        self.network_profile != saved.network_profile
            || self.peer_listen != saved.peer_listen
            || self.relay_ttl_days != saved.relay_ttl_days
            || self.http_listen != saved.http_listen
    }
}

/// Compute the Settings lifecycle badge from saved vs confirmed applied.
pub fn settings_apply_phase(
    saved: &DesktopSettings,
    applied: Option<&AppliedRuntimeSettings>,
) -> SettingsApplyPhase {
    match applied {
        None => SettingsApplyPhase::Undefined,
        Some(a) if a.differs_restart_relevant(saved) => SettingsApplyPhase::RestartNeeded,
        Some(_) => SettingsApplyPhase::Applied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_desktop_runtime::{DesktopPaths, NetworkProfile};

    fn sample_settings(profile: NetworkProfile) -> DesktopSettings {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let mut s = DesktopSettings::default_p0(&paths);
        s.network_profile = profile;
        if profile.requires_peer_listen() {
            s.peer_listen = Some("127.0.0.1:4001".into());
        }
        s
    }

    #[test]
    fn unconfirmed_is_undefined_not_applied() {
        let s = sample_settings(NetworkProfile::P0);
        assert_eq!(
            settings_apply_phase(&s, None),
            SettingsApplyPhase::Undefined
        );
    }

    #[test]
    fn saved_equals_confirmed_is_applied_phase() {
        let s = sample_settings(NetworkProfile::P0);
        let applied = AppliedRuntimeSettings::from_settings(&s);
        assert_eq!(
            settings_apply_phase(&s, Some(&applied)),
            SettingsApplyPhase::Applied
        );
    }

    #[test]
    fn profile_change_needs_restart_until_reapplied() {
        let mut saved = sample_settings(NetworkProfile::P0);
        let applied = AppliedRuntimeSettings::from_settings(&saved);
        saved.network_profile = NetworkProfile::P1;
        saved.peer_listen = Some("127.0.0.1:4001".into());
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied)),
            SettingsApplyPhase::RestartNeeded
        );
        let applied2 = AppliedRuntimeSettings::from_settings(&saved);
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied2)),
            SettingsApplyPhase::Applied
        );
    }

    #[test]
    fn open_ui_change_does_not_affect_restart_relevant() {
        let mut saved = sample_settings(NetworkProfile::P1);
        saved.peer_listen = Some("127.0.0.1:4001".into());
        let applied = AppliedRuntimeSettings::from_settings(&saved);
        saved.open_ui_on_start = !saved.open_ui_on_start;
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied)),
            SettingsApplyPhase::Applied
        );
    }

    #[test]
    fn status_without_running_is_undefined() {
        assert!(AppliedRuntimeSettings::from_status(LifecycleStatus::Stopped, None).is_none());
        let view = PidRecordView {
            pid: 1,
            instance_id: "x".into(),
            root: "/tmp".into(),
            listen: "127.0.0.1:8787".into(),
            peer_pid: None,
            peer_listen: None,
            peer_network_profile: None,
            peer_relay_ttl_days: None,
        };
        assert!(
            AppliedRuntimeSettings::from_status(LifecycleStatus::Stopped, Some(&view)).is_none()
        );
    }

    #[test]
    fn running_http_only_confirms_p0() {
        let view = PidRecordView {
            pid: 1,
            instance_id: "x".into(),
            root: "/tmp".into(),
            listen: "127.0.0.1:8787".into(),
            peer_pid: None,
            peer_listen: None,
            peer_network_profile: None,
            peer_relay_ttl_days: None,
        };
        let applied =
            AppliedRuntimeSettings::from_status(LifecycleStatus::Running, Some(&view)).unwrap();
        assert_eq!(applied.network_profile, NetworkProfile::P0);
        assert_eq!(applied.http_listen, "127.0.0.1:8787");
        assert!(applied.peer_listen.is_none());
        let saved = sample_settings(NetworkProfile::P1);
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied)),
            SettingsApplyPhase::RestartNeeded
        );
    }

    #[test]
    fn running_with_peer_confirms_profile() {
        let view = PidRecordView {
            pid: 1,
            instance_id: "x".into(),
            root: "/tmp".into(),
            listen: "127.0.0.1:8787".into(),
            peer_pid: Some(2),
            peer_listen: Some("127.0.0.1:4001".into()),
            peer_network_profile: Some(NetworkProfile::P1),
            peer_relay_ttl_days: None,
        };
        let applied =
            AppliedRuntimeSettings::from_status(LifecycleStatus::Running, Some(&view)).unwrap();
        assert_eq!(applied.network_profile, NetworkProfile::P1);
        assert_eq!(applied.peer_listen.as_deref(), Some("127.0.0.1:4001"));
    }

    #[test]
    fn start_outcome_uses_outcome_listens() {
        let used = sample_settings(NetworkProfile::P1);
        let outcome = StartOutcome {
            status: LifecycleStatus::Running,
            attached: false,
            pid: Some(9),
            listen: "127.0.0.1:9999".into(),
            instance_id: used.instance_id.clone(),
            data_root: std::env::temp_dir(),
            peer_pid: Some(10),
            peer_listen: Some("127.0.0.1:4001".into()),
            peer_attached: false,
            used_settings: used.clone(),
        };
        let applied = AppliedRuntimeSettings::from_start_outcome(&outcome, &used);
        assert_eq!(applied.http_listen, "127.0.0.1:9999");
        assert_eq!(applied.peer_listen.as_deref(), Some("127.0.0.1:4001"));
        assert_eq!(applied.network_profile, NetworkProfile::P1);
    }

    #[test]
    fn applied_uses_worker_snapshot_not_later_ui_settings() {
        let worker = sample_settings(NetworkProfile::P0);
        let mut ui_later = worker.clone();
        ui_later.network_profile = NetworkProfile::P1;
        ui_later.peer_listen = Some("127.0.0.1:4001".into());
        let outcome = StartOutcome {
            status: LifecycleStatus::Running,
            attached: false,
            pid: Some(1),
            listen: "127.0.0.1:8787".into(),
            instance_id: worker.instance_id.clone(),
            data_root: std::env::temp_dir(),
            peer_pid: None,
            peer_listen: None,
            peer_attached: false,
            used_settings: worker.clone(),
        };
        let applied = AppliedRuntimeSettings::from_start_outcome(&outcome, &outcome.used_settings);
        assert_eq!(applied.network_profile, NetworkProfile::P0);
        assert_ne!(applied.network_profile, ui_later.network_profile);
        assert_eq!(
            settings_apply_phase(&ui_later, Some(&applied)),
            SettingsApplyPhase::RestartNeeded
        );
    }
}
