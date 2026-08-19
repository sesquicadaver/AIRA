//! Node signing-secret rotate, backups, and prune (Analyze-82).

use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::AiraRef;

use super::error::{normalize_rfc3339, parse_rfc3339, utc_now_rfc3339, CryptoError};
use super::keyring::{register_keyring, set_primary_signer, sign_with_key, Keyring};
use super::trust_store::ensure_trust_defaults;

/// Relative path (under node `identity/`) for opt-in previous signing secret backup.
pub const NODE_SECRET_BACKUP_FILE: &str = "local.ed25519.prev";
/// Sidecar metadata for [`NODE_SECRET_BACKUP_FILE`] (pubkey + timestamp; never the secret).
pub const NODE_SECRET_BACKUP_META_FILE: &str = "local.ed25519.prev.meta.json";

const NODE_SECRET_BACKUP_TMP: &str = "local.ed25519.prev.tmp";
const NODE_SECRET_BACKUP_META_TMP: &str = "local.ed25519.prev.meta.json.tmp";

/// One listed node signing-secret backup (latest slot or archived timestamped slot).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSecretBackupInfo {
    /// `latest` or compact UTC stamp (`YYYYMMDDTHHMMSSZ`[+`-N`]).
    pub stamp: String,
    pub secret_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backed_up_at: Option<String>,
    /// True when this is the canonical [`NODE_SECRET_BACKUP_FILE`] slot.
    pub is_latest: bool,
}

fn compact_utc_stamp(rfc3339: &str) -> Result<String, CryptoError> {
    let dt = parse_rfc3339(rfc3339)?;
    Ok(format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        dt.year(),
        u8::from(dt.month()),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ))
}

fn backup_stamp_from_meta(meta_path: &Path) -> Result<String, CryptoError> {
    if meta_path.is_file() {
        let raw = fs::read_to_string(meta_path).map_err(|e| CryptoError::Io(e.to_string()))?;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(s) = v.get("backed_up_at").and_then(|x| x.as_str()) {
                if let Ok(stamp) = compact_utc_stamp(s) {
                    return Ok(stamp);
                }
            }
        }
    }
    compact_utc_stamp(&utc_now_rfc3339()?)
}

fn unique_archived_backup_stamp(identity_dir: &Path, base: &str) -> String {
    let mut stamp = base.to_string();
    let mut n = 2u32;
    loop {
        let candidate = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}"));
        if !candidate.exists() {
            return stamp;
        }
        stamp = format!("{base}-{n}");
        n += 1;
    }
}

/// Move the canonical `.prev` slot into a timestamped archive name (Analyze-41).
///
/// No-op when the latest slot is missing. On I/O failure returns `Err` without
/// deleting the latest slot.
fn archive_latest_prev_slot(identity_dir: &Path) -> Result<Option<PathBuf>, CryptoError> {
    let latest = identity_dir.join(NODE_SECRET_BACKUP_FILE);
    if !latest.is_file() {
        return Ok(None);
    }
    let meta = identity_dir.join(NODE_SECRET_BACKUP_META_FILE);
    let base = backup_stamp_from_meta(&meta)?;
    let stamp = unique_archived_backup_stamp(identity_dir, &base);
    let archived = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}"));
    let archived_meta = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}.meta.json"));

    fs::rename(&latest, &archived).map_err(|e| {
        CryptoError::Io(format!(
            "archive prev rename failed ({} → {}): {e}",
            latest.display(),
            archived.display()
        ))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&archived, fs::Permissions::from_mode(0o600));
    }

    if meta.is_file() {
        let raw = fs::read_to_string(&meta).map_err(|e| CryptoError::Io(e.to_string()))?;
        let mut v: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "secret_path".into(),
                serde_json::json!(format!("identity/{NODE_SECRET_BACKUP_FILE}.{stamp}")),
            );
            if let Ok(now) = utc_now_rfc3339() {
                obj.insert("archived_at".into(), serde_json::json!(now));
            }
            obj.insert("archive_stamp".into(), serde_json::json!(stamp));
        }
        let out = serde_json::to_string_pretty(&v).map_err(|e| CryptoError::Io(e.to_string()))?;
        fs::write(&archived_meta, format!("{out}\n"))
            .map_err(|e| CryptoError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&archived_meta, fs::Permissions::from_mode(0o600));
        }
        let _ = fs::remove_file(&meta);
    }

    Ok(Some(archived))
}

