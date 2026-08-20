//! AIRA Desktop lifecycle library (QUEUE #76 / Analyze-111).
//!
//! Shared by `aira desktop …` and the Desktop GUI. OS autostart is `#78`;
//! Linux packaging layout/install is `#79`. P1 peer supervise is `#82`.

mod autostart;
mod bootstrap;
mod health;
mod invite;
mod launcher;
mod paths;
mod peer;
mod process;
mod settings;

pub use autostart::{
    autostart_desktop_entry, is_autostart_enabled, is_autostart_enabled_in, set_autostart,
    set_autostart_in, sync_autostart_from_settings, AIRA_AUTOSTART_FILENAME,
};
pub use bootstrap::ensure_bootstrap;
pub use invite::{
    build_local_invite, export_invite_file, import_invite, import_invite_file, load_invite_file,
    validate_peer_invite, ImportInviteOutcome, PeerInvite, PEER_INVITE_SCHEMA_ID,
};
pub use launcher::{
    install_gui_launcher_to, install_launcher_to, install_menu_entries_to, install_user_launcher,
    install_user_menu_entries, uninstall_gui_launcher_from, uninstall_launcher_from,
    uninstall_menu_entries_from, uninstall_user_launcher, uninstall_user_menu_entries,
    validate_desktop_entry, validate_desktop_file, validate_gui_desktop_entry,
    validate_gui_desktop_file, AIRA_DESKTOP_ENTRY, AIRA_DESKTOP_FILENAME, AIRA_GUI_DESKTOP_ENTRY,
    AIRA_GUI_DESKTOP_FILENAME,
};
pub use paths::DesktopPaths;
pub use peer::PeerPidRecordView;
pub use process::{start, status, stop, LifecycleStatus, PidRecordView, StartOutcome};
pub use settings::{
    effective_peer_listen, load_or_create_settings, normalize_settings, validate_listen_addr,
    write_settings, DesktopSettings, HttpAuthMode, NetworkProfile, DEFAULT_PEER_LISTEN,
    SETTINGS_SCHEMA_ID,
};

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
