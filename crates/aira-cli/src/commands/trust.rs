//! Identity trust subcommands (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use aira_flow::NodePaths;
use anyhow::{bail, Result};

use crate::cli::TrustCommands;
use crate::support::ensure_init;

pub(crate) fn run(root: &Path, command: TrustCommands) -> Result<ExitCode> {
    match command {
        TrustCommands::List => {
            ensure_init(root)?;
            let store =
                aira_object::ensure_trust_defaults(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if store.entries.is_empty() {
                println!("(empty trusted)");
            } else {
                for e in &store.entries {
                    println!("{}\t{}\t{}", e.identity_id, e.algorithm, e.public_key_hex);
                }
            }
            if !store.revoked.is_empty() {
                println!("# revoked");
                for r in &store.revoked {
                    let reason = r.reason.as_deref().unwrap_or("-");
                    println!("REVOKED\t{}\t{}", r.identity_id, reason);
                }
            }
            println!("trust {}", NodePaths::new(root).trust_json().display());
            Ok(ExitCode::SUCCESS)
        }
        TrustCommands::Add {
            key_ref,
            pubkey_hex,
        } => {
            ensure_init(root)?;
            let mut store =
                aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            store
                .upsert(&key_ref, &pubkey_hex)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            store.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            aira_object::register_trust_store(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("trusted {key_ref}");
            Ok(ExitCode::SUCCESS)
        }
        TrustCommands::Remove { key_ref } => {
            ensure_init(root)?;
            let mut store =
                aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if key_ref == aira_object::LOCAL_TEST_KEY_REF {
                bail!("refusing to remove local-test from trust store");
            }
            if store.remove(&key_ref) {
                store.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
                aira_object::sync_trust_verifiers(root).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("removed {key_ref}");
            } else {
                println!("not found {key_ref}");
            }
            Ok(ExitCode::SUCCESS)
        }
        TrustCommands::Revoke { key_ref, reason } => {
            ensure_init(root)?;
            let mut store =
                aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            store
                .revoke(&key_ref, reason.as_deref())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            store.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            aira_object::sync_trust_verifiers(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let audit = aira_object::TrustAuditEntry::new(
                aira_object::TrustAuditAction::Revoke,
                &key_ref,
                Some("cli"),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .with_reason(reason.as_deref());
            aira_object::TrustAuditLog::append(root, &audit).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("revoked {key_ref}");
            Ok(ExitCode::SUCCESS)
        }
        TrustCommands::Unrevoke { key_ref } => {
            ensure_init(root)?;
            let mut store =
                aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            store
                .unrevoke(&key_ref)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            store.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let audit = aira_object::TrustAuditEntry::new(
                aira_object::TrustAuditAction::Unrevoke,
                &key_ref,
                Some("cli"),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            aira_object::TrustAuditLog::append(root, &audit).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("unrevoked {key_ref} (not trusted until `trust add`)");
            Ok(ExitCode::SUCCESS)
        }
        TrustCommands::Rotate {
            old_key_ref,
            new_key_ref,
            pubkey_hex,
            reason,
            until,
        } => {
            ensure_init(root)?;
            let mut store =
                aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            store
                .rotate(
                    &old_key_ref,
                    &new_key_ref,
                    &pubkey_hex,
                    reason.as_deref(),
                    until.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            store.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            aira_object::sync_trust_verifiers(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let audit = aira_object::TrustAuditEntry::new(
                aira_object::TrustAuditAction::Rotate,
                &old_key_ref,
                Some("cli"),
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .with_new_id(Some(&new_key_ref))
            .with_pubkey_hex(Some(&pubkey_hex))
            .with_grace_until(until.as_deref())
            .with_reason(reason.as_deref());
            aira_object::TrustAuditLog::append(root, &audit).map_err(|e| anyhow::anyhow!("{e}"))?;
            match until {
                Some(u) => {
                    println!("rotated {old_key_ref} -> {new_key_ref} (grace until {u})")
                }
                None => println!("rotated {old_key_ref} -> {new_key_ref}"),
            }
            Ok(ExitCode::SUCCESS)
        }
        TrustCommands::Audit { last } => {
            ensure_init(root)?;
            let entries =
                aira_object::TrustAuditLog::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let slice: &[aira_object::TrustAuditEntry] = match last {
                Some(n) if n < entries.len() => &entries[entries.len() - n..],
                _ => &entries,
            };
            if slice.is_empty() {
                println!("(empty audit)");
            } else {
                for e in slice {
                    let reason = e.reason.as_deref().unwrap_or("-");
                    let new_id = e.new_id.as_deref().unwrap_or("-");
                    let source = e.source.as_deref().unwrap_or("-");
                    println!(
                        "{}\t{}\t{}\t{}\t{}\t{}",
                        e.recorded_at,
                        e.action.as_str(),
                        e.subject_id,
                        new_id,
                        reason,
                        source
                    );
                }
            }
            println!(
                "audit {}",
                NodePaths::new(root).trust_audit_jsonl().display()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}
