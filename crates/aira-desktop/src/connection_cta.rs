//! Connection next-step primary CTA (`#287` / `phase-r-plan` §9).
//!
//! Exactly one primary corrective action on System → Connection.
//! Honesty: never invent CONNECTED/Applied; UNKNOWN stays ≠ OFFLINE;
//! empty AddressBook is not live sessions.

use aira_desktop_runtime::NetworkProfile;

use crate::lexicon::HelpId;
use crate::settings_apply::SettingsApplyPhase;
use crate::system_view::{ConnectionConclusion, ProgramConclusion};

/// Single primary Connection CTA (wire id for tests / F1 later).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionPrimaryCta {
    /// Saved profile is P0 — enable peer-capable profile (P1).
    EnablePrivateNetwork,
    /// Peer profile on, but AddressBook empty — import invite.
    ImportInvite,
    /// RestartNeeded while node is up — Stop first.
    StopToApply,
    /// RestartNeeded while node is down — Start to apply.
    StartToApply,
    /// Reachability not yet useful — refresh observation.
    RefreshStatus,
    /// Direct/Relayed (or in-flight lifecycle): no primary button.
    NoneOk,
}

impl ConnectionPrimaryCta {
    /// Stable wire id (not localized).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnablePrivateNetwork => "connection.enable_private_network",
            Self::ImportInvite => "connection.import_invite",
            Self::StopToApply => "connection.stop_to_apply",
            Self::StartToApply => "connection.start_to_apply",
            Self::RefreshStatus => "connection.refresh_status",
            Self::NoneOk => "connection.none_ok",
        }
    }

    /// Default Help topic for this CTA.
    pub fn help_id(self) -> HelpId {
        match self {
            Self::EnablePrivateNetwork => HelpId::NetworkConnect,
            Self::ImportInvite => HelpId::NetworkTrust,
            Self::StopToApply | Self::StartToApply => HelpId::NodeLifecycle,
            Self::RefreshStatus => HelpId::NetworkReachability,
            Self::NoneOk => HelpId::NetworkReachability,
        }
    }

    /// True when the Connection panel should render a primary button.
    pub fn is_button(self) -> bool {
        !matches!(self, Self::NoneOk)
    }
}

/// Inputs for the Connection CTA matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionCtaInput {
    pub profile: NetworkProfile,
    pub connection: ConnectionConclusion,
    pub address_book_count: usize,
    pub apply_phase: SettingsApplyPhase,
    pub program: ProgramConclusion,
    pub restart_hint: bool,
}

