//! Linux XDG autostart hooks (QUEUE #78 / phase-e §2.5).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Filename under `autostart/` for login start of the Desktop GUI.
pub const AIRA_AUTOSTART_FILENAME: &str = "aira-desktop.desktop";

/// Body of the XDG autostart entry (`Exec=aira-desktop`).
pub fn autostart_desktop_entry() -> String {
    [
        "[Desktop Entry]",
        "Type=Application",
        "Version=1.0",
        "Name=AIRA Desktop",
        "Comment=AIRA Desktop (Developer Preview) — autostart local P0 node + UI",
        "Exec=aira-desktop",
        "TryExec=aira-desktop",
        "Terminal=false",
        "X-GNOME-Autostart-enabled=true",
        "Categories=Utility;Development;",
        "",
    ]
    .join("\n")
}

/// `$XDG_CONFIG_HOME/autostart` or `~/.config/autostart`.
pub fn user_autostart_dir() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("autostart");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config/autostart")
}

pub fn autostart_path_in(dir: &Path) -> PathBuf {
    dir.join(AIRA_AUTOSTART_FILENAME)
}

pub fn is_autostart_enabled_in(dir: &Path) -> bool {
    autostart_path_in(dir).is_file()
}

pub fn is_autostart_enabled() -> bool {
    is_autostart_enabled_in(&user_autostart_dir())
}

/// Write or remove the autostart `.desktop` to match `enabled`.
pub fn set_autostart_in(dir: &Path, enabled: bool) -> Result<PathBuf> {
    let path = autostart_path_in(dir);
    if enabled {
        fs::create_dir_all(dir).with_context(|| format!("mkdir {}", dir.display()))?;
        fs::write(&path, autostart_desktop_entry())
            .with_context(|| format!("write {}", path.display()))?;
    } else if path.is_file() {
        fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(path)
}

pub fn set_autostart(enabled: bool) -> Result<PathBuf> {
    set_autostart_in(&user_autostart_dir(), enabled)
}

/// Apply `settings.autostart_on_login` to the OS hook (idempotent).
pub fn sync_autostart_from_settings(autostart_on_login: bool) -> Result<PathBuf> {
    set_autostart(autostart_on_login)
}
