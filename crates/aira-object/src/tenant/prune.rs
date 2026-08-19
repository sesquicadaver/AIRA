//! Tenant backup list and prune (Analyze-83).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::crypto::{CryptoError, NodeSecretPruneReport};

use super::paths::{
    decode_csu_dir_name, tenants_root, CSU_TENANT_SECRET_BACKUP_FILE,
    CSU_TENANT_SECRET_BACKUP_META_FILE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsuTenantBackupInfo {
    pub csu_id: String,
    /// `latest` or filename stamp (`<unix-secs>` optional `-<n>`).
    pub stamp: String,
    pub secret_path: PathBuf,
    pub meta_path: Option<PathBuf>,
    pub old_public_key_hex: Option<String>,
    pub backed_up_at: Option<String>,
    pub is_latest: bool,
}

fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn read_tenant_backup_meta(meta_path: &Path) -> (Option<String>, Option<String>) {
    let Ok(raw) = fs::read_to_string(meta_path) else {
        return (None, None);
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (None, None);
    };
    (
        v.get("old_public_key_hex")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        v.get("backed_up_at")
            .and_then(|x| x.as_str())
            .map(str::to_string),
    )
}

/// Larger key = newer. Numeric unix stamps outrank non-numeric (non-numeric sort oldest).
fn stamp_sort_key(stamp: &str) -> (u8, u64, u32) {
    let mut parts = stamp.split('-');
    let base = parts.next().unwrap_or(stamp);
    let suffix: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    match base.parse::<u64>() {
        Ok(n) => (1, n, suffix),
        Err(_) => (0, 0, suffix),
    }
}

fn tenant_stamp_unix(stamp: &str) -> Option<i64> {
    let base = stamp.split('-').next().unwrap_or(stamp);
    base.parse::<i64>().ok().filter(|&n| n >= 0)
}

fn tenant_archive_age(info: &CsuTenantBackupInfo) -> Option<i64> {
    if let Some(ref at) = info.backed_up_at {
        if let Ok(dt) = OffsetDateTime::parse(at.trim(), &Rfc3339) {
            return Some(dt.unix_timestamp());
        }
    }
    tenant_stamp_unix(&info.stamp)
}

/// Archived secret `ed25519.prev.<stamp>` (not latest, not meta/tmp).
fn archived_prev_stamp(name: &str) -> Option<&str> {
    let prefix = format!("{CSU_TENANT_SECRET_BACKUP_FILE}.");
    if !name.starts_with(&prefix) {
        return None;
    }
    if name.ends_with(".meta.json") || name.ends_with(".tmp") {
        return None;
    }
    if name == CSU_TENANT_SECRET_BACKUP_META_FILE {
        return None;
    }
    let stamp = name.get(prefix.len()..)?;
    if stamp.is_empty() || stamp.contains('.') {
        return None;
    }
    Some(stamp)
}

fn list_one_tenant_backups(
    csu_id: &str,
    dir: &Path,
) -> Result<Vec<CsuTenantBackupInfo>, CryptoError> {
    let mut out = Vec::new();
    let latest = dir.join(CSU_TENANT_SECRET_BACKUP_FILE);
    if latest.is_file() {
        let meta = dir.join(CSU_TENANT_SECRET_BACKUP_META_FILE);
        let (old_public_key_hex, backed_up_at) = if meta.is_file() {
            read_tenant_backup_meta(&meta)
        } else {
            (None, None)
        };
        out.push(CsuTenantBackupInfo {
            csu_id: csu_id.to_string(),
            stamp: "latest".into(),
            secret_path: latest,
            meta_path: meta.is_file().then_some(meta),
            old_public_key_hex,
            backed_up_at,
            is_latest: true,
        });
    }

    if dir.is_dir() {
        let rd = fs::read_dir(dir).map_err(|e| CryptoError::Io(e.to_string()))?;
        for ent in rd {
            let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
            let name = ent.file_name();
            let name = name.to_string_lossy();
            let Some(stamp) = archived_prev_stamp(name.as_ref()) else {
                continue;
            };
            let path = ent.path();
            if !path.is_file() {
                continue;
            }
            let meta = dir.join(format!("{CSU_TENANT_SECRET_BACKUP_FILE}.{stamp}.meta.json"));
            let (old_public_key_hex, backed_up_at) = if meta.is_file() {
                read_tenant_backup_meta(&meta)
            } else {
                (None, None)
            };
            out.push(CsuTenantBackupInfo {
                csu_id: csu_id.to_string(),
                stamp: stamp.to_string(),
                secret_path: path,
                meta_path: meta.is_file().then_some(meta),
                old_public_key_hex,
                backed_up_at,
                is_latest: false,
            });
        }
    }

    out.sort_by(|a, b| match (a.is_latest, b.is_latest) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => stamp_sort_key(&b.stamp).cmp(&stamp_sort_key(&a.stamp)),
    });
    Ok(out)
}

