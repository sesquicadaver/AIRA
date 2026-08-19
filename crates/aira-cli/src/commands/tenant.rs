//! Identity csu-tenant subcommands (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_flow::NodePaths;
use anyhow::{bail, Result};

use crate::cli::{CsuTenantBackupsCommands, CsuTenantCommands};
use crate::support::ensure_init;
use crate::tenant_secret;

pub(crate) fn run(root: &Path, command: CsuTenantCommands) -> Result<ExitCode> {
    match command {
        CsuTenantCommands::Register {
            csu_id,
            publisher,
            secret_hex,
            secret_hex_file,
            force,
        } => {
            ensure_init(root)?;
            let csu = aira_object::AiraRef::parse(&csu_id)
                .map_err(|e| anyhow::anyhow!("invalid --csu-id: {e}"))?;
            let pub_id = aira_object::AiraRef::parse(&publisher)
                .map_err(|e| anyhow::anyhow!("invalid --publisher: {e}"))?;
            let sk = tenant_secret::resolve_tenant_signing(
                secret_hex.as_deref(),
                secret_hex_file.as_deref(),
            )?;
            let pub_hex = hex::encode(sk.verifying_key().to_bytes());
            let dir = aira_object::save_csu_tenant_signing(root, &csu, pub_id, sk, force)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("csu_tenant {}", csu.as_str());
            println!("publisher {publisher}");
            println!("public_key {pub_hex}");
            println!("path {}", dir.display());
            Ok(ExitCode::SUCCESS)
        }
        CsuTenantCommands::List => {
            ensure_init(root)?;
            let list =
                aira_object::list_csu_tenant_signing(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if list.is_empty() {
                println!("(no csu tenants — use `identity csu-tenant register`)");
            } else {
                for t in &list {
                    println!(
                        "{}\t{}\t{}\t{}",
                        t.csu_id,
                        t.publisher_id,
                        t.public_key_hex,
                        t.dir.display()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        CsuTenantCommands::Load => {
            ensure_init(root)?;
            let n = aira_object::load_all_csu_tenant_signing(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("loaded {n}");
            Ok(ExitCode::SUCCESS)
        }
        CsuTenantCommands::Rotate {
            csu_id,
            backup,
            secret_hex,
            secret_hex_file,
        } => {
            ensure_init(root)?;
            let csu = aira_object::AiraRef::parse(&csu_id)
                .map_err(|e| anyhow::anyhow!("invalid --csu-id: {e}"))?;
            let sk = tenant_secret::resolve_tenant_signing(
                secret_hex.as_deref(),
                secret_hex_file.as_deref(),
            )?;
            let (publisher, new_pub, old_pub, backup_path) =
                aira_object::rotate_csu_tenant_signing(root, &csu, sk, backup)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("rotated {}", csu.as_str());
            println!("publisher {}", publisher.as_str());
            println!("public_key {new_pub}");
            println!("old_public_key {old_pub}");
            if let Some(p) = backup_path {
                println!("backup {}", p.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        CsuTenantCommands::Revoke { csu_id, reason } => {
            ensure_init(root)?;
            let csu = aira_object::AiraRef::parse(&csu_id)
                .map_err(|e| anyhow::anyhow!("invalid --csu-id: {e}"))?;
            aira_object::revoke_csu_tenant_signing(root, &csu, &reason)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("revoked {}", csu.as_str());
            println!("reason {reason}");
            Ok(ExitCode::SUCCESS)
        }
        CsuTenantCommands::Backups { command: None } => {
            ensure_init(root)?;
            let list = aira_object::list_csu_tenant_secret_backups(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if list.is_empty() {
                println!("(no csu tenant backups — use `identity csu-tenant rotate --backup`)");
            } else {
                for b in &list {
                    let pk = b.old_public_key_hex.as_deref().unwrap_or("-");
                    let at = b.backed_up_at.as_deref().unwrap_or("-");
                    println!(
                        "{}\t{}\t{}\t{}\t{}",
                        b.csu_id,
                        b.stamp,
                        pk,
                        at,
                        b.secret_path.display()
                    );
                }
            }
            println!(
                "tenant_backups {}",
                NodePaths::new(root)
                    .identity_dir()
                    .join(aira_object::CSU_TENANTS_DIR)
                    .display()
            );
            Ok(ExitCode::SUCCESS)
        }
        CsuTenantCommands::Backups {
            command:
                Some(CsuTenantBackupsCommands::Prune {
                    keep,
                    older_than_days,
                    dry_run,
                }),
        } => {
            ensure_init(root)?;
            if keep.is_none() && older_than_days.is_none() {
                bail!("backups prune requires --keep and/or --older-than-days");
            }
            let report =
                aira_object::prune_csu_tenant_secret_backups(root, keep, older_than_days, dry_run)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            for (path, why) in &report.skipped {
                eprintln!("skip {}\t{why}", path.display());
            }
            let tag = if dry_run { "would_delete" } else { "deleted" };
            for p in &report.deleted {
                println!("{tag}\t{}", p.display());
            }
            println!(
                "prune tenant_deleted={} dry_run={dry_run}",
                report.deleted.len()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