fn read_backup_meta_fields(meta_path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let Ok(raw) = fs::read_to_string(meta_path) else {
        return (None, None, None);
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (None, None, None);
    };
    (
        v.get("identity_id")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        v.get("old_public_key_hex")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        v.get("backed_up_at")
            .and_then(|x| x.as_str())
            .map(str::to_string),
    )
}

/// List durable node signing-secret backups (latest + archived timestamped slots).
///
/// Newest first. Does not read or return secret material.
pub fn list_node_secret_backups(
    root: impl AsRef<Path>,
) -> Result<Vec<NodeSecretBackupInfo>, CryptoError> {
    let identity_dir = root.as_ref().join("identity");
    if !identity_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let latest = identity_dir.join(NODE_SECRET_BACKUP_FILE);
    if latest.is_file() {
        let meta = identity_dir.join(NODE_SECRET_BACKUP_META_FILE);
        let (identity_id, old_public_key_hex, backed_up_at) = if meta.is_file() {
            read_backup_meta_fields(&meta)
        } else {
            (None, None, None)
        };
        out.push(NodeSecretBackupInfo {
            stamp: "latest".into(),
            secret_path: latest,
            meta_path: meta.is_file().then_some(meta),
            identity_id,
            old_public_key_hex,
            backed_up_at,
            is_latest: true,
        });
    }

    let prefix = format!("{NODE_SECRET_BACKUP_FILE}.");
    let rd = fs::read_dir(&identity_dir).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(&prefix) {
            continue;
        }
        if name.ends_with(".meta.json")
            || name.ends_with(".tmp")
            || name == NODE_SECRET_BACKUP_META_FILE
        {
            continue;
        }
        // Archived secret: local.ed25519.prev.<stamp>
        let stamp = name[prefix.len()..].to_string();
        if stamp.is_empty() || stamp.contains('.') {
            continue;
        }
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        let meta = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}.meta.json"));
        let (identity_id, old_public_key_hex, backed_up_at) = if meta.is_file() {
            read_backup_meta_fields(&meta)
        } else {
            (None, None, None)
        };
        out.push(NodeSecretBackupInfo {
            stamp,
            secret_path: path,
            meta_path: meta.is_file().then_some(meta),
            identity_id,
            old_public_key_hex,
            backed_up_at,
            is_latest: false,
        });
    }

    out.sort_by(|a, b| {
        // latest first, then stamp descending (lexicographic works for compact UTC).
        match (a.is_latest, b.is_latest) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.stamp.cmp(&a.stamp),
        }
    });
    Ok(out)
}

/// Result of pruning archived node signing-secret stamp slots (Analyze-61).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeSecretPruneReport {
    pub deleted: Vec<PathBuf>,
    /// Paths skipped with a human reason (unparseable age, orphan meta, …).
    pub skipped: Vec<(PathBuf, String)>,
    pub dry_run: bool,
}

fn unix_now_secs() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// Parse compact stamp `YYYYMMDDTHHMMSSZ` or `…-N` collision suffix → unix seconds.
fn compact_stamp_unix(stamp: &str) -> Option<i64> {
    let base = stamp.split('-').next().unwrap_or(stamp);
    if base.len() != 16 || !base.ends_with('Z') {
        return None;
    }
    let y: i32 = base.get(0..4)?.parse().ok()?;
    let mo: u8 = base.get(4..6)?.parse().ok()?;
    let d: u8 = base.get(6..8)?.parse().ok()?;
    let h: u8 = base.get(9..11)?.parse().ok()?;
    let mi: u8 = base.get(11..13)?.parse().ok()?;
    let s: u8 = base.get(13..15)?.parse().ok()?;
    if base.as_bytes().get(8) != Some(&b'T') {
        return None;
    }
    let date = time::Date::from_calendar_date(y, time::Month::try_from(mo).ok()?, d).ok()?;
    let time_ = time::Time::from_hms(h, mi, s).ok()?;
    Some(OffsetDateTime::new_utc(date, time_).unix_timestamp())
}

fn ed25519_archive_age_unix(info: &NodeSecretBackupInfo) -> Option<i64> {
    if let Some(ref at) = info.backed_up_at {
        if let Ok(dt) = parse_rfc3339(at) {
            return Some(dt.unix_timestamp());
        }
    }
    compact_stamp_unix(&info.stamp)
}

