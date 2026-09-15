//! Desktop settings document (`aira:schema:desktop:settings:0.1`).
//!
//! E1.1 (`#81`): `network_profile` may be P0 or P1.
//! E4 (`#94`): P2 accepted with the same `peer_listen` rules as P1.
//! E4 (`#97`): P3 + `relay_ttl_days` (default 31).
//! E4 (`#100`): P4 gossip profile. P5+ fail-closed; P3|P4 mutex by profile enum.
//! Ollama bind: optional `llm_backend` / `llm_process_bin` / `llm_ollama_model`
//! (process executor ≠ mock; never grants VERIFIED).

use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ollama::DEFAULT_OLLAMA_BIN;
use crate::paths::DesktopPaths;

pub const SETTINGS_SCHEMA_ID: &str = "aira:schema:desktop:settings:0.1";

/// Default peer listen for P1+ (Phase N Prime Port; same-host Developer Preview).
pub const DEFAULT_PEER_LISTEN: &str = "127.0.0.1:49157";

/// Default relay hub offline registry TTL for P3 (phase-e §4d).
pub const DEFAULT_RELAY_TTL_DAYS: u32 = 31;

/// Staff submit executor for spawned `aira-node` (`AIRA_LLM_BACKEND`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmBackend {
    /// Reference MockBackend (CI / Developer Preview default).
    #[default]
    Mock,
    /// Host CLI process backend (ollama / llama.cpp-style argv).
    Process,
}

impl LlmBackend {
    /// Env token for `AIRA_LLM_BACKEND`.
    pub fn as_env_str(self) -> &'static str {
        match self {
            Self::Mock => "mock",
            Self::Process => "process",
        }
    }
}

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
    /// Profiles the Desktop runtime may load/persist (E4 `#100`: through P4).
    pub fn is_supported(self) -> bool {
        matches!(self, Self::P0 | Self::P1 | Self::P2 | Self::P3 | Self::P4)
    }

    /// P1–P4 profiles that require validated `peer_listen` after normalize.
    pub fn requires_peer_listen(self) -> bool {
        matches!(self, Self::P1 | Self::P2 | Self::P3 | Self::P4)
    }

    /// Relay hub profile (mutex with P4 gossip at settings level).
    pub fn is_relay_profile(self) -> bool {
        matches!(self, Self::P3)
    }

    /// Gossip profile (mutex with P3 relay at settings level).
    pub fn is_gossip_profile(self) -> bool {
        matches!(self, Self::P4)
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
    /// Relay hub offline registry TTL days; used when `network_profile=P3`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay_ttl_days: Option<u32>,
    /// Staff submit executor for `aira-node` (default mock).
    #[serde(default)]
    pub llm_backend: LlmBackend,
    /// Process program; default `ollama` when backend is process and unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_process_bin: Option<String>,
    /// Host ollama model name (`ollama list`); required when `llm_backend=process`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_ollama_model: Option<String>,
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
            relay_ttl_days: None,
            llm_backend: LlmBackend::Mock,
            llm_process_bin: None,
            llm_ollama_model: None,
        }
    }

    /// Effective process binary for ollama bind (settings or default).
    pub fn effective_llm_process_bin(&self) -> &str {
        self.llm_process_bin
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_OLLAMA_BIN)
    }

    /// Process argv token string for `AIRA_LLM_PROCESS_ARGS` (`run <model>`).
    pub fn llm_process_args(&self) -> Option<String> {
        let model = self
            .llm_ollama_model
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())?;
        Some(format!("run {model}"))
    }
}

/// Env keys honored by `execution-llm` / staff `with_backend_from_env` (RFC-0110).
const ENV_LLM_BACKEND: &str = "AIRA_LLM_BACKEND";
const ENV_PROCESS_BIN: &str = "AIRA_LLM_PROCESS_BIN";
const ENV_PROCESS_ARGS: &str = "AIRA_LLM_PROCESS_ARGS";
const ENV_PROCESS_TIMEOUT_MS: &str = "AIRA_LLM_PROCESS_TIMEOUT_MS";

