//! `aira desktop start|stop|status` (QUEUE #76 / Analyze-111).

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;

use aira_desktop_runtime::{
    install_user_launcher, start, status, stop, uninstall_user_launcher, DesktopPaths,
    LifecycleStatus,
};

use crate::cli::DesktopCommands;

pub(crate) fn run(command: DesktopCommands) -> Result<ExitCode> {
    match command {
        DesktopCommands::Start {
            data_root,
            node_bin,
        } => {
            let paths = resolve_paths(data_root);
            let outcome = start(&paths, node_bin)?;
            if outcome.attached {
                println!("attached");
            } else {
                println!("started");
            }
            println!("status {}", status_label(outcome.status));
            if let Some(pid) = outcome.pid {
                println!("pid {pid}");
            }
            println!("listen {}", outcome.listen);
            println!("instance_id {}", outcome.instance_id);
            println!("data_root {}", outcome.data_root.display());
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::Stop { data_root } => {
            let paths = resolve_paths(data_root);
            let st = stop(&paths)?;
            println!("status {}", status_label(st));
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::Status { data_root } => {
            let paths = resolve_paths(data_root);
            let (st, rec) = status(&paths)?;
            println!("status {}", status_label(st));
            if let Some(r) = rec {
                println!("pid {}", r.pid);
                println!("instance_id {}", r.instance_id);
                println!("listen {}", r.listen);
                println!("root {}", r.root);
            }
            println!("settings {}", paths.settings_file.display());
            println!("data_root {}", paths.data_root.display());
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::LauncherInstall => {
            let dest = install_user_launcher()?;
            println!("installed {}", dest.display());
            println!("start: menu → AIRA  (or `aira desktop start`)");
            println!("stop:  menu → AIRA → Stop AIRA  (or `aira desktop stop`)");
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::LauncherUninstall => {
            match uninstall_user_launcher()? {
                Some(p) => println!("removed {}", p.display()),
                None => println!("not installed"),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn resolve_paths(data_root: Option<PathBuf>) -> DesktopPaths {
    match data_root {
        Some(root) => DesktopPaths::for_data_root(root),
        None => DesktopPaths::system(),
    }
}

fn status_label(st: LifecycleStatus) -> &'static str {
    match st {
        LifecycleStatus::Stopped => "stopped",
        LifecycleStatus::Starting => "starting",
        LifecycleStatus::Running => "running",
        LifecycleStatus::Unhealthy => "unhealthy",
        LifecycleStatus::Stopping => "stopping",
        LifecycleStatus::Failed => "failed",
    }
}