pub(crate) fn should_retain_archived(
    rank: u64,
    age_unix: Option<i64>,
    keep: Option<u64>,
    older_than_days: Option<u64>,
    now: i64,
) -> Result<bool, String> {
    let keep_ok = match keep {
        None => true,
        Some(n) => rank < n,
    };
    let age_ok = match older_than_days {
        None => true,
        Some(days) => {
            let age = age_unix.ok_or_else(|| "unparseable age".to_string())?;
            let limit = i64::try_from(days)
                .unwrap_or(i64::MAX)
                .saturating_mul(86_400);
            now.saturating_sub(age) <= limit
        }
    };
    Ok(keep_ok && age_ok)
}

/// Prune archived `local.ed25519.prev.<stamp>` slots (Analyze-61).
///
/// Never deletes the canonical latest `.prev` / `.prev.meta.json`.
/// Requires at least one of `keep` / `older_than_days`.
/// Retain = intersection of supplied policies among archived slots (newest rank 0).
pub fn prune_node_secret_backups(
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
    let identity_dir = root.as_ref().join("identity");
    let mut report = NodeSecretPruneReport {
        dry_run,
        ..Default::default()
    };

    // Orphan meta: never delete.
    if identity_dir.is_dir() {
        let prefix = format!("{NODE_SECRET_BACKUP_FILE}.");
        let rd = fs::read_dir(&identity_dir).map_err(|e| CryptoError::Io(e.to_string()))?;
        for ent in rd {
            let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
            let name = ent.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with(&prefix) || !name.ends_with(".meta.json") {
                continue;
            }
            // Canonical latest meta is `local.ed25519.prev.meta.json` (not a stamp archive).
            if name.as_ref() == NODE_SECRET_BACKUP_META_FILE {
                continue;
            }
            let end = name.len().saturating_sub(".meta.json".len());
            if prefix.len() >= end {
                continue;
            }
            // local.ed25519.prev.<stamp>.meta.json
            let mid = &name[prefix.len()..end];
            if mid.is_empty() || mid.contains('.') {
                continue;
            }
            let secret = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{mid}"));
            if !secret.is_file() {
                report.skipped.push((ent.path(), "orphan-meta".into()));
            }
        }
    }

    let list = list_node_secret_backups(&root)?;
    let mut archived: Vec<_> = list.into_iter().filter(|b| !b.is_latest).collect();
    archived.sort_by(|a, b| b.stamp.cmp(&a.stamp));
    let now = unix_now_secs();

    for (rank, info) in archived.into_iter().enumerate() {
        let rank = rank as u64;
        let age = ed25519_archive_age_unix(&info);
        match should_retain_archived(rank, age, keep, older_than_days, now) {
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
    Ok(report)
}

fn remove_path_quiet(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(path);
}

fn clear_staging_files(tmp: &Path, meta_tmp: &Path) {
    // Only regular leftover files from a crashed attempt — keep a directory trap so write fails.
    let _ = fs::remove_file(tmp);
    let _ = fs::remove_file(meta_tmp);
}

/// Rotate the node signing secret under fixed paths, keeping the same `identity_id`.
///
/// Rewrites `identity/local.ed25519` and updates `identity/local.identity.json` public key +
/// descriptor signature. Trust store gets an upsert (no CRL).
///
/// - Without `grace_until`: immediate cutover — previous verifying key for this id is dropped.
/// - With `grace_until` (RFC3339 UTC): persists `previous_public_key` + `previous_grace_until`
///   so old signatures under the same `key_ref` still verify until that instant (Analyze-37).
///
/// If trust upsert fails after the files were rewritten, previous secret + JSON are restored
/// so disk and trust stay consistent.
///
/// When `backup` is true, stages the previous secret under `*.tmp` (unix mode `0600`) **before**
/// overwrite and renames to `identity/local.ed25519.prev` (+ meta sidecar) only after a successful
/// rotate. If a prior `.prev` already exists, it is archived to
/// `local.ed25519.prev.<YYYYMMDDTHHMMSSZ>` (+ matching meta) before the new latest slot is committed
/// (Analyze-41). Staging/I/O failure aborts without changing the active secret; abort after staging
/// removes only tmp files (existing `.prev` / history slots are left intact).
///
/// Returns `(identity_id, new_public_key_hex, old_public_key_hex, backup_path)`.
pub fn rotate_node_signing_secret(
    root: impl AsRef<Path>,
    new_signing: SigningKey,
    backup: bool,
    grace_until: Option<&str>,
) -> Result<(AiraRef, String, String, Option<PathBuf>), CryptoError> {
    let root = root.as_ref();
    let identity_dir = root.join("identity");
    let json_path = identity_dir.join("local.identity.json");
    let key_path = identity_dir.join("local.ed25519");
    let backup_path = identity_dir.join(NODE_SECRET_BACKUP_FILE);
    let backup_meta_path = identity_dir.join(NODE_SECRET_BACKUP_META_FILE);
    let backup_tmp = identity_dir.join(NODE_SECRET_BACKUP_TMP);
    let backup_meta_tmp = identity_dir.join(NODE_SECRET_BACKUP_META_TMP);
    if !json_path.exists() {
        return Err(CryptoError::Io(format!(
            "missing {} — run `aira identity create` first",
            json_path.display()
        )));
    }
    // Fail closed if current material is inconsistent before overwrite.
    let (id, old_ring) = Keyring::load_node_identity(root)?;
    let old_json = fs::read_to_string(&json_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let old_secret = if key_path.exists() {
        Some(fs::read_to_string(&key_path).map_err(|e| CryptoError::Io(e.to_string()))?)
    } else {
        None
    };
    let mut desc: serde_json::Value =
        serde_json::from_str(&old_json).map_err(|e| CryptoError::Io(e.to_string()))?;
    let old_pub = desc
        .get("public_key")
        .and_then(|p| p.get("key_hex"))
        .and_then(|v| v.as_str())
        .ok_or(CryptoError::InvalidKey)?
        .trim()
        .to_string();

    let grace_until = match grace_until {
        Some(s) => Some(normalize_rfc3339(s)?),
        None => None,
    };

    let new_pub = hex::encode(new_signing.verifying_key().to_bytes());
    let secret_hex = hex::encode(new_signing.to_bytes());
    fs::create_dir_all(&identity_dir).map_err(|e| CryptoError::Io(e.to_string()))?;

    let cleanup_staging = || {
        remove_path_quiet(&backup_tmp);
        remove_path_quiet(&backup_meta_tmp);
    };

    let mut staged_backup = false;
    if backup {
        let secret = old_secret.as_ref().ok_or_else(|| {
            CryptoError::Io("cannot backup: missing identity/local.ed25519 before rotate".into())
        })?;
        // Drop leftover staging *files* from a previous crash (not directory traps).
        clear_staging_files(&backup_tmp, &backup_meta_tmp);
        if let Err(e) = fs::write(&backup_tmp, secret) {
            cleanup_staging();
            return Err(CryptoError::Io(format!(
                "backup stage failed ({}): {e} — rotate aborted",
                backup_tmp.display()
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(&backup_tmp, fs::Permissions::from_mode(0o600)) {
                cleanup_staging();
                return Err(CryptoError::Io(format!(
                    "backup stage chmod failed ({}): {e} — rotate aborted",
                    backup_tmp.display()
                )));
            }
        }
        let meta = serde_json::json!({
            "identity_id": id.as_str(),
            "old_public_key_hex": old_pub,
            "backed_up_at": match utc_now_rfc3339() {
                Ok(t) => t,
                Err(e) => {
                    cleanup_staging();
                    return Err(e);
                }
            },
            "secret_path": format!("identity/{NODE_SECRET_BACKUP_FILE}"),
        });
        let meta_out = match serde_json::to_string_pretty(&meta) {
            Ok(s) => s,
            Err(e) => {
                cleanup_staging();
                return Err(CryptoError::Io(e.to_string()));
            }
        };
        if let Err(e) = fs::write(&backup_meta_tmp, format!("{meta_out}\n")) {
            cleanup_staging();
            return Err(CryptoError::Io(format!(
                "backup meta stage failed ({}): {e} — rotate aborted",
                backup_meta_tmp.display()
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(&backup_meta_tmp, fs::Permissions::from_mode(0o600))
            {
                cleanup_staging();
                return Err(CryptoError::Io(format!(
                    "backup meta chmod failed ({}): {e} — rotate aborted",
                    backup_meta_tmp.display()
                )));
            }
        }
        staged_backup = true;
    }

    let restore_previous = || -> Result<(), CryptoError> {
        if let Some(secret) = &old_secret {
            fs::write(&key_path, secret).map_err(|e| CryptoError::Io(e.to_string()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
            }
        }
        fs::write(&json_path, &old_json).map_err(|e| CryptoError::Io(e.to_string()))?;
        register_keyring(&old_ring);
        set_primary_signer(id.clone());
        Ok(())
    };

    let abort_after_stage = |err: CryptoError| -> CryptoError {
        cleanup_staging();
        err
    };

    if let Err(e) = fs::write(&key_path, format!("{secret_hex}\n")) {
        return Err(abort_after_stage(CryptoError::Io(e.to_string())));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
    }

    let identity_id = id.as_str().to_string();
    let sig = sign_with_key(id.clone(), &new_signing, identity_id.as_bytes());
    if let Some(obj) = desc.as_object_mut() {
        obj.insert(
            "public_key".into(),
            serde_json::json!({
                "algorithm": "ed25519",
                "key_hex": new_pub
            }),
        );
        obj.insert(
            "signature".into(),
            serde_json::to_value(&sig).map_err(|e| {
                let _ = restore_previous();
                abort_after_stage(CryptoError::Io(e.to_string()))
            })?,
        );
        obj.insert(
            "key_path".into(),
            serde_json::json!("identity/local.ed25519"),
        );
        let rotated_at = match utc_now_rfc3339() {
            Ok(t) => t,
            Err(e) => {
                let _ = restore_previous();
                return Err(abort_after_stage(e));
            }
        };
        obj.insert("rotated_at".into(), serde_json::json!(rotated_at));
        if let Some(until) = grace_until.as_deref() {
            obj.insert(
                "previous_public_key".into(),
                serde_json::json!({
                    "algorithm": "ed25519",
                    "key_hex": old_pub
                }),
            );
            obj.insert("previous_grace_until".into(), serde_json::json!(until));
        } else {
            obj.remove("previous_public_key");
            obj.remove("previous_grace_until");
        }
    } else {
        let _ = restore_previous();
        return Err(abort_after_stage(CryptoError::InvalidKey));
    }
    let out = match serde_json::to_string_pretty(&desc) {
        Ok(s) => s,
        Err(e) => {
            let _ = restore_previous();
            return Err(abort_after_stage(CryptoError::Io(e.to_string())));
        }
    };
    if let Err(e) = fs::write(&json_path, format!("{out}\n")) {
        let _ = restore_previous();
        return Err(abort_after_stage(CryptoError::Io(e.to_string())));
    }

    if let Err(e) = ensure_trust_defaults(root) {
        let _ = restore_previous();
        return Err(abort_after_stage(e));
    }

    let mut wrote_backup: Option<PathBuf> = None;
    if staged_backup {
        // Destination must be a replaceable file path (not a directory trap).
        if backup_path.is_dir() {
            remove_path_quiet(&backup_path);
        }
        // Archive prior latest into timestamped history before committing the new latest.
        // On archive failure: leave staging tmp + existing `.prev` (never destroy history).
        let archive_ok = match archive_latest_prev_slot(&identity_dir) {
            Ok(_) => true,
            Err(_) => {
                wrote_backup = Some(backup_tmp.clone());
                false
            }
        };
        if archive_ok {
            match fs::rename(&backup_tmp, &backup_path) {
                Ok(()) => {
                    if backup_meta_path.is_dir() {
                        remove_path_quiet(&backup_meta_path);
                    }
                    if fs::rename(&backup_meta_tmp, &backup_meta_path).is_err() {
                        let _ = fs::copy(&backup_meta_tmp, &backup_meta_path);
                        remove_path_quiet(&backup_meta_tmp);
                    }
                    wrote_backup = Some(backup_path);
                }
                Err(_) => {
                    // Crypto + trust already committed — never restore_previous here.
                    // Leave staging tmp so the previous secret remains recoverable.
                    wrote_backup = Some(backup_tmp);
                }
            }
        }
    }

    // Reload so dual-key grace (if any) is registered for the same key_ref.
    let (id, ring) = Keyring::load_node_identity(root)?;
    register_keyring(&ring);
    set_primary_signer(id.clone());

    // Durable ceremony audit (pubkey only — never the secret).
    let audit = crate::audit::TrustAuditEntry::new(
        crate::audit::TrustAuditAction::NodeRotate,
        id.as_str(),
        Some("node-rotate"),
    )?
    .with_pubkey_hex(Some(new_pub.as_str()))
    .with_grace_until(grace_until.as_deref())
    .with_reason(Some("node signing secret rotated"));
    crate::audit::TrustAuditLog::append(root, &audit)?;

    Ok((id, new_pub, old_pub, wrote_backup))
}
