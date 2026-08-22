//! Desktop federation join (QUEUE #103) — descriptor file → local pin + membership read.

use std::path::Path;

use aira_protocol::{
    join_federation, load_federation_membership, FederationDescriptor, FederationMembership,
    JoinOutcome,
};
use anyhow::{Context, Result};

use crate::bootstrap::ensure_bootstrap;
use crate::paths::DesktopPaths;
use crate::settings::load_or_create_settings;

/// Join federation from a JSON descriptor file (same semantics as `aira federation join`).
pub fn join_federation_descriptor_file(
    paths: &DesktopPaths,
    descriptor_path: &Path,
) -> Result<JoinOutcome> {
    paths.ensure_dirs().context("desktop dirs")?;
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    let raw = std::fs::read_to_string(descriptor_path)
        .with_context(|| format!("read {}", descriptor_path.display()))?;
    let desc: FederationDescriptor = serde_json::from_str(&raw)
        .with_context(|| format!("parse federation descriptor {}", descriptor_path.display()))?;
    join_federation(&paths.data_root, &desc).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Read persisted federation membership for this Desktop data root.
pub fn read_federation_membership(paths: &DesktopPaths) -> Result<Option<FederationMembership>> {
    load_federation_membership(&paths.data_root).map_err(|e| anyhow::anyhow!("{e}"))
}
