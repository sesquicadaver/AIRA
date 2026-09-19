//! Settings apply lifecycle (`#262` / `#268` / `#356` / `desktop-ux` §5).
//!
//! Disk-saved values and runtime-applied values are tracked separately.
//! Applied values come only from confirmed runtime (Start/attach/status).
//! When unconfirmed → [`SettingsApplyPhase::Undefined`], never fake Applied from settings.
//! Draft edits that are not yet saved → [`SettingsApplyPhase::Changed`] (`#356` / RFC-0239).

use aira_desktop_runtime::{
    DesktopSettings, LifecycleStatus, LlmBackend, NetworkProfile, PidRecordView, StartOutcome,
    DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
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
    /// Connection draft in the UI differs from disk-saved values (`#356`).
    Changed,
}

/// Subset of settings that bind into a started node / peer listen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedRuntimeSettings {
    pub network_profile: NetworkProfile,
    pub peer_listen: Option<String>,
    pub relay_ttl_days: Option<u32>,
    pub http_listen: String,
    pub llm_backend: LlmBackend,
    pub llm_process_bin: Option<String>,
    pub llm_ollama_model: Option<String>,
    pub llm_process_timeout_ms: Option<u64>,
}

impl AppliedRuntimeSettings {
    /// Build from disk settings — only valid after a Start that used those settings.
    pub fn from_settings(s: &DesktopSettings) -> Self {
        Self {
            network_profile: s.network_profile,
            peer_listen: s.peer_listen.clone(),
            relay_ttl_days: s.relay_ttl_days,
            http_listen: s.http_listen.clone(),
            llm_backend: s.llm_backend,
            llm_process_bin: s.llm_process_bin.clone(),
            llm_ollama_model: s.llm_ollama_model.clone(),
            llm_process_timeout_ms: s.llm_process_timeout_ms,
        }
    }

    /// Confirmed values after a successful Start / attach outcome (`#268`).
    ///
    /// Listens come from the outcome; profile/TTL/LLM are those used for the successful start.
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
                llm_backend: rec.llm_backend,
                llm_process_bin: rec.llm_process_bin.clone(),
                llm_ollama_model: rec.llm_ollama_model.clone(),
                llm_process_timeout_ms: rec.llm_process_timeout_ms,
            }),
            _ => Some(Self {
                network_profile: NetworkProfile::P0,
                peer_listen: None,
                relay_ttl_days: None,
                http_listen: rec.listen.clone(),
                llm_backend: rec.llm_backend,
                llm_process_bin: rec.llm_process_bin.clone(),
                llm_ollama_model: rec.llm_ollama_model.clone(),
                llm_process_timeout_ms: rec.llm_process_timeout_ms,
            }),
        }
    }

    /// True when saved settings differ in fields that need node restart to take effect.
    pub fn differs_restart_relevant(&self, saved: &DesktopSettings) -> bool {
        self.network_profile != saved.network_profile
            || self.peer_listen != saved.peer_listen
            || self.relay_ttl_days != saved.relay_ttl_days
            || self.http_listen != saved.http_listen
            || self.llm_backend != saved.llm_backend
            || self.llm_process_bin != saved.llm_process_bin
            || self.llm_process_timeout_ms != saved.llm_process_timeout_ms
    }
}

/// True when Connection draft fields differ from disk-saved values (`#356`).
pub fn settings_connection_draft_dirty(
    saved: &DesktopSettings,
    peer_listen_edit: &str,
    relay_ttl_edit: &str,
) -> bool {
    if saved.network_profile.requires_peer_listen() {
        let disk = saved.peer_listen.as_deref().unwrap_or(DEFAULT_PEER_LISTEN);
        if peer_listen_edit.trim() != disk.trim() {
            return true;
        }
    }
    if saved.network_profile.is_relay_profile() {
        let disk = saved
            .relay_ttl_days
            .map(|d| d.to_string())
            .unwrap_or_else(|| DEFAULT_RELAY_TTL_DAYS.to_string());
        if relay_ttl_edit.trim() != disk.trim() {
            return true;
        }
    }
    false
}

/// Compute the Settings lifecycle badge from draft + saved vs confirmed applied.
///
/// Priority: Changed (unsaved draft) → RestartNeeded → Applied → Undefined.
pub fn settings_apply_phase(
    saved: &DesktopSettings,
    applied: Option<&AppliedRuntimeSettings>,
    draft_dirty: bool,
) -> SettingsApplyPhase {
    if draft_dirty {
        return SettingsApplyPhase::Changed;
    }
    match applied {
        None => SettingsApplyPhase::Undefined,
        Some(a) if a.differs_restart_relevant(saved) => SettingsApplyPhase::RestartNeeded,
        Some(_) => SettingsApplyPhase::Applied,
    }
}