/// Apply LLM env for a spawned `aira-node` so staff submit follows Settings.
///
/// Explicitly sets / clears vars so a parent shell cannot leak a stale process
/// bind into a mock node (or the reverse). Never claims VERIFIED.
pub fn apply_node_llm_env(cmd: &mut Command, settings: &DesktopSettings) {
    // Clear inherited process knobs first.
    cmd.env_remove(ENV_LLM_BACKEND);
    cmd.env_remove(ENV_PROCESS_BIN);
    cmd.env_remove(ENV_PROCESS_ARGS);
    cmd.env_remove(ENV_PROCESS_TIMEOUT_MS);

    match settings.llm_backend {
        LlmBackend::Mock => {
            cmd.env(ENV_LLM_BACKEND, "mock");
        }
        LlmBackend::Process => {
            cmd.env(ENV_LLM_BACKEND, "process");
            cmd.env(ENV_PROCESS_BIN, settings.effective_llm_process_bin());
            if let Some(args) = settings.llm_process_args() {
                cmd.env(ENV_PROCESS_ARGS, args);
            }
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

/// Fail-closed profile + listen validation; fill defaults for P1–P4.
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
            "Desktop runtime supports network_profile=P0|P1|P2|P3|P4 only (got {:?}; P5+ Out of E4)",
            settings.network_profile
        );
    }
    if settings.network_profile.is_relay_profile() && settings.network_profile.is_gossip_profile() {
        bail!("network_profile cannot combine P3 relay and P4 gossip");
    }
    validate_listen_addr(&settings.http_listen).context("http_listen")?;
    normalize_llm_settings(settings)?;
    match settings.network_profile {
        NetworkProfile::P0 => {
            settings.relay_ttl_days = None;
            Ok(())
        }
        NetworkProfile::P1 | NetworkProfile::P2 => {
            normalize_peer_listen(settings)?;
            settings.relay_ttl_days = None;
            Ok(())
        }
        NetworkProfile::P3 => {
            normalize_peer_listen(settings)?;
            let ttl = settings.relay_ttl_days.unwrap_or(DEFAULT_RELAY_TTL_DAYS);
            if ttl == 0 {
                bail!("relay_ttl_days must be >= 1");
            }
            settings.relay_ttl_days = Some(ttl);
            Ok(())
        }
        NetworkProfile::P4 => {
            normalize_peer_listen(settings)?;
            settings.relay_ttl_days = None;
            Ok(())
        }
        NetworkProfile::P5 | NetworkProfile::P6 => {
            unreachable!("is_supported already rejected")
        }
    }
}

fn normalize_llm_settings(settings: &mut DesktopSettings) -> Result<()> {
    if let Some(bin) = settings.llm_process_bin.as_mut() {
        let t = bin.trim().to_string();
        if t.is_empty() {
            settings.llm_process_bin = None;
        } else {
            *bin = t;
        }
    }
    if let Some(model) = settings.llm_ollama_model.as_mut() {
        let t = model.trim().to_string();
        if t.is_empty() {
            settings.llm_ollama_model = None;
        } else {
            *model = t;
        }
    }
    match settings.llm_backend {
        LlmBackend::Mock => Ok(()),
        LlmBackend::Process => {
            if settings.llm_ollama_model.is_none() {
                bail!(
                    "llm_backend=process requires llm_ollama_model (host name from `ollama list`)"
                );
            }
            Ok(())
        }
    }
}

fn normalize_peer_listen(settings: &mut DesktopSettings) -> Result<()> {
    let listen = match settings.peer_listen.as_deref().map(str::trim) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => DEFAULT_PEER_LISTEN.to_string(),
    };
    validate_listen_addr(&listen).context("peer_listen")?;
    aira_peer::validate_aira_bind(&listen)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("peer_listen")?;
    settings.peer_listen = Some(listen);
    Ok(())
}

/// Effective relay TTL days for P3 (after normalize), or `None` off P3.
pub fn effective_relay_ttl_days(settings: &DesktopSettings) -> Option<u32> {
    if settings.network_profile.is_relay_profile() {
        settings.relay_ttl_days
    } else {
        None
    }
}

