//! OS autostart hooks (QUEUE #78 Linux XDG; QUEUE #87 macOS LaunchAgent).
//!
//! `sync_autostart_from_settings` / `set_autostart` dispatch by target OS.
//! Linux XDG `.desktop` helpers remain available on all OS for tests/packaging.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// Filename under XDG `autostart/` for login start of the Desktop GUI (Linux).
pub const AIRA_AUTOSTART_FILENAME: &str = "aira-desktop.desktop";

/// LaunchAgent label / plist basename (macOS).
pub const AIRA_LAUNCH_AGENT_LABEL: &str = "ai.aira.desktop";

/// Filename under `~/Library/LaunchAgents/`.
pub const AIRA_LAUNCH_AGENT_FILENAME: &str = "ai.aira.desktop.plist";

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

/// LaunchAgent plist body. `program` should be an absolute path when possible.
pub fn launch_agent_plist(program: &str) -> String {
    let program = xml_escape(program.trim());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{label}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{program}</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>KeepAlive</key>
	<false/>
</dict>
</plist>
"#,
        label = AIRA_LAUNCH_AGENT_LABEL,
        program = program,
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

/// `~/Library/LaunchAgents` (macOS).
pub fn user_launch_agents_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Library/LaunchAgents")
}

/// LaunchAgents dir under an explicit home (tests).
pub fn launch_agents_dir_for_home(home: &Path) -> PathBuf {
    home.join("Library/LaunchAgents")
}

pub fn autostart_path_in(dir: &Path) -> PathBuf {
    dir.join(AIRA_AUTOSTART_FILENAME)
}

pub fn launch_agent_path_in(dir: &Path) -> PathBuf {
    dir.join(AIRA_LAUNCH_AGENT_FILENAME)
}

pub fn is_autostart_enabled_in(dir: &Path) -> bool {
    autostart_path_in(dir).is_file()
}

pub fn is_launch_agent_enabled_in(dir: &Path) -> bool {
    launch_agent_path_in(dir).is_file()
}

/// Whether the OS-native autostart hook is present for this build.
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "macos")]
    {
        is_launch_agent_enabled_in(&user_launch_agents_dir())
    }
    #[cfg(not(target_os = "macos"))]
    {
        is_autostart_enabled_in(&user_autostart_dir())
    }
}

/// Write or remove the Linux XDG autostart `.desktop` to match `enabled`.
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

/// Write or remove a LaunchAgent plist under `dir` (usually `~/Library/LaunchAgents`).
pub fn set_launch_agent_in(dir: &Path, enabled: bool, program: &str) -> Result<PathBuf> {
    let path = launch_agent_path_in(dir);
    if enabled {
        fs::create_dir_all(dir).with_context(|| format!("mkdir {}", dir.display()))?;
        let body = launch_agent_plist(program);
        fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    } else if path.is_file() {
        fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(path)
}

/// Resolve `aira-desktop` for LaunchAgent `ProgramArguments` (absolute when found).
pub fn resolve_desktop_program() -> String {
    if let Ok(p) = std::env::var("AIRA_DESKTOP_BIN") {
        let t = p.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("aira-desktop");
            if cand.is_file() {
                return cand.display().to_string();
            }
        }
    }
    if let Ok(out) = Command::new("which").arg("aira-desktop").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    // PATH-style fallback (Developer Preview; same spirit as Linux Exec=aira-desktop).
    "aira-desktop".to_string()
}

/// Enable/disable the OS-native autostart hook for this build.
pub fn set_autostart(enabled: bool) -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        set_launch_agent_in(
            &user_launch_agents_dir(),
            enabled,
            &resolve_desktop_program(),
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        set_autostart_in(&user_autostart_dir(), enabled)
    }
}

/// Apply `settings.autostart_on_login` to the OS hook (idempotent).
pub fn sync_autostart_from_settings(autostart_on_login: bool) -> Result<PathBuf> {
    set_autostart(autostart_on_login)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_agent_plist_contains_label_and_program() {
        let body = launch_agent_plist("/Users/dev/bin/aira-desktop");
        assert!(body.contains(AIRA_LAUNCH_AGENT_LABEL));
        assert!(body.contains("<string>/Users/dev/bin/aira-desktop</string>"));
        assert!(body.contains("<key>RunAtLoad</key>"));
        assert!(body.contains("<?xml"));
    }

    #[test]
    fn launch_agent_plist_escapes_xml() {
        let body = launch_agent_plist("/tmp/a&b<c>.bin");
        assert!(body.contains("/tmp/a&amp;b&lt;c&gt;.bin"));
    }

    #[test]
    fn set_launch_agent_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = launch_agents_dir_for_home(tmp.path());
        assert!(!is_launch_agent_enabled_in(&dir));
        let path = set_launch_agent_in(&dir, true, "/opt/aira/bin/aira-desktop").unwrap();
        assert!(path.ends_with(AIRA_LAUNCH_AGENT_FILENAME));
        assert!(is_launch_agent_enabled_in(&dir));
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text, launch_agent_plist("/opt/aira/bin/aira-desktop"));
        set_launch_agent_in(&dir, false, "/opt/aira/bin/aira-desktop").unwrap();
        assert!(!is_launch_agent_enabled_in(&dir));
    }
}
