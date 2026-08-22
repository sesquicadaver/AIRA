//! OS autostart hooks (QUEUE #78 Linux XDG; QUEUE #87 macOS LaunchAgent; QUEUE #91 Windows Startup).
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

/// Startup folder batch hook basename (Windows).
pub const AIRA_WINDOWS_STARTUP_FILENAME: &str = "AIRA Desktop.bat";

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

/// Startup `.bat` body for Windows login autostart (`start "" "<program>"`).
pub fn windows_startup_bat(program: &str) -> String {
    let program = program.trim().replace('"', "");
    format!("@echo off\r\nstart \"\" \"{program}\"\r\n")
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

/// `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup` (Windows).
pub fn user_windows_startup_dir() -> PathBuf {
    let app_data = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    windows_startup_dir_for_app_data(&app_data)
}

/// Startup folder under an explicit `%APPDATA%` root (tests).
pub fn windows_startup_dir_for_app_data(app_data: &Path) -> PathBuf {
    app_data.join("Microsoft/Windows/Start Menu/Programs/Startup")
}

pub fn autostart_path_in(dir: &Path) -> PathBuf {
    dir.join(AIRA_AUTOSTART_FILENAME)
}

pub fn launch_agent_path_in(dir: &Path) -> PathBuf {
    dir.join(AIRA_LAUNCH_AGENT_FILENAME)
}

pub fn windows_startup_path_in(dir: &Path) -> PathBuf {
    dir.join(AIRA_WINDOWS_STARTUP_FILENAME)
}

pub fn is_autostart_enabled_in(dir: &Path) -> bool {
    autostart_path_in(dir).is_file()
}

pub fn is_launch_agent_enabled_in(dir: &Path) -> bool {
    launch_agent_path_in(dir).is_file()
}

pub fn is_windows_startup_enabled_in(dir: &Path) -> bool {
    windows_startup_path_in(dir).is_file()
}

/// Whether the OS-native autostart hook is present for this build.
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        is_windows_startup_enabled_in(&user_windows_startup_dir())
    }
    #[cfg(target_os = "macos")]
    {
        is_launch_agent_enabled_in(&user_launch_agents_dir())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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

/// Write or remove a Windows Startup `.bat` under `dir`.
pub fn set_windows_startup_in(dir: &Path, enabled: bool, program: &str) -> Result<PathBuf> {
    let path = windows_startup_path_in(dir);
    if enabled {
        fs::create_dir_all(dir).with_context(|| format!("mkdir {}", dir.display()))?;
        let body = windows_startup_bat(program);
        fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    } else if path.is_file() {
        fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(path)
}

/// Resolve `aira-desktop` for autostart hooks (absolute when found).
pub fn resolve_desktop_program() -> String {
    if let Ok(p) = std::env::var("AIRA_DESKTOP_BIN") {
        let t = p.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in desktop_binary_names() {
                let cand = dir.join(name);
                if cand.is_file() {
                    return cand.display().to_string();
                }
            }
        }
    }
    if let Some(found) = resolve_desktop_on_path() {
        return found;
    }
    desktop_binary_names()[0].to_string()
}

fn desktop_binary_names() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["aira-desktop.exe", "aira-desktop"]
    }
    #[cfg(not(target_os = "windows"))]
    {
        &["aira-desktop"]
    }
}

fn resolve_desktop_on_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    let cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let cmd = "which";

    for name in desktop_binary_names() {
        if let Ok(out) = Command::new(cmd).arg(name).output() {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }
    None
}

/// Enable/disable the OS-native autostart hook for this build.
pub fn set_autostart(enabled: bool) -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        set_windows_startup_in(
            &user_windows_startup_dir(),
            enabled,
            &resolve_desktop_program(),
        )
    }
    #[cfg(target_os = "macos")]
    {
        set_launch_agent_in(
            &user_launch_agents_dir(),
            enabled,
            &resolve_desktop_program(),
        )
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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

    #[test]
    fn windows_startup_bat_contains_program() {
        let body = windows_startup_bat(r"C:\Program Files\AIRA\aira-desktop.exe");
        assert!(body.contains("@echo off"));
        assert!(body.contains(r#"start "" "C:\Program Files\AIRA\aira-desktop.exe""#));
    }

    #[test]
    fn set_windows_startup_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = windows_startup_dir_for_app_data(tmp.path());
        let prog = r"C:\Users\dev\AppData\Local\Programs\AIRA\aira-desktop.exe";
        assert!(!is_windows_startup_enabled_in(&dir));
        let path = set_windows_startup_in(&dir, true, prog).unwrap();
        assert!(path.ends_with(AIRA_WINDOWS_STARTUP_FILENAME));
        assert!(is_windows_startup_enabled_in(&dir));
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text, windows_startup_bat(prog));
        set_windows_startup_in(&dir, false, prog).unwrap();
        assert!(!is_windows_startup_enabled_in(&dir));
    }
}