/// List durable tenant backups (latest + archived stamps). Newest first per tenant.
///
/// `csu_id` comes from the directory name, not `meta.json`. Does not read secrets.
pub fn list_csu_tenant_secret_backups(
    root: impl AsRef<Path>,
) -> Result<Vec<CsuTenantBackupInfo>, CryptoError> {
    let base = tenants_root(root.as_ref());
    if !base.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    let rd = fs::read_dir(&base).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        let dir = ent.path();
        if !dir.is_dir() {
            continue;
        }
        let enc = ent.file_name();
        let Ok(csu_id) = decode_csu_dir_name(&enc.to_string_lossy()) else {
            continue;
        };
        dirs.push((csu_id, dir));
    }
    dirs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Vec::new();
    for (csu_id, dir) in dirs {
        out.extend(list_one_tenant_backups(&csu_id, &dir)?);
    }
    Ok(out)
}

fn scan_orphan_tenant_meta(
    dir: &Path,
    report: &mut NodeSecretPruneReport,
) -> Result<(), CryptoError> {
    let prefix = format!("{CSU_TENANT_SECRET_BACKUP_FILE}.");
    let rd = fs::read_dir(dir).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(&prefix) || !name.ends_with(".meta.json") {
            continue;
        }
        if name.as_ref() == CSU_TENANT_SECRET_BACKUP_META_FILE {
            continue;
        }
        let end = name.len().saturating_sub(".meta.json".len());
        if prefix.len() >= end {
            continue;
        }
        let mid = &name[prefix.len()..end];
        if mid.is_empty() || mid.contains('.') {
            continue;
        }
        let secret = dir.join(format!("{CSU_TENANT_SECRET_BACKUP_FILE}.{mid}"));
        if !secret.is_file() {
            report.skipped.push((ent.path(), "orphan-meta".into()));
        }
    }
    Ok(())
}

/// Prune archived `ed25519.prev.<stamp>` slots per tenant (Analyze-71).
///
/// Never deletes latest `.prev` / live `ed25519`. Requires `--keep` and/or `--older-than-days`.
/// Retain = intersection of policies **per tenant dir** (newest archived rank 0).
pub fn prune_csu_tenant_secret_backups(
    root: impl AsRef<Path>,
    keep: Option<u64>,
    older_than_days: Option<u64>,
    dry_run: bool,
) -> Result<NodeSecretPruneReport, CryptoError> {
    if keep.is_none() && older_than_days.is_none() {
        return Err(CryptoError::Io(
            "prune requires --keep and/or --older-than-days".into(),
        ));
    }
    let root = root.as_ref();
    let base = tenants_root(root);
    let mut report = NodeSecretPruneReport {
        dry_run,
        ..Default::default()
    };
    if !base.is_dir() {
        return Ok(report);
    }

    let mut decoded = Vec::new();
    let rd = fs::read_dir(&base).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        let dir = ent.path();
        if !dir.is_dir() {
            continue;
        }
        let enc = ent.file_name();
        match decode_csu_dir_name(&enc.to_string_lossy()) {
            Ok(csu_id) => decoded.push((csu_id, dir)),
            Err(_) => report.skipped.push((dir, "undecodable-dir".into())),
        }
    }

    let now = unix_now_secs();
    for (csu_id, dir) in decoded {
        scan_orphan_tenant_meta(&dir, &mut report)?;
        let list = list_one_tenant_backups(&csu_id, &dir)?;
        let mut archived: Vec<_> = list.into_iter().filter(|b| !b.is_latest).collect();
        archived.sort_by_key(|b| std::cmp::Reverse(stamp_sort_key(&b.stamp)));
        for (rank, info) in archived.into_iter().enumerate() {
            let rank = rank as u64;
            let age = tenant_archive_age(&info);
            match crate::crypto::should_retain_archived(rank, age, keep, older_than_days, now) {
                Ok(true) => {}
                Ok(false) => {
                    if dry_run {
                        report.deleted.push(info.secret_path.clone());
                        if let Some(ref m) = info.meta_path {
                            report.deleted.push(m.clone());
                        }
                    } else {
                        fs::remove_file(&info.secret_path).map_err(|e| {
                            CryptoError::Io(format!("prune {}: {e}", info.secret_path.display()))
                        })?;
                        report.deleted.push(info.secret_path.clone());
                        if let Some(ref m) = info.meta_path {
                            if m.is_file() {
                                fs::remove_file(m).map_err(|e| {
                                    CryptoError::Io(format!("prune meta {}: {e}", m.display()))
                                })?;
                                report.deleted.push(m.clone());
                            }
                        }
                    }
                }
                Err(reason) => {
                    report.skipped.push((info.secret_path.clone(), reason));
                }
            }
        }
    }
    Ok(report)
}
