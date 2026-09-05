//! AIRA Desktop lifecycle library (QUEUE #76 / Analyze-111).
//!
//! Shared by `aira desktop …` and the Desktop GUI. OS autostart is `#78` (Linux)
//! / `#87` (macOS LaunchAgent) / `#91` (Windows Startup). Linux packaging is `#79`.
//! P1 peer supervise is `#82`. Invite file IO is `#83`; QR PNG is `#84`. macOS paths are `#86`.
//! Invite file IO is `#83`; QR PNG is `#84`. macOS paths are `#86`.

mod autostart;
mod bootstrap;
mod discovery;
mod federation;
mod health;
mod invite;
mod invite_qr;
mod launcher;
mod network_mesh;
mod node_http;
mod paths;
mod peer;
mod process;
mod settings;
mod ui_prefs;

pub use autostart::{
    autostart_desktop_entry, is_autostart_enabled, is_autostart_enabled_in,
    is_launch_agent_enabled_in, is_windows_startup_enabled_in, launch_agent_path_in,
    launch_agent_plist, launch_agents_dir_for_home, resolve_desktop_program, set_autostart,
    set_autostart_in, set_launch_agent_in, set_windows_startup_in, should_show_window,
    sync_autostart_from_settings, user_launch_agents_dir, user_windows_startup_dir,
    windows_startup_bat, windows_startup_dir_for_app_data, windows_startup_path_in,
    AIRA_AUTOSTART_FILENAME, AIRA_LAUNCH_AGENT_FILENAME, AIRA_LAUNCH_AGENT_LABEL,
    AIRA_WINDOWS_STARTUP_FILENAME, FROM_AUTOSTART_FLAG,
};
pub use bootstrap::ensure_bootstrap;
pub use discovery::{run_discv_announce, run_discv_find, run_stun_query, DiscoveryStunOutcome};
pub use federation::{
    join_federation_descriptor_file, leave_federation_local, read_federation_membership,
};
pub use invite::{
    build_local_invite, export_invite_file, import_invite, import_invite_file, load_invite_file,
    validate_peer_invite, ImportInviteOutcome, PeerInvite, PEER_INVITE_SCHEMA_ID,
};
pub use invite_qr::{
    decode_invite_luma, decode_invite_png, encode_invite_luma, encode_invite_png,
    encode_invite_rgba, export_invite_qr_png, import_invite_qr_file, import_invite_qr_luma,
    invite_qr_payload,
};
pub use launcher::{
    install_gui_launcher_to, install_launcher_to, install_menu_entries_to, install_user_launcher,
    install_user_menu_entries, uninstall_gui_launcher_from, uninstall_launcher_from,
    uninstall_menu_entries_from, uninstall_user_launcher, uninstall_user_menu_entries,
    validate_desktop_entry, validate_desktop_file, validate_gui_desktop_entry,
    validate_gui_desktop_file, AIRA_DESKTOP_ENTRY, AIRA_DESKTOP_FILENAME, AIRA_GUI_DESKTOP_ENTRY,
    AIRA_GUI_DESKTOP_FILENAME,
};
pub use network_mesh::{load_network_mesh_snapshot, MeshTopLevel, NetworkMeshSnapshot};
pub use node_http::{submit_desktop_problem, submit_problem_http};
pub use paths::DesktopPaths;
pub use peer::PeerPidRecordView;
pub use process::{start, status, stop, LifecycleStatus, PidRecordView, StartOutcome};
pub use settings::{
    effective_peer_listen, effective_relay_ttl_days, load_or_create_settings, normalize_settings,
    validate_listen_addr, write_settings, DesktopSettings, HttpAuthMode, NetworkProfile,
    DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS, SETTINGS_SCHEMA_ID,
};
pub use ui_prefs::{detect_ui_lang, load_or_create_ui_prefs, write_ui_prefs, UiLang, UiPrefs};

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
