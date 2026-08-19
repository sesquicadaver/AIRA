//! Shared CLI helpers (Analyze-81). Mechanical extract from `main.rs`.

use std::path::{Path, PathBuf};

use aira_flow::node_config_present;
use aira_schema::{find_repo_root, SchemaRegistry};
use anyhow::{bail, Context, Result};

pub(crate) fn ensure_init(root: &Path) -> Result<()> {
    if !node_config_present(root) {
        bail!(
            "node not initialized at {} — run `aira init --root {}`",
            root.display(),
            root.display()
        );
    }
    Ok(())
}

pub(crate) fn require_trusted(root: &Path, key_ref: &str) -> Result<()> {
    let store = aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    if store.is_revoked(key_ref) {
        bail!("peer identity revoked: {key_ref}");
    }
    if !store.entries.iter().any(|e| e.identity_id == key_ref) {
        bail!("peer not trusted — run `aira identity trust add` first: {key_ref}");
    }
    Ok(())
}

pub(crate) fn build_peer_ping(root: &Path, text: &str) -> Result<aira_protocol::ProtocolEnvelope> {
    aira_peer::make_peer_ping(root, text).map_err(|e| anyhow::anyhow!("{e}"))
}
pub(crate) fn default_csu_registry(root: &Path) -> PathBuf {
    let modern = root.join("csu").join("registry.json");
    if modern.exists() {
        return modern;
    }
    // Legacy fallback from Epic 5 CLI.
    let legacy = PathBuf::from(".aira/csu-registry.json");
    if legacy.exists() {
        return legacy;
    }
    modern
}

pub(crate) fn load_schema_registry(schemas_dir: Option<PathBuf>) -> Result<SchemaRegistry> {
    let dir = if let Some(d) = schemas_dir {
        d
    } else {
        let root = find_repo_root(std::env::current_dir()?)
            .or_else(|_| find_repo_root(env!("CARGO_MANIFEST_DIR")).context("locate repo root"))?;
        root.join("schemas")
    };
    if !dir.is_dir() {
        bail!("schemas dir not found: {}", dir.display());
    }
    SchemaRegistry::load(dir)
}
