//! AIRA Desktop GUI — native Status / Settings / Quit (QUEUE #78).
//!
//! Uses `aira-desktop-runtime` for lifecycle and XDG autostart; no CLI shell-out.

mod app;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use aira_desktop_runtime::{
    load_or_create_settings, start, sync_autostart_from_settings, DesktopPaths,
};

use crate::app::AiraDesktopApp;

#[derive(Parser, Debug)]
#[command(
    name = "aira-desktop",
    version,
    about = "AIRA Desktop (Developer Preview) — local P0 node UI"
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
    /// Force show the UI even when `open_ui_on_start=false`.
    #[arg(long, default_value_t = false)]
    force_ui: bool,
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
    let args = Args::parse();
    let paths = match args.data_root {
        Some(r) => DesktopPaths::for_data_root(r),
        None => DesktopPaths::system(),
    };
    let settings = load_or_create_settings(&paths)?;
    let _ = sync_autostart_from_settings(settings.autostart_on_login);

    let auto_start = !args.no_auto_start;
    let show_ui = args.force_ui || settings.open_ui_on_start;

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
        }
        return Ok(());
    }

    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 360.0])
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
