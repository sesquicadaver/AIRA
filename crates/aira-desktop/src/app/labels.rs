use aira_desktop_runtime::{
    LifecycleStatus, NetworkProfile, UiLang, DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};

use super::i18n::Labels;

pub(super) fn status_label(st: LifecycleStatus, lang: UiLang) -> &'static str {
    let l = Labels::get(lang);
    match st {
        LifecycleStatus::Stopped => l.st_stopped,
        LifecycleStatus::Starting => l.st_starting,
        LifecycleStatus::Running => l.st_running,
        LifecycleStatus::Unhealthy => l.st_unhealthy,
        LifecycleStatus::Stopping => l.st_stopping,
        LifecycleStatus::Failed => l.st_failed,
    }
}

pub(super) fn format_peer_running(
    profile: NetworkProfile,
    pid: u32,
    listen: &str,
    relay_ttl_days: Option<u32>,
) -> String {
    match profile {
        NetworkProfile::P4 => {
            format!("peer running (gossip+dht+apply-book) · pid {pid} @ {listen}")
        }
        NetworkProfile::P3 => {
            let ttl = relay_ttl_days.unwrap_or(DEFAULT_RELAY_TTL_DAYS);
            format!("peer running (relay · TTL {ttl}d) · pid {pid} @ {listen}")
        }
        NetworkProfile::P2 => format!("peer running (dht+apply-book) · pid {pid} @ {listen}"),
        NetworkProfile::P1 => format!("peer running · pid {pid} @ {listen}"),
        _ => format!("peer running · pid {pid} @ {listen}"),
    }
}

pub(super) fn format_peer_not_running(profile: NetworkProfile) -> String {
    match profile {
        NetworkProfile::P4 => "peer not running (Start with P4 gossip)".into(),
        NetworkProfile::P3 => "peer not running (Start with P3 relay)".into(),
        NetworkProfile::P2 => "peer not running (Start with P2)".into(),
        NetworkProfile::P1 => "peer not running (Start with P1)".into(),
        _ => "peer not running".into(),
    }
}

pub(super) fn format_peer_configured(
    profile: NetworkProfile,
    listen: Option<&str>,
    relay_ttl_days: Option<u32>,
) -> String {
    let addr = listen.unwrap_or(DEFAULT_PEER_LISTEN);
    match profile {
        NetworkProfile::P4 => format!("peer configured (gossip+dht+apply-book) · {addr}"),
        NetworkProfile::P3 => {
            let ttl = relay_ttl_days.unwrap_or(DEFAULT_RELAY_TTL_DAYS);
            format!("peer configured (relay · TTL {ttl}d) · {addr}")
        }
        NetworkProfile::P2 => format!("peer configured (dht+apply-book) · {addr}"),
        NetworkProfile::P1 => format!("peer configured · {addr}"),
        _ => format!("peer configured · {addr}"),
    }
}
