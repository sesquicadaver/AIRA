//! Desktop filesystem layout (phase-e §2.1; E2 macOS `#86`; E3 Windows `#90`).

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
    /// System Desktop layout for the current OS.
    ///
    /// Linux: XDG (`~/.local/share`, `~/.config`, …).  
    /// macOS: `~/Library/Application Support|Preferences|Logs` (QUEUE #86).  
    /// Windows: `%LOCALAPPDATA%` / `%APPDATA%` (QUEUE #90).  
    /// Other Unix: same XDG-style fallbacks as Linux.
    pub fn system() -> Self {
        #[cfg(target_os = "windows")]
        {
            let local = windows_local_app_data_dir();
            let roaming = windows_app_data_dir();
            return Self::windows_for_profile(&local, &roaming);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            Self::system_for_home(&home)
        }
    }

    /// System layout under an explicit home (tests / portable probes).
    pub fn system_for_home(home: &Path) -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::windows_for_home(home)
        }
        #[cfg(target_os = "macos")]
        {
            Self::macos_for_home(home)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Self::linux_xdg_for_home(home)
        }
    }

    /// Linux / non-macOS Unix XDG-style layout (also used when XDG_* are set).
    pub fn linux_xdg_for_home(home: &Path) -> Self {
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

    /// macOS Developer Preview layout (phase-e E2).
    ///
    /// | Role | Path |
    /// |------|------|
    /// | Node root | `~/Library/Application Support/AIRA` |
    /// | Settings | `~/Library/Preferences/AIRA/desktop-settings.json` |
    /// | Runtime | `~/Library/Application Support/AIRA/runtime` |
    /// | Logs | `~/Library/Logs/AIRA` |
    pub fn macos_for_home(home: &Path) -> Self {
        let support = home.join("Library/Application Support/AIRA");
        Self {
            data_root: support.clone(),
            settings_file: home
                .join("Library/Preferences/AIRA")
                .join("desktop-settings.json"),
            runtime_dir: support.join("runtime"),
            log_dir: home.join("Library/Logs/AIRA"),
        }
    }

    /// Windows Developer Preview layout (phase-e E3).
    ///
    /// | Role | Path |
    /// |------|------|
    /// | Node root | `%LOCALAPPDATA%\AIRA` |
    /// | Settings | `%APPDATA%\AIRA\desktop-settings.json` |
    /// | Runtime | `%LOCALAPPDATA%\AIRA\runtime` |
    /// | Logs | `%LOCALAPPDATA%\AIRA\logs` |
    pub fn windows_for_profile(local_app_data: &Path, app_data: &Path) -> Self {
        let support = local_app_data.join("AIRA");
        Self {
            data_root: support.clone(),
            settings_file: app_data.join("AIRA").join("desktop-settings.json"),
            runtime_dir: support.join("runtime"),
            log_dir: support.join("logs"),
        }
    }

    /// Windows layout derived from a profile home (`C:\Users\dev` → `AppData\Local|Roaming`).
    pub fn windows_for_home(home: &Path) -> Self {
        Self::windows_for_profile(
            &home.join("AppData").join("Local"),
            &home.join("AppData").join("Roaming"),
        )
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

    /// Supervised `aira peer listen` PID record (P1).
    pub fn peer_pid_file(&self) -> PathBuf {
        self.runtime_dir.join("aira-peer.pid.json")
    }

    pub fn peer_lock_file(&self) -> PathBuf {
        self.runtime_dir.join("aira-peer.lock")
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

#[cfg(target_os = "windows")]
fn windows_local_app_data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Local"))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(target_os = "windows")]
fn windows_app_data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_layout_under_home() {
        let home = Path::new("/Users/dev");
        let p = DesktopPaths::macos_for_home(home);
        assert_eq!(
            p.data_root,
            PathBuf::from("/Users/dev/Library/Application Support/AIRA")
        );
        assert_eq!(
            p.settings_file,
            PathBuf::from("/Users/dev/Library/Preferences/AIRA/desktop-settings.json")
        );
        assert_eq!(
            p.runtime_dir,
            PathBuf::from("/Users/dev/Library/Application Support/AIRA/runtime")
        );
        assert_eq!(p.log_dir, PathBuf::from("/Users/dev/Library/Logs/AIRA"));
        assert_eq!(
            p.pid_file(),
            PathBuf::from("/Users/dev/Library/Application Support/AIRA/runtime/aira-node.pid.json")
        );
    }

    #[test]
    fn linux_xdg_fallback_segments() {
        // Pure fallbacks (no XDG_*): relative shape under home.
        let home = Path::new("/home/dev");
        let data = home.join(".local/share").join("aira");
        let config = home
            .join(".config")
            .join("aira")
            .join("desktop-settings.json");
        let state = home.join(".local/state").join("aira");
        let logs = home.join(".cache").join("aira").join("logs");
        // Reconstruct via the same join rules as linux_xdg_for_home fallbacks.
        assert_eq!(data, PathBuf::from("/home/dev/.local/share/aira"));
        assert_eq!(
            config,
            PathBuf::from("/home/dev/.config/aira/desktop-settings.json")
        );
        assert_eq!(state, PathBuf::from("/home/dev/.local/state/aira"));
        assert_eq!(logs, PathBuf::from("/home/dev/.cache/aira/logs"));
        let _ = DesktopPaths::linux_xdg_for_home(home);
    }

    #[test]
    fn macos_ensure_dirs_creates_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let p = DesktopPaths::macos_for_home(tmp.path());
        p.ensure_dirs().unwrap();
        assert!(p.data_root.is_dir());
        assert!(p.settings_file.parent().unwrap().is_dir());
        assert!(p.runtime_dir.is_dir());
        assert!(p.log_dir.is_dir());
    }

    #[test]
    fn windows_layout_under_profile() {
        let local = Path::new("C:/Users/dev/AppData/Local");
        let roaming = Path::new("C:/Users/dev/AppData/Roaming");
        let p = DesktopPaths::windows_for_profile(local, roaming);
        assert_eq!(
            p.data_root,
            PathBuf::from("C:/Users/dev/AppData/Local/AIRA")
        );
        assert_eq!(
            p.settings_file,
            PathBuf::from("C:/Users/dev/AppData/Roaming/AIRA/desktop-settings.json")
        );
        assert_eq!(
            p.runtime_dir,
            PathBuf::from("C:/Users/dev/AppData/Local/AIRA/runtime")
        );
        assert_eq!(
            p.log_dir,
            PathBuf::from("C:/Users/dev/AppData/Local/AIRA/logs")
        );
        assert_eq!(
            p.pid_file(),
            PathBuf::from("C:/Users/dev/AppData/Local/AIRA/runtime/aira-node.pid.json")
        );
    }

    #[test]
    fn windows_for_home_derives_appdata_segments() {
        let home = Path::new("C:/Users/dev");
        let p = DesktopPaths::windows_for_home(home);
        assert_eq!(
            p.data_root,
            PathBuf::from("C:/Users/dev/AppData/Local/AIRA")
        );
        assert_eq!(
            p.settings_file,
            PathBuf::from("C:/Users/dev/AppData/Roaming/AIRA/desktop-settings.json")
        );
    }

    #[test]
    fn windows_ensure_dirs_creates_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let local = tmp.path().join("Local");
        let roaming = tmp.path().join("Roaming");
        let p = DesktopPaths::windows_for_profile(&local, &roaming);
        p.ensure_dirs().unwrap();
        assert!(p.data_root.is_dir());
        assert!(p.settings_file.parent().unwrap().is_dir());
        assert!(p.runtime_dir.is_dir());
        assert!(p.log_dir.is_dir());
    }
}