/// Disk-only phase (ignore UI draft) — for restart CTA (`#356`).
pub fn settings_apply_phase_disk(
    saved: &DesktopSettings,
    applied: Option<&AppliedRuntimeSettings>,
) -> SettingsApplyPhase {
    settings_apply_phase(saved, applied, false)
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
            settings_apply_phase(&s, None, false),
            SettingsApplyPhase::Undefined
        );
    }

    #[test]
    fn saved_equals_confirmed_is_applied_phase() {
        let s = sample_settings(NetworkProfile::P0);
        let applied = AppliedRuntimeSettings::from_settings(&s);
        assert_eq!(
            settings_apply_phase(&s, Some(&applied), false),
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
            settings_apply_phase(&saved, Some(&applied), false),
            SettingsApplyPhase::RestartNeeded
        );
        let applied2 = AppliedRuntimeSettings::from_settings(&saved);
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied2), false),
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
            settings_apply_phase(&saved, Some(&applied), false),
            SettingsApplyPhase::Applied
        );
    }

    #[test]
    fn llm_backend_change_needs_restart() {
        let mut saved = sample_settings(NetworkProfile::P0);
        let applied = AppliedRuntimeSettings::from_settings(&saved);
        saved.llm_backend = LlmBackend::Process;
        saved.llm_ollama_model = Some("llama3:latest".into());
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied), false),
            SettingsApplyPhase::RestartNeeded
        );
    }

    #[test]
    fn process_model_switch_does_not_need_restart() {
        let mut saved = sample_settings(NetworkProfile::P0);
        saved.llm_backend = LlmBackend::Process;
        saved.llm_ollama_model = Some("model-a:latest".into());
        let applied = AppliedRuntimeSettings::from_settings(&saved);
        saved.llm_ollama_model = Some("model-b:latest".into());
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied), false),
            SettingsApplyPhase::Applied
        );
    }

    /// Pack B / audit #9: applied LLM facts from pidfile; disk tip differ → RestartNeeded.
    #[test]
    fn attach_applied_llm_from_pidfile_not_disk() {
        let mut disk = sample_settings(NetworkProfile::P0);
        disk.llm_backend = LlmBackend::Process;
        disk.llm_ollama_model = Some("disk-tip:latest".into());
        let mut pidfile = disk.clone();
        pidfile.llm_backend = LlmBackend::Mock;
        pidfile.llm_ollama_model = None;
        let applied = AppliedRuntimeSettings::from_settings(&pidfile);
        assert_eq!(applied.llm_backend, LlmBackend::Mock);
        assert!(applied.llm_ollama_model.is_none());
        assert_eq!(
            settings_apply_phase(&disk, Some(&applied), false),
            SettingsApplyPhase::RestartNeeded
        );
    }

    #[test]
    fn draft_dirty_is_changed_before_restart() {
        let mut saved = sample_settings(NetworkProfile::P1);
        saved.peer_listen = Some("127.0.0.1:4001".into());
        let applied = AppliedRuntimeSettings::from_settings(&saved);
        // Disk already matches applied, but draft differs.
        assert!(settings_connection_draft_dirty(
            &saved,
            "127.0.0.1:4111",
            "7"
        ));
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied), true),
            SettingsApplyPhase::Changed
        );
        // Disk-only phase stays Applied (restart CTA must not fire on draft alone).
        assert_eq!(
            settings_apply_phase_disk(&saved, Some(&applied)),
            SettingsApplyPhase::Applied
        );
    }

    #[test]
    fn draft_dirty_false_when_edits_match_disk() {
        let mut saved = sample_settings(NetworkProfile::P1);
        saved.peer_listen = Some("127.0.0.1:4001".into());
        assert!(!settings_connection_draft_dirty(
            &saved,
            "127.0.0.1:4001",
            "7"
        ));
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
            llm_backend: LlmBackend::Mock,
            llm_process_bin: None,
            llm_ollama_model: None,
            llm_process_timeout_ms: None,
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
            llm_backend: LlmBackend::Mock,
            llm_process_bin: None,
            llm_ollama_model: None,
            llm_process_timeout_ms: None,
        };
        let applied =
            AppliedRuntimeSettings::from_status(LifecycleStatus::Running, Some(&view)).unwrap();
        assert_eq!(applied.network_profile, NetworkProfile::P0);
        assert_eq!(applied.http_listen, "127.0.0.1:8787");
        assert!(applied.peer_listen.is_none());
        assert_eq!(applied.llm_backend, LlmBackend::Mock);
        let saved = sample_settings(NetworkProfile::P1);
        assert_eq!(
            settings_apply_phase(&saved, Some(&applied), false),
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
            llm_backend: LlmBackend::Process,
            llm_process_bin: Some("ollama".into()),
            llm_ollama_model: Some("llama3:latest".into()),
            llm_process_timeout_ms: Some(120_000),
        };
        let applied =
            AppliedRuntimeSettings::from_status(LifecycleStatus::Running, Some(&view)).unwrap();
        assert_eq!(applied.network_profile, NetworkProfile::P1);
        assert_eq!(applied.peer_listen.as_deref(), Some("127.0.0.1:4001"));
        assert_eq!(applied.llm_backend, LlmBackend::Process);
        assert_eq!(applied.llm_ollama_model.as_deref(), Some("llama3:latest"));
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
            settings_apply_phase(&ui_later, Some(&applied), false),
            SettingsApplyPhase::RestartNeeded
        );
    }
}
