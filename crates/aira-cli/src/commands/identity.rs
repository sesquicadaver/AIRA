//! Identity create/rotate/sign/verify/backups (Analyze-81).

use std::path::Path;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use aira_flow::NodePaths;

use crate::cli::{BackupsCommands, IdentityCommands};
use crate::commands::{tenant, trust};
use crate::support::ensure_init;

pub(crate) fn run(root: &Path, command: IdentityCommands) -> Result<ExitCode> {
    match command {
        IdentityCommands::Create { name } => {
            ensure_init(root)?;
            let paths = NodePaths::new(root);
            let mut rng = OsRng;
            let signing = SigningKey::generate(&mut rng);
            let verifying: VerifyingKey = signing.verifying_key();
            let secret_hex = hex::encode(signing.to_bytes());
            let public_hex = hex::encode(verifying.to_bytes());
            std::fs::create_dir_all(paths.identity_dir())?;
            std::fs::write(paths.identity_key(), format!("{secret_hex}\n"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    paths.identity_key(),
                    std::fs::Permissions::from_mode(0o600),
                );
            }
            let identity_id = format!("aira:identity:{name}");
            let id_ref =
                aira_object::AiraRef::parse(&identity_id).map_err(|e| anyhow::anyhow!("{e}"))?;
            let sig = aira_object::sign_with_key(id_ref.clone(), &signing, identity_id.as_bytes());
            let desc = serde_json::json!({
                "identity_id": identity_id,
                "identity_type": "local",
                "display_name": name,
                "public_key": {
                    "algorithm": "ed25519",
                    "key_hex": public_hex
                },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519",
                "signature": sig
            });
            std::fs::write(paths.identity_json(), serde_json::to_string_pretty(&desc)?)?;
            let mut ring = aira_object::Keyring::with_local_test();
            ring.insert_signing(id_ref.clone(), signing);
            aira_object::register_keyring(&ring);
            aira_object::set_primary_signer(id_ref);
            let _ = aira_object::ensure_trust_defaults(root);
            println!("created {identity_id}");
            println!("public_key {public_hex}");
            println!("identity {}", paths.identity_json().display());
            Ok(ExitCode::SUCCESS)
        }
        IdentityCommands::Rotate {
            backup,
            until,
            notify_peers,
        } => {
            ensure_init(root)?;
            let mut rng = OsRng;
            let signing = SigningKey::generate(&mut rng);
            let new_pub = hex::encode(signing.verifying_key().to_bytes());
            // Notify *before* cutover so peers can still verify hello with the old pubkey.
            if notify_peers {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .context("tokio runtime")?;
                let results = rt
                    .block_on(aira_peer::notify_peers_of_rekey(
                        root,
                        &new_pub,
                        until.as_deref(),
                    ))
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                if results.is_empty() {
                    println!("notify_peers (empty address book)");
                } else {
                    for r in results {
                        if r.ok {
                            println!("notified {}", r.peer_id);
                        } else {
                            eprintln!(
                                "notify failed {}\t{}",
                                r.peer_id,
                                r.error.unwrap_or_default()
                            );
                        }
                    }
                }
            }
            let (id, reported_new, old_pub, backup_path) =
                aira_object::rotate_node_signing_secret(root, signing, backup, until.as_deref())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            let noise = aira_peer::rotate_noise_static(root, backup)
                .map_err(|e| anyhow::anyhow!("x25519 rotate: {e}"))?;
            println!("rotated {}", id.as_str());
            println!("old_public_key {old_pub}");
            println!("public_key {reported_new}");
            if let Some(until) = until.as_deref() {
                println!("grace_until {until}");
            }
            if let Some(path) = backup_path {
                println!("backup {}", path.display());
            }
            if let Some(ref old_x) = noise.old_public_hex {
                println!("x25519_old_public_key {old_x}");
            }
            println!("x25519_public_key {}", noise.new_public_hex);
            if let Some(path) = noise.backup_path {
                println!("x25519_backup {}", path.display());
            }
            println!(
                "identity {}",
                NodePaths::new(root).identity_json().display()
            );
            Ok(ExitCode::SUCCESS)
        }
        IdentityCommands::Backups { command: None } => {
            ensure_init(root)?;
            let list =
                aira_object::list_node_secret_backups(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let xlist =
                aira_peer::list_noise_static_backups(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if list.is_empty() && xlist.is_empty() {
                println!("(no backups — use `identity rotate --backup`)");
            } else {
                for b in &list {
                    let pk = b.old_public_key_hex.as_deref().unwrap_or("-");
                    let at = b.backed_up_at.as_deref().unwrap_or("-");
                    println!(
                        "ed25519\t{}\t{}\t{}\t{}",
                        b.stamp,
                        pk,
                        at,
                        b.secret_path.display()
                    );
                }
                for b in &xlist {
                    println!("x25519\t{}\t-\t-\t{}", b.stamp, b.secret_path.display());
                }
            }
            println!("backups {}", NodePaths::new(root).identity_dir().display());
            Ok(ExitCode::SUCCESS)
        }
        IdentityCommands::Backups {
            command:
                Some(BackupsCommands::Prune {
                    keep,
                    older_than_days,
                    dry_run,
                }),
        } => {
            ensure_init(root)?;
            if keep.is_none() && older_than_days.is_none() {
                bail!("backups prune requires --keep and/or --older-than-days");
            }
            let ed = aira_object::prune_node_secret_backups(root, keep, older_than_days, dry_run)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let x = aira_peer::prune_noise_static_backups(root, keep, older_than_days, dry_run)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            for (path, why) in ed.skipped.iter().chain(x.skipped.iter()) {
                eprintln!("skip {}\t{why}", path.display());
            }
            let tag = if dry_run { "would_delete" } else { "deleted" };
            for p in ed.deleted.iter().chain(x.deleted.iter()) {
                println!("{tag}\t{}", p.display());
            }
            println!(
                "prune ed25519_deleted={} x25519_deleted={} dry_run={dry_run}",
                ed.deleted.len(),
                x.deleted.len()
            );
            Ok(ExitCode::SUCCESS)
        }
        IdentityCommands::Sign { text } => {
            ensure_init(root)?;
            let id = aira_object::register_node_identity(root)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .ok_or_else(|| anyhow::anyhow!("no identity — run `aira identity create` first"))?;
            let ring = aira_object::process_keyring_snapshot();
            let sig = ring
                .sign(&id, text.as_bytes())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{}", serde_json::to_string_pretty(&sig)?);
            Ok(ExitCode::SUCCESS)
        }
        IdentityCommands::Verify {
            text,
            signature,
            key_ref,
        } => {
            ensure_init(root)?;
            let node_id =
                aira_object::register_node_identity(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            let key_ref = match key_ref {
                Some(k) => aira_object::AiraRef::parse(k).map_err(|e| anyhow::anyhow!("{e}"))?,
                None => node_id.unwrap_or_else(|| {
                    aira_object::AiraRef::parse(aira_object::LOCAL_TEST_KEY_REF).unwrap()
                }),
            };
            let sig = aira_object::Signature {
                algorithm: "ed25519".into(),
                key_ref,
                signature_value: signature,
            };
            match aira_object::verify_ed25519(&sig, text.as_bytes()) {
                Ok(()) => {
                    println!("OK: signature valid");
                    Ok(ExitCode::SUCCESS)
                }
                Err(e) => {
                    eprintln!("FAIL: {e}");
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        IdentityCommands::Trust { command } => trust::run(root, command),
        IdentityCommands::CsuTenant { command } => tenant::run(root, command),
    }
}
