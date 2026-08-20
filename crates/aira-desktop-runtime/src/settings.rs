//! Desktop settings document (`aira:schema:desktop:settings:0.1`).

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::paths::DesktopPaths;

pub const SETTINGS_SCHEMA_ID: &str = "aira:schema:desktop:settings:0.1";

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

/// Load settings or create defaults on disk.
pub fn load_or_create_settings(paths: &DesktopPaths) -> Result<DesktopSettings> {
    paths.ensure_dirs().context("create desktop dirs")?;
    if paths.settings_file.is_file() {
        let text = fs::read_to_string(&paths.settings_file)
            .with_context(|| format!("read {}", paths.settings_file.display()))?;
        let s: DesktopSettings = serde_json::from_str(&text)
            .with_context(|| format!("parse {}", paths.settings_file.display()))?;
        if s.payload_schema != SETTINGS_SCHEMA_ID {
            bail!(
                "unsupported desktop settings schema {} (want {})",
                s.payload_schema,
                SETTINGS_SCHEMA_ID
            );
        }
        if s.network_profile != NetworkProfile::P0 {
            bail!(
                "E1 Desktop runtime supports network_profile=P0 only (got {:?})",
                s.network_profile
            );
        }
        return Ok(s);
    }
    let s = DesktopSettings::default_p0(paths);
    write_settings(paths, &s)?;
    Ok(s)
}

pub fn write_settings(paths: &DesktopPaths, settings: &DesktopSettings) -> Result<()> {
    if let Some(parent) = paths.settings_file.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(settings)?;
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
