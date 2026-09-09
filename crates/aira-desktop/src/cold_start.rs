//! Cold-start empty-profile guidance (`#293` / `phase-r-plan` R7).
//!
//! New / default root on **P0** with an empty address book gets one honest
//! guidance line on Connection plus the existing primary CTA. Never invent
//! Applied or CONNECTED for this state.

use aira_desktop_runtime::NetworkProfile;

use crate::system_view::ConnectionConclusion;

/// True when Connection should show cold-start empty-profile guidance.
///
/// Matches default Developer Preview: local-only profile, no trusted peers yet.
pub fn is_cold_start_empty_profile(profile: NetworkProfile, address_book_count: usize) -> bool {
    profile == NetworkProfile::P0 && address_book_count == 0
}

/// Honesty guard: cold-start must not be painted as Direct/Relayed.
pub fn cold_start_forbids_connected_claim(
    profile: NetworkProfile,
    address_book_count: usize,
    connection: ConnectionConclusion,
) -> bool {
    if !is_cold_start_empty_profile(profile, address_book_count) {
        return true;
    }
    !matches!(
        connection,
        ConnectionConclusion::Direct | ConnectionConclusion::Relayed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p0_empty_book_is_cold_start() {
        assert!(is_cold_start_empty_profile(NetworkProfile::P0, 0));
    }

    #[test]
    fn p1_or_nonempty_book_is_not_cold_start() {
        assert!(!is_cold_start_empty_profile(NetworkProfile::P1, 0));
        assert!(!is_cold_start_empty_profile(NetworkProfile::P0, 1));
    }

    #[test]
    fn cold_start_unknown_is_allowed_not_direct() {
        assert!(cold_start_forbids_connected_claim(
            NetworkProfile::P0,
            0,
            ConnectionConclusion::Unknown
        ));
        assert!(cold_start_forbids_connected_claim(
            NetworkProfile::P0,
            0,
            ConnectionConclusion::LocalOnly
        ));
        assert!(!cold_start_forbids_connected_claim(
            NetworkProfile::P0,
            0,
            ConnectionConclusion::Direct
        ));
    }

    #[test]
    fn non_cold_start_does_not_forbid_direct() {
        assert!(cold_start_forbids_connected_claim(
            NetworkProfile::P1,
            1,
            ConnectionConclusion::Direct
        ));
    }
}
