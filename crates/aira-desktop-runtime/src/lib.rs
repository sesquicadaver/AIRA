//! AIRA Desktop lifecycle library (QUEUE #76 / Analyze-111).
//!
//! Shared by `aira desktop …` and future tray/GUI. Does **not** implement OS
//! autostart hooks (those are `#78`).

mod autostart;
mod bootstrap;
mod health;
mod launcher;
mod paths;
mod process;
mod settings;

pub use autostart::{
    autostart_desktop_entry, is_autostart_enabled, is_autostart_enabled_in, set_autostart,
    set_autostart_in, sync_autostart_from_settings, AIRA_AUTOSTART_FILENAME,
};
pub use bootstrap::ensure_bootstrap;
pub use launcher::{
    install_launcher_to, install_user_launcher, uninstall_launcher_from, uninstall_user_launcher,
    validate_desktop_entry, validate_desktop_file, AIRA_DESKTOP_ENTRY, AIRA_DESKTOP_FILENAME,
};
pub use paths::DesktopPaths;
pub use process::{start, status, stop, LifecycleStatus, PidRecordView, StartOutcome};
pub use settings::{
    load_or_create_settings, write_settings, DesktopSettings, HttpAuthMode, NetworkProfile,
};

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
