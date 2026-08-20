//! AIRA Desktop lifecycle library (QUEUE #76 / Analyze-111).
//!
//! Shared by `aira desktop …` and future tray/GUI. Does **not** implement OS
//! autostart hooks (those are `#78`).

mod bootstrap;
mod health;
mod paths;
mod process;
mod settings;

pub use bootstrap::ensure_bootstrap;
pub use paths::DesktopPaths;
pub use process::{start, status, stop, LifecycleStatus, PidRecordView, StartOutcome};
pub use settings::{
    load_or_create_settings, write_settings, DesktopSettings, HttpAuthMode, NetworkProfile,
};

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
