//! P6 Advanced discovery operators (QUEUE #105) — explicit STUN/discv/FIND shortcuts.

use std::net::SocketAddr;

use aira_peer::{
    iterative_discv_find, query_and_save_stun_reflexive, send_discv_announce, DiscvFindReport,
    STUN_QUERY_TIMEOUT,
};
use anyhow::{bail, Context, Result};

use crate::bootstrap::ensure_bootstrap;
use crate::paths::DesktopPaths;
use crate::settings::load_or_create_settings;

/// Outcome of an explicit STUN Binding query (no public default server).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryStunOutcome {
    pub reflexive_addr: String,
    pub stun_server: String,
}

/// Run STUN Binding against an operator-supplied server (fail-closed if empty).
pub fn run_stun_query(paths: &DesktopPaths, stun_server: &str) -> Result<DiscoveryStunOutcome> {
    let trimmed = stun_server.trim();
    if trimmed.is_empty() {
        bail!("STUN server required — no public default; pass explicit host:port");
    }
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    let rec = query_and_save_stun_reflexive(&paths.data_root, trimmed, STUN_QUERY_TIMEOUT)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(DiscoveryStunOutcome {
        reflexive_addr: rec.addr,
        stun_server: rec.stun_server,
    })
}

/// Send one signed UDP discv announce to `to` advertising `advertised_addr`.
pub fn run_discv_announce(paths: &DesktopPaths, to: &str, advertised_addr: &str) -> Result<String> {
    let to_addr: SocketAddr = to
        .trim()
        .parse()
        .with_context(|| format!("invalid discv --to `{to}`"))?;
    let addr = advertised_addr.trim();
    if addr.is_empty() {
        bail!("discv announce requires explicit --addr (no public STUN default)");
    }
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    send_discv_announce(&paths.data_root, addr, to_addr).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(format!("discv announced {addr} -> {to_addr}"))
}

/// Iterative UDP FIND_NODE over discv (optional seed `to`).
pub fn run_discv_find(
    paths: &DesktopPaths,
    key_ref: &str,
    to: Option<&str>,
    k: u32,
) -> Result<DiscvFindReport> {
    let key = key_ref.trim();
    if key.is_empty() {
        bail!("FIND requires non-empty key_ref (aira:identity:…)");
    }
    let mut seeds = Vec::new();
    if let Some(t) = to {
        let t = t.trim();
        if !t.is_empty() {
            seeds.push(
                t.parse::<SocketAddr>()
                    .with_context(|| format!("invalid discv find --to `{t}`"))?,
            );
        }
    }
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    iterative_discv_find(&paths.data_root, key, &seeds, k.max(1) as usize)
        .map_err(|e| anyhow::anyhow!("{e}"))
}
