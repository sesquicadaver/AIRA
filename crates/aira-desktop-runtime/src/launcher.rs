//! Linux XDG `.desktop` launcher entries (QUEUE #77 / #79).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Canonical start/stop Desktop Entry shipped under `deploy/desktop/aira.desktop`.
pub const AIRA_DESKTOP_ENTRY: &str = include_str!("../../../deploy/desktop/aira.desktop");

/// Canonical GUI Desktop Entry shipped under `deploy/desktop/aira-desktop.desktop`.
pub const AIRA_GUI_DESKTOP_ENTRY: &str =
    include_str!("../../../deploy/desktop/aira-desktop.desktop");

/// Filename installed into applications directories (start/stop menu entry).
pub const AIRA_DESKTOP_FILENAME: &str = "aira.desktop";

/// Filename for the native GUI menu entry.
pub const AIRA_GUI_DESKTOP_FILENAME: &str = "aira-desktop.desktop";

/// Validate required Freedesktop keys for the AIRA start launcher.
pub fn validate_desktop_entry(text: &str) -> Result<()> {
    validate_named_exec(text, "aira desktop start")
}

/// Validate required Freedesktop keys for the AIRA GUI launcher.
pub fn validate_gui_desktop_entry(text: &str) -> Result<()> {
    validate_named_exec(text, "aira-desktop")
}

fn validate_named_exec(text: &str, exec_must_contain: &str) -> Result<()> {
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
                    if !exec.contains(exec_must_contain) {
                        bail!("Exec must contain `{exec_must_contain}`, got: {exec}");
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

fn write_entry(applications_dir: &Path, filename: &str, body: &str) -> Result<PathBuf> {
    fs::create_dir_all(applications_dir)
        .with_context(|| format!("mkdir {}", applications_dir.display()))?;
    let dest = applications_dir.join(filename);
    fs::write(&dest, body).with_context(|| format!("write {}", dest.display()))?;
    Ok(dest)
}

/// Install `aira.desktop` into an applications directory.
pub fn install_launcher_to(applications_dir: &Path) -> Result<PathBuf> {
    validate_desktop_entry(AIRA_DESKTOP_ENTRY)?;
    write_entry(applications_dir, AIRA_DESKTOP_FILENAME, AIRA_DESKTOP_ENTRY)
}

/// Install `aira-desktop.desktop` into an applications directory.
pub fn install_gui_launcher_to(applications_dir: &Path) -> Result<PathBuf> {
    validate_gui_desktop_entry(AIRA_GUI_DESKTOP_ENTRY)?;
    write_entry(
        applications_dir,
        AIRA_GUI_DESKTOP_FILENAME,
        AIRA_GUI_DESKTOP_ENTRY,
    )
}

/// Install both menu entries into an applications directory.
pub fn install_menu_entries_to(applications_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let start = install_launcher_to(applications_dir)?;
    let gui = install_gui_launcher_to(applications_dir)?;
    Ok((start, gui))
}

/// Install `aira.desktop` into the user applications directory.
pub fn install_user_launcher() -> Result<PathBuf> {
    install_launcher_to(&user_applications_dir())
}

/// Install both menu entries into the user applications directory.
pub fn install_user_menu_entries() -> Result<(PathBuf, PathBuf)> {
    install_menu_entries_to(&user_applications_dir())
}

/// Remove a launcher file from an applications directory if present.
pub fn uninstall_launcher_from(applications_dir: &Path) -> Result<Option<PathBuf>> {
    remove_entry(applications_dir, AIRA_DESKTOP_FILENAME)
}

/// Remove the GUI menu entry if present.
pub fn uninstall_gui_launcher_from(applications_dir: &Path) -> Result<Option<PathBuf>> {
    remove_entry(applications_dir, AIRA_GUI_DESKTOP_FILENAME)
}

/// Remove both menu entries if present.
pub fn uninstall_menu_entries_from(applications_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut removed = Vec::new();
    if let Some(p) = uninstall_launcher_from(applications_dir)? {
        removed.push(p);
    }
    if let Some(p) = uninstall_gui_launcher_from(applications_dir)? {
        removed.push(p);
    }
    Ok(removed)
}

fn remove_entry(applications_dir: &Path, filename: &str) -> Result<Option<PathBuf>> {
    let dest = applications_dir.join(filename);
    if dest.is_file() {
        fs::remove_file(&dest).with_context(|| format!("remove {}", dest.display()))?;
        Ok(Some(dest))
    } else {
        Ok(None)
    }
}

/// Remove the user start launcher if present.
pub fn uninstall_user_launcher() -> Result<Option<PathBuf>> {
    uninstall_launcher_from(&user_applications_dir())
}

/// Remove both user menu entries if present.
pub fn uninstall_user_menu_entries() -> Result<Vec<PathBuf>> {
    uninstall_menu_entries_from(&user_applications_dir())
}

/// Read and validate a start desktop file from disk.
pub fn validate_desktop_file(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    validate_desktop_entry(&text)
}

/// Read and validate a GUI desktop file from disk.
pub fn validate_gui_desktop_file(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    validate_gui_desktop_entry(&text)
}
