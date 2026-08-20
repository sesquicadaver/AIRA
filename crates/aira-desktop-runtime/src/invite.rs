//! PeerInvite file export/import (QUEUE #83 / Analyze-118).
//!
//! Export local identity as `aira:schema:desktop:peer-invite:0.1`.
//! Import → trust upsert, then optional address-book upsert.

use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use aira_flow::NodePaths;
use aira_object::{register_trust_store, TrustStore};
use aira_peer::AddressBook;

use crate::bootstrap::ensure_bootstrap;
use crate::paths::DesktopPaths;
use crate::settings::{load_or_create_settings, DesktopSettings, NetworkProfile};

pub const PEER_INVITE_SCHEMA_ID: &str = "aira:schema:desktop:peer-invite:0.1";

/// Desktop PeerInvite document (`aira:schema:desktop:peer-invite:0.1`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerInvite {
    pub payload_schema: String,
    pub identity_ref: String,
    pub public_key_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Result of applying an invite into a node root.
#[derive(Debug, Clone)]
pub struct ImportInviteOutcome {
    pub identity_ref: String,
    pub trusted: bool,
    pub book_updated: bool,
    pub addr: Option<String>,
}

/// Validate invite shape (schema contract without pulling jsonschema into runtime).
pub fn validate_peer_invite(invite: &PeerInvite) -> Result<()> {
    if invite.payload_schema != PEER_INVITE_SCHEMA_ID {
        bail!(
            "unsupported peer invite schema {} (want {})",
            invite.payload_schema,
            PEER_INVITE_SCHEMA_ID
        );
    }
    let _ = aira_object::AiraRef::parse(&invite.identity_ref)
        .map_err(|e| anyhow::anyhow!("identity_ref: {e}"))?;
    let pk = invite.public_key_hex.trim();
    if pk.len() != 64 || !pk.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("public_key_hex must be 64 hex chars");
    }
    if let Some(addr) = invite.addr.as_deref() {
        let addr = addr.trim();
        if addr.is_empty() {
            bail!("addr empty");
        }
        addr.parse::<SocketAddr>()
            .with_context(|| format!("invalid invite addr `{addr}`"))?;
    }
    Ok(())
}

/// Build invite from the Desktop node identity (+ optional dial addr).
pub fn build_local_invite(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
    addr_override: Option<String>,
) -> Result<PeerInvite> {
    let np = NodePaths::new(&paths.data_root);
    let text = fs::read_to_string(np.identity_json())
        .with_context(|| format!("read {}", np.identity_json().display()))?;
    let desc: serde_json::Value =
        serde_json::from_str(&text).context("parse local.identity.json")?;
    let identity_ref = desc
        .get("identity_id")
        .and_then(|v| v.as_str())
        .context("identity_id missing")?
        .to_string();
    let public_key_hex = desc
        .pointer("/public_key/key_hex")
        .and_then(|v| v.as_str())
        .context("public_key.key_hex missing")?
        .trim()
        .to_string();
    let display_name = desc
        .get("display_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let addr = match addr_override {
        Some(a) => Some(a),
        None => match settings.network_profile {
            NetworkProfile::P1 => settings.peer_listen.clone(),
            _ => None,
        },
    };

    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());

    let invite = PeerInvite {
        payload_schema: PEER_INVITE_SCHEMA_ID.to_string(),
        identity_ref,
        public_key_hex,
        addr,
        display_name,
        created_at: Some(created_at),
    };
    validate_peer_invite(&invite)?;
    Ok(invite)
}

/// Export invite JSON to `out_path` (pretty + trailing newline).
pub fn export_invite_file(
    paths: &DesktopPaths,
    out_path: &Path,
    addr_override: Option<String>,
) -> Result<PeerInvite> {
    paths.ensure_dirs()?;
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    let invite = build_local_invite(paths, &settings, addr_override)?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&invite)?;
    fs::write(out_path, format!("{text}\n"))
        .with_context(|| format!("write {}", out_path.display()))?;
    Ok(invite)
}

/// Load invite JSON from disk and validate.
pub fn load_invite_file(path: &Path) -> Result<PeerInvite> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let invite: PeerInvite =
        serde_json::from_str(&text).with_context(|| format!("parse invite {}", path.display()))?;
    validate_peer_invite(&invite)?;
    Ok(invite)
}

/// Import invite: trust add, then address-book upsert when `addr` is set.
pub fn import_invite(paths: &DesktopPaths, invite: &PeerInvite) -> Result<ImportInviteOutcome> {
    validate_peer_invite(invite)?;
    paths.ensure_dirs()?;
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;

    let root = &paths.data_root;
    let mut store = TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    store
        .upsert(&invite.identity_ref, invite.public_key_hex.trim())
        .map_err(|e| anyhow::anyhow!("trust upsert: {e}"))?;
    store.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    register_trust_store(root).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut book_updated = false;
    let addr = invite.addr.as_ref().map(|a| a.trim().to_string());
    if let Some(ref a) = addr {
        let mut book = AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
        book.upsert(&invite.identity_ref, a);
        book.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
        book_updated = true;
    }

    Ok(ImportInviteOutcome {
        identity_ref: invite.identity_ref.clone(),
        trusted: true,
        book_updated,
        addr,
    })
}

/// Import from a JSON file path.
pub fn import_invite_file(paths: &DesktopPaths, path: &Path) -> Result<ImportInviteOutcome> {
    let invite = load_invite_file(path)?;
    import_invite(paths, &invite)
}