/// Effective peer listen for P1–P4 (after normalize), or `None` on P0.
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
        let relay_ttl_missing = s.network_profile.is_relay_profile() && s.relay_ttl_days.is_none();
        normalize_settings(&mut s)?;
        if peer_missing || relay_ttl_missing {
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

    fn base(profile: NetworkProfile) -> DesktopSettings {
        DesktopSettings {
            payload_schema: SETTINGS_SCHEMA_ID.into(),
            network_profile: profile,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".into(),
            instance_id: "aira:instance:test".into(),
            http_auth_mode: HttpAuthMode::BearerToken,
            http_token_ref: None,
            peer_listen: None,
            relay_ttl_days: None,
            llm_backend: LlmBackend::Mock,
            llm_process_bin: None,
            llm_ollama_model: None,
        }
    }

    #[test]
    fn p1_fills_default_peer_listen() {
        let mut s = base(NetworkProfile::P1);
        normalize_settings(&mut s).unwrap();
        assert_eq!(s.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
    }

    #[test]
    fn p2_fills_default_peer_listen() {
        let mut s = base(NetworkProfile::P2);
        normalize_settings(&mut s).unwrap();
        assert_eq!(s.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
    }

    #[test]
    fn p3_fills_default_relay_ttl() {
        let mut s = base(NetworkProfile::P3);
        s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
        normalize_settings(&mut s).unwrap();
        assert_eq!(s.relay_ttl_days, Some(DEFAULT_RELAY_TTL_DAYS));
    }

    #[test]
    fn p5_rejected() {
        let mut s = base(NetworkProfile::P5);
        s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
        let err = normalize_settings(&mut s).unwrap_err().to_string();
        assert!(err.contains("P0|P1|P2|P3|P4"), "{err}");
    }

    #[test]
    fn p4_clears_relay_ttl() {
        let mut s = base(NetworkProfile::P4);
        s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
        s.relay_ttl_days = Some(31);
        normalize_settings(&mut s).unwrap();
        assert!(s.relay_ttl_days.is_none());
    }

    #[test]
    fn p1_clears_relay_ttl() {
        let mut s = base(NetworkProfile::P1);
        s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
        s.relay_ttl_days = Some(31);
        normalize_settings(&mut s).unwrap();
        assert!(s.relay_ttl_days.is_none());
    }

    #[test]
    fn invalid_peer_listen_rejected() {
        let mut s = base(NetworkProfile::P1);
        s.peer_listen = Some("not-an-addr".into());
        assert!(normalize_settings(&mut s).is_err());
    }

    #[test]
    fn process_without_model_rejected() {
        let mut s = base(NetworkProfile::P0);
        s.llm_backend = LlmBackend::Process;
        let err = normalize_settings(&mut s).unwrap_err().to_string();
        assert!(err.contains("llm_ollama_model"), "{err}");
    }

    #[test]
    fn process_with_model_ok_and_args() {
        let mut s = base(NetworkProfile::P0);
        s.llm_backend = LlmBackend::Process;
        s.llm_ollama_model = Some("  llama3:latest  ".into());
        normalize_settings(&mut s).unwrap();
        assert_eq!(s.llm_ollama_model.as_deref(), Some("llama3:latest"));
        assert_eq!(s.llm_process_args().as_deref(), Some("run llama3:latest"));
        assert_eq!(s.effective_llm_process_bin(), DEFAULT_OLLAMA_BIN);
    }

    #[test]
    fn apply_node_llm_env_sets_process() {
        let mut s = base(NetworkProfile::P0);
        s.llm_backend = LlmBackend::Process;
        s.llm_ollama_model = Some("llama3:latest".into());
        normalize_settings(&mut s).unwrap();
        let mut cmd = Command::new("true");
        apply_node_llm_env(&mut cmd, &s);
        // Command debug formatting includes env for inspection on Unix.
        let dbg = format!("{cmd:?}");
        assert!(dbg.contains("AIRA_LLM_BACKEND") || dbg.contains("process"), "{dbg}");
    }
}
