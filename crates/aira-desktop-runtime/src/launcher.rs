//! Linux XDG `.desktop` launcher (QUEUE #77).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Canonical Desktop Entry shipped under `deploy/desktop/aira.desktop`.
pub const AIRA_DESKTOP_ENTRY: &str = include_str!("../../../deploy/desktop/aira.desktop");

/// Filename installed into applications directories.
pub const AIRA_DESKTOP_FILENAME: &str = "aira.desktop";

/// Validate required Freedesktop keys for the AIRA launcher.
pub fn validate_desktop_entry(text: &str) -> Result<()> {
    let mut has_type = false;
    let mut has_name = false;
    let mut has_exec = false;
    let mut in_main = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_main = line == "[Desktop Entry]";
            continue;
        }
        if !in_main {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "Type" => {
                    if v.trim() != "Application" {
                        bail!("Type must be Application");
                    }
                    has_type = true;
                }
                "Name" => {
                    if v.trim().is_empty() {
                        bail!("Name empty");
                    }
                    has_name = true;
                }
                "Exec" => {
                    let exec = v.trim();
                    if !exec.contains("aira desktop start") {
                        bail!("Exec must invoke `aira desktop start`, got: {exec}");
                    }
                    has_exec = true;
                }
                _ => {}
            }
        }
    }
    if !has_type || !has_name || !has_exec {
        bail!("desktop entry missing Type/Name/Exec in [Desktop Entry]");
    }
    Ok(())
}

/// Default user applications dir (`$XDG_DATA_HOME/applications` or `~/.local/share/applications`).
pub fn user_applications_dir() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("applications");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/share/applications")
}

/// Install `aira.desktop` into an applications directory.
pub fn install_launcher_to(applications_dir: &Path) -> Result<PathBuf> {
    validate_desktop_entry(AIRA_DESKTOP_ENTRY)?;
    fs::create_dir_all(applications_dir)
        .with_context(|| format!("mkdir {}", applications_dir.display()))?;
    let dest = applications_dir.join(AIRA_DESKTOP_FILENAME);
    fs::write(&dest, AIRA_DESKTOP_ENTRY).with_context(|| format!("write {}", dest.display()))?;
    Ok(dest)
}

/// Install `aira.desktop` into the user applications directory.
pub fn install_user_launcher() -> Result<PathBuf> {
    install_launcher_to(&user_applications_dir())
}

/// Remove a launcher file from an applications directory if present.
pub fn uninstall_launcher_from(applications_dir: &Path) -> Result<Option<PathBuf>> {
    let dest = applications_dir.join(AIRA_DESKTOP_FILENAME);
    if dest.is_file() {
        fs::remove_file(&dest).with_context(|| format!("remove {}", dest.display()))?;
        Ok(Some(dest))
    } else {
        Ok(None)
    }
}

/// Remove the user launcher if present.
pub fn uninstall_user_launcher() -> Result<Option<PathBuf>> {
    uninstall_launcher_from(&user_applications_dir())
}

/// Read and validate a desktop file from disk.
pub fn validate_desktop_file(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    validate_desktop_entry(&text)
}
