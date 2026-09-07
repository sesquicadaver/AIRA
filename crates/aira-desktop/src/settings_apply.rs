//! Settings apply lifecycle (`#262` / `desktop-ux` §5).
//!
//! Disk-saved values and runtime-applied values are tracked separately.
//! Network/listen changes require Stop→Start; language/autostart apply immediately.

use aira_desktop_runtime::{DesktopSettings, NetworkProfile};

/// Coarse apply phase shown on the Settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsApplyPhase {
    /// Saved values match what the running Desktop considers applied.
    Applied,
    /// Saved to disk; runtime still uses previous network/listen values.
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
    pub fn from_settings(s: &DesktopSettings) -> Self {
        Self {
            network_profile: s.network_profile,
            peer_listen: s.peer_listen.clone(),
            relay_ttl_days: s.relay_ttl_days,
            http_listen: s.http_listen.clone(),
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

/// Compute the Settings lifecycle badge from saved vs applied.
pub fn settings_apply_phase(
    saved: &DesktopSettings,
    applied: &AppliedRuntimeSettings,
) -> SettingsApplyPhase {
    if applied.differs_restart_relevant(saved) {
        SettingsApplyPhase::RestartNeeded
    } else {
        SettingsApplyPhase::Applied
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
    fn saved_equals_applied_is_applied_phase() {
        let s = sample_settings(NetworkProfile::P0);
        let applied = AppliedRuntimeSettings::from_settings(&s);
        assert_eq!(
            settings_apply_phase(&s, &applied),
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
            settings_apply_phase(&saved, &applied),
            SettingsApplyPhase::RestartNeeded
        );
        let applied2 = AppliedRuntimeSettings::from_settings(&saved);
        assert_eq!(
            settings_apply_phase(&saved, &applied2),
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
            settings_apply_phase(&saved, &applied),
            SettingsApplyPhase::Applied
        );
    }
}