/// Pick exactly one primary Connection CTA from honest state.
///
/// Priority (phase-r §9 + Phase S `#301`): RestartNeeded → P0 enable network →
/// empty book import → **stopped peer profile → Start (not Refresh-only)** →
/// Unknown/LocalOnly/Outbound/Offline refresh → Direct/Relayed OK.
pub fn primary_connection_cta(input: ConnectionCtaInput) -> ConnectionPrimaryCta {
    let needs_restart =
        input.apply_phase == SettingsApplyPhase::RestartNeeded || input.restart_hint;
    if needs_restart {
        return match input.program {
            ProgramConclusion::Running | ProgramConclusion::Unhealthy => {
                ConnectionPrimaryCta::StopToApply
            }
            ProgramConclusion::Stopped | ProgramConclusion::Failed => {
                ConnectionPrimaryCta::StartToApply
            }
            ProgramConclusion::Starting | ProgramConclusion::Stopping => {
                ConnectionPrimaryCta::NoneOk
            }
        };
    }
    if input.profile == NetworkProfile::P0 {
        return ConnectionPrimaryCta::EnablePrivateNetwork;
    }
    if input.address_book_count == 0 {
        return ConnectionPrimaryCta::ImportInvite;
    }
    // Phase S `#301`: after Stop, peer profiles must not dead-end on Refresh-only.
    let stopped = matches!(
        input.program,
        ProgramConclusion::Stopped | ProgramConclusion::Failed
    );
    if stopped && input.profile.requires_peer_listen() {
        return ConnectionPrimaryCta::StartToApply;
    }
    match input.connection {
        ConnectionConclusion::Direct | ConnectionConclusion::Relayed => {
            ConnectionPrimaryCta::NoneOk
        }
        ConnectionConclusion::Unknown
        | ConnectionConclusion::LocalOnly
        | ConnectionConclusion::OutboundOnly
        | ConnectionConclusion::Offline => ConnectionPrimaryCta::RefreshStatus,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ConnectionCtaInput {
        ConnectionCtaInput {
            profile: NetworkProfile::P1,
            connection: ConnectionConclusion::Unknown,
            address_book_count: 1,
            apply_phase: SettingsApplyPhase::Applied,
            program: ProgramConclusion::Running,
            restart_hint: false,
        }
    }

    #[test]
    fn restart_needed_running_is_stop() {
        let mut i = base();
        i.apply_phase = SettingsApplyPhase::RestartNeeded;
        assert_eq!(primary_connection_cta(i), ConnectionPrimaryCta::StopToApply);
        assert_eq!(
            ConnectionPrimaryCta::StopToApply.as_str(),
            "connection.stop_to_apply"
        );
    }

    #[test]
    fn restart_hint_stopped_is_start() {
        let mut i = base();
        i.restart_hint = true;
        i.program = ProgramConclusion::Stopped;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::StartToApply
        );
    }

    #[test]
    fn restart_in_flight_has_no_button() {
        let mut i = base();
        i.apply_phase = SettingsApplyPhase::RestartNeeded;
        i.program = ProgramConclusion::Starting;
        assert_eq!(primary_connection_cta(i), ConnectionPrimaryCta::NoneOk);
        assert!(!ConnectionPrimaryCta::NoneOk.is_button());
    }

    #[test]
    fn p0_is_enable_private_network_not_refresh() {
        let mut i = base();
        i.profile = NetworkProfile::P0;
        i.connection = ConnectionConclusion::Unknown;
        i.address_book_count = 0;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::EnablePrivateNetwork
        );
    }

    #[test]
    fn empty_book_on_p1_is_import() {
        let mut i = base();
        i.address_book_count = 0;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::ImportInvite
        );
    }

    #[test]
    fn unknown_with_peers_is_refresh_not_offline() {
        let mut i = base();
        i.connection = ConnectionConclusion::Unknown;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::RefreshStatus
        );
        assert_ne!(i.connection, ConnectionConclusion::Offline);
    }

    #[test]
    fn local_only_with_peers_is_refresh() {
        let mut i = base();
        i.connection = ConnectionConclusion::LocalOnly;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::RefreshStatus
        );
    }

    #[test]
    fn direct_with_peers_is_none_ok() {
        let mut i = base();
        i.connection = ConnectionConclusion::Direct;
        assert_eq!(primary_connection_cta(i), ConnectionPrimaryCta::NoneOk);
    }

    #[test]
    fn relayed_with_peers_is_none_ok() {
        let mut i = base();
        i.connection = ConnectionConclusion::Relayed;
        assert_eq!(primary_connection_cta(i), ConnectionPrimaryCta::NoneOk);
    }

    #[test]
    fn restart_beats_p0_and_empty_book() {
        let mut i = base();
        i.profile = NetworkProfile::P0;
        i.address_book_count = 0;
        i.apply_phase = SettingsApplyPhase::RestartNeeded;
        i.program = ProgramConclusion::Running;
        assert_eq!(primary_connection_cta(i), ConnectionPrimaryCta::StopToApply);
    }

    /// Phase S `#301`: stopped + peer profile + unknown must Start, not Refresh.
    #[test]
    fn stopped_peer_profile_unknown_is_start_not_refresh() {
        let mut i = base();
        i.program = ProgramConclusion::Stopped;
        i.connection = ConnectionConclusion::Unknown;
        i.apply_phase = SettingsApplyPhase::Applied;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::StartToApply
        );
    }

    #[test]
    fn stopped_peer_profile_local_only_is_start() {
        let mut i = base();
        i.program = ProgramConclusion::Stopped;
        i.connection = ConnectionConclusion::LocalOnly;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::StartToApply
        );
    }

    #[test]
    fn failed_peer_profile_is_start_not_refresh() {
        let mut i = base();
        i.program = ProgramConclusion::Failed;
        i.connection = ConnectionConclusion::Offline;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::StartToApply
        );
    }

    #[test]
    fn stopped_p0_still_enable_private_network() {
        let mut i = base();
        i.profile = NetworkProfile::P0;
        i.program = ProgramConclusion::Stopped;
        i.address_book_count = 0;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::EnablePrivateNetwork
        );
    }

    #[test]
    fn stopped_empty_book_still_import_before_start() {
        let mut i = base();
        i.program = ProgramConclusion::Stopped;
        i.address_book_count = 0;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::ImportInvite
        );
    }

    #[test]
    fn running_unknown_still_refresh() {
        let mut i = base();
        i.program = ProgramConclusion::Running;
        i.connection = ConnectionConclusion::Unknown;
        assert_eq!(
            primary_connection_cta(i),
            ConnectionPrimaryCta::RefreshStatus
        );
    }

    #[test]
    fn wire_ids_unique() {
        use std::collections::HashSet;
        let all = [
            ConnectionPrimaryCta::EnablePrivateNetwork,
            ConnectionPrimaryCta::ImportInvite,
            ConnectionPrimaryCta::StopToApply,
            ConnectionPrimaryCta::StartToApply,
            ConnectionPrimaryCta::RefreshStatus,
            ConnectionPrimaryCta::NoneOk,
        ];
        let mut set = HashSet::new();
        for c in all {
            assert!(set.insert(c.as_str()), "dup {}", c.as_str());
            assert!(!c.help_id().as_str().is_empty());
        }
    }
}
