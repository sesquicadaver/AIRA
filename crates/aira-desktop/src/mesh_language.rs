//! Human-primary mesh / strip language (`#289` / `phase-r-plan` R3).
//!
//! Primary copy uses [`ConnectionConclusion`]; raw top-level enums
//! (`UNKNOWN`, `LOCAL ONLY`, …) stay secondary / tech. Strip must not
//! paint unexpected labels as OFFLINE when Connection says Unknown.

use aira_desktop_runtime::DataQuality;

use crate::system_view::ConnectionConclusion;

/// Compact Network cell on the status strip (human phrases, not raw enums).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripNetworkPhrase {
    Connected,
    LocalOnly,
    Stale,
    NotChecked,
    Offline,
}

impl StripNetworkPhrase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connected => "strip.network.connected",
            Self::LocalOnly => "strip.network.local_only",
            Self::Stale => "strip.network.stale",
            Self::NotChecked => "strip.network.not_checked",
            Self::Offline => "strip.network.offline",
        }
    }
}

/// Map freshness + honest connection conclusion → strip Network phrase.
///
/// Stale/Unknown/Unavailable quality always wins over Direct/Relayed so
/// strip never claims «connected» from stale evidence (`#267` / `#289`).
pub fn strip_network_phrase(
    quality: DataQuality,
    connection: ConnectionConclusion,
) -> StripNetworkPhrase {
    match quality {
        DataQuality::Stale => StripNetworkPhrase::Stale,
        DataQuality::Unknown | DataQuality::Unavailable => StripNetworkPhrase::NotChecked,
        DataQuality::Current => match connection {
            ConnectionConclusion::Direct | ConnectionConclusion::Relayed => {
                StripNetworkPhrase::Connected
            }
            ConnectionConclusion::LocalOnly | ConnectionConclusion::OutboundOnly => {
                StripNetworkPhrase::LocalOnly
            }
            ConnectionConclusion::Unknown => StripNetworkPhrase::NotChecked,
            ConnectionConclusion::Offline => StripNetworkPhrase::Offline,
        },
    }
}

/// Derive strip phrase from raw mesh top-level (same mapping as Connection).
pub fn strip_network_from_top_level(quality: DataQuality, top_level: &str) -> StripNetworkPhrase {
    strip_network_phrase(quality, ConnectionConclusion::from_top_level(top_level))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_top_level_is_not_checked_not_offline() {
        assert_eq!(
            strip_network_from_top_level(DataQuality::Current, "UNKNOWN"),
            StripNetworkPhrase::NotChecked
        );
        assert_ne!(
            strip_network_from_top_level(DataQuality::Current, "UNKNOWN"),
            StripNetworkPhrase::Offline
        );
        assert_eq!(
            ConnectionConclusion::from_top_level("UNKNOWN"),
            ConnectionConclusion::Unknown
        );
    }

    #[test]
    fn unexpected_label_matches_connection_unknown_not_offline() {
        let phrase = strip_network_from_top_level(DataQuality::Current, "WEIRD BANNER");
        assert_eq!(phrase, StripNetworkPhrase::NotChecked);
        assert_eq!(
            ConnectionConclusion::from_top_level("WEIRD BANNER"),
            ConnectionConclusion::Unknown
        );
    }

    #[test]
    fn offline_top_level_is_offline() {
        assert_eq!(
            strip_network_from_top_level(DataQuality::Current, "OFFLINE"),
            StripNetworkPhrase::Offline
        );
    }

    #[test]
    fn stale_quality_overrides_direct() {
        assert_eq!(
            strip_network_from_top_level(DataQuality::Stale, "DIRECT"),
            StripNetworkPhrase::Stale
        );
    }

    #[test]
    fn direct_current_is_connected() {
        assert_eq!(
            strip_network_from_top_level(DataQuality::Current, "DIRECT"),
            StripNetworkPhrase::Connected
        );
        assert_eq!(
            strip_network_from_top_level(DataQuality::Current, "RELAYED"),
            StripNetworkPhrase::Connected
        );
    }

    #[test]
    fn local_and_outbound_are_local_only_phrase() {
        assert_eq!(
            strip_network_from_top_level(DataQuality::Current, "LOCAL ONLY"),
            StripNetworkPhrase::LocalOnly
        );
        assert_eq!(
            strip_network_from_top_level(DataQuality::Current, "OUTBOUND ONLY"),
            StripNetworkPhrase::LocalOnly
        );
    }

    #[test]
    fn strip_agrees_with_connection_conclusion() {
        for top in [
            "DIRECT",
            "RELAYED",
            "OUTBOUND ONLY",
            "LOCAL ONLY",
            "UNKNOWN",
            "OFFLINE",
            "GARBAGE",
        ] {
            let c = ConnectionConclusion::from_top_level(top);
            let phrase = strip_network_phrase(DataQuality::Current, c);
            match c {
                ConnectionConclusion::Direct | ConnectionConclusion::Relayed => {
                    assert_eq!(phrase, StripNetworkPhrase::Connected);
                }
                ConnectionConclusion::LocalOnly | ConnectionConclusion::OutboundOnly => {
                    assert_eq!(phrase, StripNetworkPhrase::LocalOnly);
                }
                ConnectionConclusion::Unknown => {
                    assert_eq!(phrase, StripNetworkPhrase::NotChecked);
                }
                ConnectionConclusion::Offline => {
                    assert_eq!(phrase, StripNetworkPhrase::Offline);
                }
            }
        }
    }
}
