//! Sidecar UI language prefs (`ui-prefs.json`).
//!
//! Kept out of `aira:schema:desktop:settings:0.1` so language does not require
//! a settings-schema RFC. Missing file → detect from `LANG` / `LC_*`.

use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths::DesktopPaths;

/// UI chrome language (Work / Node / Network / Settings labels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLang {
    En,
    Uk,
}

impl UiLang {
    /// Parse `uk` / `en`; unknown → English.
    pub fn parse_code(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "uk" | "ua" | "ukrainian" => Self::Uk,
            _ => Self::En,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPrefs {
    pub ui_lang: UiLang,
}

impl UiPrefs {
    pub fn new(ui_lang: UiLang) -> Self {
        Self { ui_lang }
    }
}

/// Detect language from POSIX locale env (`uk*` → Ukrainian).
pub fn detect_ui_lang() -> UiLang {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if t.is_empty() || t == "C" || t.eq_ignore_ascii_case("POSIX") {
                continue;
            }
            let lower = t.to_ascii_lowercase();
            if lower.starts_with("uk") {
                return UiLang::Uk;
            }
        }
    }
    UiLang::En
}

/// Load sidecar or create from locale.
pub fn load_or_create_ui_prefs(paths: &DesktopPaths) -> Result<UiPrefs> {
    let path = paths.ui_prefs_file();
    if path.is_file() {
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let prefs: UiPrefs =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        return Ok(prefs);
    }
    let prefs = UiPrefs::new(detect_ui_lang());
    write_ui_prefs(paths, &prefs)?;
    Ok(prefs)
}

/// Persist sidecar next to `desktop-settings.json`.
pub fn write_ui_prefs(paths: &DesktopPaths, prefs: &UiPrefs) -> Result<()> {
    if let Some(parent) = paths.ui_prefs_file().parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }
    let path = paths.ui_prefs_file();
    let text = serde_json::to_string_pretty(prefs)?;
    fs::write(&path, format!("{text}\n")).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uk_codes() {
        assert_eq!(UiLang::parse_code("uk"), UiLang::Uk);
        assert_eq!(UiLang::parse_code("UK"), UiLang::Uk);
        assert_eq!(UiLang::parse_code("en"), UiLang::En);
        assert_eq!(UiLang::parse_code("de"), UiLang::En);
    }

    #[test]
    fn roundtrip_sidecar() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        write_ui_prefs(&paths, &UiPrefs::new(UiLang::Uk)).unwrap();
        let loaded = load_or_create_ui_prefs(&paths).unwrap();
        assert_eq!(loaded.ui_lang, UiLang::Uk);
        assert!(paths.ui_prefs_file().is_file());
        assert_ne!(paths.ui_prefs_file(), paths.settings_file);
    }
}
