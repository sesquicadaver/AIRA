//! Desktop filesystem layout (phase-e §2.1).

use std::path::{Path, PathBuf};

/// Resolved Desktop / Dev paths.
#[derive(Debug, Clone)]
pub struct DesktopPaths {
    /// Node root (`init` / identity / sqlite / artifacts).
    pub data_root: PathBuf,
    /// `desktop-settings.json` location.
    pub settings_file: PathBuf,
    /// PID / lock / token / logs.
    pub runtime_dir: PathBuf,
    /// Bounded stdout/stderr of supervised `aira-node`.
    pub log_dir: PathBuf,
}

impl DesktopPaths {
    /// System Desktop layout (Linux XDG-style; other OS use home fallbacks).
    pub fn system() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let data_root = xdg_dir("XDG_DATA_HOME", home.join(".local/share")).join("aira");
        let config = xdg_dir("XDG_CONFIG_HOME", home.join(".config")).join("aira");
        let state = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| xdg_dir("XDG_STATE_HOME", home.join(".local/state")))
            .join("aira");
        let log_dir = xdg_dir("XDG_CACHE_HOME", home.join(".cache"))
            .join("aira")
            .join("logs");
        Self {
            data_root,
            settings_file: config.join("desktop-settings.json"),
            runtime_dir: state,
            log_dir,
        }
    }

    /// Dev / test layout colocated under an explicit root.
    pub fn for_data_root(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            data_root: root.clone(),
            settings_file: root.join("desktop-settings.json"),
            runtime_dir: root.join("runtime"),
            log_dir: root.join("logs"),
        }
    }

    pub fn pid_file(&self) -> PathBuf {
        self.runtime_dir.join("aira-node.pid.json")
    }

    pub fn lock_file(&self) -> PathBuf {
        self.runtime_dir.join("aira-node.lock")
    }

    pub fn token_file(&self) -> PathBuf {
        self.runtime_dir.join("http-token")
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.data_root)?;
        if let Some(p) = self.settings_file.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::create_dir_all(&self.runtime_dir)?;
        std::fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }
}

fn xdg_dir(var: &str, fallback: PathBuf) -> PathBuf {
    std::env::var_os(var).map(PathBuf::from).unwrap_or(fallback)
}
