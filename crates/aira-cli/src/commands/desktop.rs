//! `aira desktop start|stop|status|gui` (QUEUE #76 / #78).

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use anyhow::{bail, Context, Result};

use aira_desktop_runtime::{
    export_invite_file, import_invite_file, install_user_menu_entries, start, status, stop,
    uninstall_user_menu_entries, DesktopPaths, LifecycleStatus,
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
            if let Some(pp) = outcome.peer_pid {
                println!("peer_pid {pp}");
            }
            if let Some(pl) = outcome.peer_listen.as_ref() {
                println!("peer_listen {pl}");
                if outcome.peer_attached {
                    println!("peer_attached");
                }
            }
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
                if let Some(pp) = r.peer_pid {
                    println!("peer_pid {pp}");
                }
                if let Some(pl) = r.peer_listen.as_ref() {
                    println!("peer_listen {pl}");
                }
            }
            println!("settings {}", paths.settings_file.display());
            println!("data_root {}", paths.data_root.display());
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::LauncherInstall => {
            let (start_entry, gui_entry) = install_user_menu_entries()?;
            println!("installed {}", start_entry.display());
            println!("installed {}", gui_entry.display());
            println!("start: menu → AIRA  (or `aira desktop start`)");
            println!("stop:  menu → AIRA → Stop AIRA  (or `aira desktop stop`)");
            println!("gui:   menu → AIRA Desktop  (or `aira-desktop`)");
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::LauncherUninstall => {
            let removed = uninstall_user_menu_entries()?;
            if removed.is_empty() {
                println!("not installed");
            } else {
                for p in removed {
                    println!("removed {}", p.display());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::Gui {
            data_root,
            node_bin,
            no_auto_start,
            force_ui,
        } => {
            let bin = resolve_desktop_bin()?;
            let mut cmd = Command::new(&bin);
            if let Some(r) = data_root {
                cmd.arg("--data-root").arg(r);
            }
            if let Some(n) = node_bin {
                cmd.arg("--node-bin").arg(n);
            }
            if no_auto_start {
                cmd.arg("--no-auto-start");
            }
            if force_ui {
                cmd.arg("--force-ui");
            }
            let status = cmd
                .status()
                .with_context(|| format!("spawn {}", bin.display()))?;
            if status.success() {
                Ok(ExitCode::SUCCESS)
            } else {
                bail!("aira-desktop exited with {status}");
            }
        }
        DesktopCommands::InviteExport {
            data_root,
            out,
            addr,
        } => {
            let paths = resolve_paths(data_root);
            let invite = export_invite_file(&paths, &out, addr)?;
            println!("exported {}", out.display());
            println!("identity_ref {}", invite.identity_ref);
            if let Some(a) = invite.addr.as_ref() {
                println!("addr {a}");
            } else {
                println!("addr (none — trust-only)");
            }
            Ok(ExitCode::SUCCESS)
        }
        DesktopCommands::InviteImport { data_root, file } => {
            let paths = resolve_paths(data_root);
            let out = import_invite_file(&paths, &file)?;
            println!("imported {}", file.display());
            println!("trusted {}", out.identity_ref);
            if out.book_updated {
                if let Some(a) = out.addr.as_ref() {
                    println!("address_book {} -> {a}", out.identity_ref);
                }
            } else {
                println!("address_book (skipped — no addr)");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn resolve_desktop_bin() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("AIRA_DESKTOP_BIN") {
        return Ok(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("aira-desktop");
            if cand.is_file() {
                return Ok(cand);
            }
        }
    }
    Ok(PathBuf::from("aira-desktop"))
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
