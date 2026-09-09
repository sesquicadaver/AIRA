//! AIRA Desktop GUI — Work / Node / Network / Settings (QUEUE #78 / #85).
//!
//! Uses `aira-desktop-runtime` for lifecycle and OS autostart; no CLI shell-out.

mod actions;
mod app;
mod async_jobs;
mod camera;
mod connection_cta;
mod help;
mod lexicon;
mod mesh_language;
mod problem_action;
mod settings_apply;
mod system_view;
mod work_view;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use aira_desktop_runtime::{
    load_or_create_settings, should_show_window, start, sync_autostart_from_settings, DesktopPaths,
};

use crate::app::AiraDesktopApp;
use crate::connection_cta::ConnectionPrimaryCta;
use crate::lexicon::{ActionId, HelpId};
use crate::mesh_language::StripNetworkPhrase;
use crate::problem_action::action_next_step;

fn assert_lexicon_linked() {
    // Keep Phase O lexicon reachable in the binary (`#258` → F1 `#263`).
    assert_eq!(HelpId::catalog().len(), 11);
    assert_eq!(ActionId::catalog().len(), 6);
    assert_eq!(ActionId::Quit.as_str(), "app.quit");
    assert_eq!(ActionId::Quit.help_id(), HelpId::NodeLifecycle);
    assert_eq!(HelpId::parse("work.submit"), Some(HelpId::WorkSubmit));
    assert!(HelpId::parse("missing").is_none());
    // Phase R `#287` Connection CTA wire ids stay linked.
    assert_eq!(
        ConnectionPrimaryCta::RefreshStatus.as_str(),
        "connection.refresh_status"
    );
    assert_eq!(
        ConnectionPrimaryCta::EnablePrivateNetwork.help_id(),
        HelpId::NetworkConnect
    );
    // Phase R `#289` strip phrases stay linked (UNKNOWN ≠ OFFLINE).
    assert_eq!(
        StripNetworkPhrase::NotChecked.as_str(),
        "strip.network.not_checked"
    );
    assert_ne!(StripNetworkPhrase::NotChecked, StripNetworkPhrase::Offline);
    // Phase R `#290` human next-step copy stays linked (not wire-only).
    let start_en = action_next_step(ActionId::NodeStart, aira_desktop_runtime::UiLang::En);
    assert!(start_en.contains("Start"));
    assert!(!start_en.contains("node.start"));
}

#[derive(Parser, Debug)]
#[command(
    name = "aira-desktop",
    version,
    about = "AIRA Desktop (Developer Preview) — local P0/P1 node UI"
)]
struct Args {
    /// Dev/test data root (colocated settings/runtime). Default: OS Desktop layout.
    #[arg(long)]
    data_root: Option<PathBuf>,
    /// Path to `aira-node` (else `AIRA_NODE_BIN` / sibling / PATH).
    #[arg(long)]
    node_bin: Option<PathBuf>,
    /// Do not auto-start the node on launch.
    #[arg(long, default_value_t = false)]
    no_auto_start: bool,
    /// Force show the UI even when login autostart would be headless.
    #[arg(long, default_value_t = false)]
    force_ui: bool,
    /// Login autostart launch: honor `open_ui_on_start`. Interactive launches ignore that flag.
    #[arg(long, default_value_t = false)]
    from_autostart: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    assert_lexicon_linked();
    let args = Args::parse();
    let paths = match args.data_root {
        Some(r) => DesktopPaths::for_data_root(r),
        None => DesktopPaths::system(),
    };
    let settings = load_or_create_settings(&paths)?;
    let _ = sync_autostart_from_settings(settings.autostart_on_login);

    let auto_start = !args.no_auto_start;
    let show_ui = should_show_window(
        args.force_ui,
        args.from_autostart,
        settings.open_ui_on_start,
    );

    if !show_ui {
        if auto_start {
            let outcome = start(&paths, args.node_bin.clone())?;
            println!("started (no UI)");
            println!(
                "status {}",
                match outcome.status {
                    aira_desktop_runtime::LifecycleStatus::Stopped => "stopped",
                    aira_desktop_runtime::LifecycleStatus::Starting => "starting",
                    aira_desktop_runtime::LifecycleStatus::Running => "running",
                    aira_desktop_runtime::LifecycleStatus::Unhealthy => "unhealthy",
                    aira_desktop_runtime::LifecycleStatus::Stopping => "stopping",
                    aira_desktop_runtime::LifecycleStatus::Failed => "failed",
                }
            );
            if let Some(pid) = outcome.pid {
                println!("pid {pid}");
            }
            println!("listen {}", outcome.listen);
            if let Some(pp) = outcome.peer_pid {
                println!("peer_pid {pp}");
            }
            if let Some(pl) = outcome.peer_listen.as_ref() {
                println!("peer_listen {pl}");
            }
        }
        return Ok(());
    }

    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 720.0])
            .with_title("AIRA Desktop"),
        ..Default::default()
    };
    let node_bin = args.node_bin;
    eframe::run_native(
        "AIRA Desktop",
        native,
        Box::new(move |cc| {
            Ok(Box::new(AiraDesktopApp::new(
                cc, paths, node_bin, auto_start,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))?;
    Ok(())
}
