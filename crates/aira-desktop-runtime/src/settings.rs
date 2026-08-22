//! Desktop settings document (`aira:schema:desktop:settings:0.1`).
//!
//! E1.1 (`#81`): `network_profile` may be P0 or P1.
//! E4 (`#94`): P2 accepted with the same `peer_listen` rules as P1. P3+ fail-closed.

use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::paths::DesktopPaths;

pub const SETTINGS_SCHEMA_ID: &str = "aira:schema:desktop:settings:0.1";

/// Default peer listen for P1 (phase-e §4a; same-host Developer Preview).
pub const DEFAULT_PEER_LISTEN: &str = "127.0.0.1:9797";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NetworkProfile {
    P0,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
}

impl NetworkProfile {
    /// Profiles the Desktop runtime may load/persist (E4 `#94`: through P2).
    pub fn is_supported(self) -> bool {
        matches!(self, Self::P0 | Self::P1 | Self::P2)
    }

    /// P1+ profiles that require validated `peer_listen` after normalize.
    pub fn requires_peer_listen(self) -> bool {
        matches!(self, Self::P1 | Self::P2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpAuthMode {
    BearerToken,
    DesktopIpc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub payload_schema: String,
    pub network_profile: NetworkProfile,
    pub open_ui_on_start: bool,
    pub autostart_on_login: bool,
    pub http_listen: String,
    pub instance_id: String,
    pub http_auth_mode: HttpAuthMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_token_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_listen: Option<String>,
}

impl DesktopSettings {
    /// Default P0 Desktop settings (bearer token contract).
    pub fn default_p0(paths: &DesktopPaths) -> Self {
        let instance_id = format!("aira:instance:{}", Uuid::now_v7());
        Self {
            payload_schema: SETTINGS_SCHEMA_ID.to_string(),
            network_profile: NetworkProfile::P0,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".to_string(),
            instance_id,
            http_auth_mode: HttpAuthMode::BearerToken,
            http_token_ref: Some(paths.token_file().display().to_string()),
            peer_listen: None,
        }
    }
}

/// Parse `host:port` (IPv4 / IPv6 / hostname forms accepted by `SocketAddr`).
pub fn validate_listen_addr(listen: &str) -> Result<SocketAddr> {
    let listen = listen.trim();
    if listen.is_empty() {
        bail!("listen address empty");
    }
    listen
        .parse::<SocketAddr>()
        .with_context(|| format!("invalid listen address `{listen}` (want host:port)"))
}

/// Fail-closed profile + listen validation; fill P1/P2 `peer_listen` default when missing.
pub fn normalize_settings(settings: &mut DesktopSettings) -> Result<()> {
    if settings.payload_schema != SETTINGS_SCHEMA_ID {
        bail!(
            "unsupported desktop settings schema {} (want {})",
            settings.payload_schema,
            SETTINGS_SCHEMA_ID
        );
    }
    if !settings.network_profile.is_supported() {
        bail!(
            "Desktop runtime supports network_profile=P0|P1|P2 only (got {:?}; P3+ Out of E4)",
            settings.network_profile
        );
    }
    validate_listen_addr(&settings.http_listen).context("http_listen")?;
    match settings.network_profile {
        NetworkProfile::P0 => Ok(()),
        NetworkProfile::P1 | NetworkProfile::P2 => {
            let listen = match settings.peer_listen.as_deref().map(str::trim) {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => DEFAULT_PEER_LISTEN.to_string(),
            };
            validate_listen_addr(&listen).context("peer_listen")?;
            settings.peer_listen = Some(listen);
            Ok(())
        }
        NetworkProfile::P3
        | NetworkProfile::P4
        | NetworkProfile::P5
        | NetworkProfile::P6 => unreachable!("is_supported already rejected"),
    }
}

/// Effective peer listen for P1/P2 (after normalize), or `None` on P0.
pub fn effective_peer_listen(settings: &DesktopSettings) -> Option<&str> {
    if settings.network_profile.requires_peer_listen() {
        settings.peer_listen.as_deref()
    } else {
        None
    }
}

/// Load settings or create defaults on disk.
pub fn load_or_create_settings(paths: &DesktopPaths) -> Result<DesktopSettings> {
    paths.ensure_dirs().context("create desktop dirs")?;
    if paths.settings_file.is_file() {
        let text = fs::read_to_string(&paths.settings_file)
            .with_context(|| format!("read {}", paths.settings_file.display()))?;
        let mut s: DesktopSettings = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", paths.settings_file.display()))?;
        let peer_missing = s.network_profile.requires_peer_listen()
            && s.peer_listen
                .as_deref()
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .is_none();
        normalize_settings(&mut s)?;
        if peer_missing {
            write_settings(paths, &s)?;
        }
        return Ok(s);
    }
    let s = DesktopSettings::default_p0(paths);
    write_settings(paths, &s)?;
    Ok(s)
}

pub fn write_settings(paths: &DesktopPaths, settings: &DesktopSettings) -> Result<()> {
    let mut settings = settings.clone();
    normalize_settings(&mut settings)?;
    if let Some(parent) = paths.settings_file.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&settings)?;
    fs::write(&paths.settings_file, format!("{text}\n"))
        .with_context(|| format!("write {}", paths.settings_file.display()))?;
    Ok(())
}

pub fn resolve_token_path(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
) -> Result<std::path::PathBuf> {
    match settings.http_auth_mode {
        HttpAuthMode::BearerToken => {
            if let Some(ref r) = settings.http_token_ref {
                let p = Path::new(r);
                if p.is_absolute() {
                    Ok(p.to_path_buf())
                } else {
                    Ok(paths.data_root.join(p))
                }
            } else {
                Ok(paths.token_file())
            }
        }
        HttpAuthMode::DesktopIpc => {
            bail!("http_auth_mode=desktop_ipc is reserved; #76 implements bearer_token only")
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn p1_fills_default_peer_listen() {
        let mut s = DesktopSettings {
            payload_schema: SETTINGS_SCHEMA_ID.into(),
            network_profile: NetworkProfile::P1,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".into(),
            instance_id: "aira:instance:test".into(),
            http_auth_mode: HttpAuthMode::BearerToken,
            http_token_ref: None,
            peer_listen: None,
        };
        normalize_settings(&mut s).unwrap();
        assert_eq!(s.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
    }

    #[test]
    fn p2_fills_default_peer_listen() {
        let mut s = DesktopSettings {
            payload_schema: SETTINGS_SCHEMA_ID.into(),
            network_profile: NetworkProfile::P2,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".into(),
            instance_id: "aira:instance:test".into(),
            http_auth_mode: HttpAuthMode::BearerToken,
            http_token_ref: None,
            peer_listen: None,
        };
        normalize_settings(&mut s).unwrap();
        assert_eq!(s.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
    }

    #[test]
    fn p3_rejected() {
        let mut s = DesktopSettings {
            payload_schema: SETTINGS_SCHEMA_ID.into(),
            network_profile: NetworkProfile::P3,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".into(),
            instance_id: "aira:instance:test".into(),
            http_auth_mode: HttpAuthMode::BearerToken,
            http_token_ref: None,
            peer_listen: Some(DEFAULT_PEER_LISTEN.into()),
        };
        let err = normalize_settings(&mut s).unwrap_err().to_string();
        assert!(err.contains("P0|P1|P2"), "{err}");
    }

    #[test]
    fn invalid_peer_listen_rejected() {
        let mut s = DesktopSettings {
            payload_schema: SETTINGS_SCHEMA_ID.into(),
            network_profile: NetworkProfile::P1,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".into(),
            instance_id: "aira:instance:test".into(),
            http_auth_mode: HttpAuthMode::BearerToken,
            http_token_ref: None,
            peer_listen: Some("not-an-addr".into()),
        };
        assert!(normalize_settings(&mut s).is_err());
    }
}
