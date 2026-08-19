//! Tenant rotate / revoke ceremony (Analyze-83).

use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;

use crate::audit::{TrustAuditAction, TrustAuditEntry, TrustAuditLog};
use crate::crypto::{unregister_verifying, utc_now_rfc3339, CryptoError};
use crate::types::AiraRef;

use super::map::{
    publisher_owned_by_other_csu, register_csu_tenant_signing, unregister_csu_tenant,
};
use super::paths::{
    remove_path_quiet, tenant_dir, unix_stamp, CSU_TENANT_SECRET_BACKUP_FILE,
    CSU_TENANT_SECRET_BACKUP_META_FILE, CSU_TENANT_SECRET_FILE,
};
use super::persist::{commit_secret_then_meta, read_meta, write_secret_0600, CsuTenantMeta};

fn archive_latest_tenant_prev(dir: &Path) -> Result<(), CryptoError> {
    let latest = dir.join(CSU_TENANT_SECRET_BACKUP_FILE);
    if !latest.exists() {
        return Ok(());
    }
    let stamp = unix_stamp();
    let archived = dir.join(format!("{CSU_TENANT_SECRET_BACKUP_FILE}.{stamp}"));
    let meta = dir.join(CSU_TENANT_SECRET_BACKUP_META_FILE);
    let archived_meta = dir.join(format!("{CSU_TENANT_SECRET_BACKUP_FILE}.{stamp}.meta.json"));
    fs::rename(&latest, &archived).map_err(|e| CryptoError::Io(e.to_string()))?;
    if meta.exists() {
        let _ = fs::rename(&meta, &archived_meta);
    }
    Ok(())
}

/// Rotate durable tenant signing secret (same `publisher_id`).
///
/// Returns `(publisher, new_pub_hex, old_pub_hex, backup_path)`.
pub fn rotate_csu_tenant_signing(
    root: impl AsRef<Path>,
    csu_id: &AiraRef,
    new_signing: SigningKey,
    backup: bool,
) -> Result<(AiraRef, String, String, Option<PathBuf>), CryptoError> {
    let root = root.as_ref();
    let csu = csu_id.as_str().trim();
    let dir = tenant_dir(root, csu);
    if !dir.is_dir() {
        return Err(CryptoError::Io(format!(
            "csu tenant not found on disk: {}",
            dir.display()
        )));
    }
    let meta = read_meta(&dir)?;
    if meta.csu_id.trim() != csu {
        return Err(CryptoError::Io(format!(
            "csu tenant meta csu_id mismatch: {} vs {csu}",
            meta.csu_id
        )));
    }
    let publisher =
        AiraRef::parse(meta.publisher_id.trim()).map_err(|_| CryptoError::InvalidKey)?;
    let old_pub = meta.public_key_hex.trim().to_string();
    let old_secret_path = dir.join(CSU_TENANT_SECRET_FILE);
    let old_secret_raw = if old_secret_path.exists() {
        Some(fs::read_to_string(&old_secret_path).map_err(|e| CryptoError::Io(e.to_string()))?)
    } else {
        None
    };

    if let Some(other) = publisher_owned_by_other_csu(publisher.as_str(), csu) {
        return Err(CryptoError::TenantIsolation(format!(
            "publisher {} already bound to csu {other}",
            publisher.as_str()
        )));
    }

    let backup_path = dir.join(CSU_TENANT_SECRET_BACKUP_FILE);
    let backup_meta_path = dir.join(CSU_TENANT_SECRET_BACKUP_META_FILE);
    let backup_tmp = dir.join("ed25519.prev.tmp");
    let backup_meta_tmp = dir.join("ed25519.prev.meta.json.tmp");
    let mut wrote_backup: Option<PathBuf> = None;

    if backup {
        let secret = old_secret_raw.as_ref().ok_or_else(|| {
            CryptoError::Io("cannot backup: missing ed25519 before rotate".into())
        })?;
        remove_path_quiet(&backup_tmp);
        remove_path_quiet(&backup_meta_tmp);
        write_secret_0600(&backup_tmp, secret.as_bytes())?;
        let bmeta = serde_json::json!({
            "csu_id": csu,
            "publisher_id": publisher.as_str(),
            "old_public_key_hex": old_pub,
            "backed_up_at": utc_now_rfc3339()?,
        });
        let bmeta_out =
            serde_json::to_string_pretty(&bmeta).map_err(|e| CryptoError::Io(e.to_string()))?;
        fs::write(&backup_meta_tmp, format!("{bmeta_out}\n"))
            .map_err(|e| CryptoError::Io(e.to_string()))?;
        archive_latest_tenant_prev(&dir)?;
        fs::rename(&backup_tmp, &backup_path).map_err(|e| CryptoError::Io(e.to_string()))?;
        let _ = fs::rename(&backup_meta_tmp, &backup_meta_path);
        wrote_backup = Some(backup_path);
    }

    let new_pub = hex::encode(new_signing.verifying_key().to_bytes());
    let secret_hex = hex::encode(new_signing.to_bytes());
    let new_meta = CsuTenantMeta {
        csu_id: csu.to_string(),
        publisher_id: publisher.as_str().to_string(),
        public_key_hex: new_pub.clone(),
        created_at: meta.created_at.clone(),
        rotated_at: utc_now_rfc3339().ok(),
    };
    commit_secret_then_meta(&dir, &secret_hex, &new_meta)?;
    register_csu_tenant_signing(csu_id, publisher.clone(), new_signing)?;

    let audit = TrustAuditEntry::new(TrustAuditAction::TenantRotate, csu, Some("csu-tenant"))?
        .with_pubkey_hex(Some(new_pub.as_str()))
        .with_reason(Some("csu tenant signing rotated"));
    TrustAuditLog::append(root, &audit)?;

    Ok((publisher, new_pub, old_pub, wrote_backup))
}

/// Revoke durable tenant: unload map, drop verifying (if unshared), delete dir, audit.
///
/// `reason` must be non-empty. Does **not** touch TrustStore CRL (signing-side only).
pub fn revoke_csu_tenant_signing(
    root: impl AsRef<Path>,
    csu_id: &AiraRef,
    reason: &str,
) -> Result<(), CryptoError> {
    let root = root.as_ref();
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(CryptoError::Io(
            "csu tenant revoke requires a non-empty reason".into(),
        ));
    }
    let csu = csu_id.as_str().trim();
    let dir = tenant_dir(root, csu);
    if !dir.is_dir() {
        return Err(CryptoError::Io(format!(
            "csu tenant not found on disk: {}",
            dir.display()
        )));
    }
    let meta = read_meta(&dir).unwrap_or(CsuTenantMeta {
        csu_id: csu.to_string(),
        publisher_id: String::new(),
        public_key_hex: String::new(),
        created_at: None,
        rotated_at: None,
    });
    let publisher_id = meta.publisher_id.trim().to_string();

    unregister_csu_tenant(csu_id);
    if !publisher_id.is_empty() {
        let shared = publisher_owned_by_other_csu(&publisher_id, csu).is_some();
        if !shared {
            if let Ok(pub_ref) = AiraRef::parse(&publisher_id) {
                let _ = unregister_verifying(&pub_ref);
            }
        }
    }
    fs::remove_dir_all(&dir).map_err(|e| CryptoError::Io(e.to_string()))?;

    let audit = TrustAuditEntry::new(TrustAuditAction::TenantRevoke, csu, Some("csu-tenant"))?
        .with_reason(Some(reason))
        .with_pubkey_hex(Some(meta.public_key_hex.trim()).filter(|s| !s.is_empty()));
    TrustAuditLog::append(root, &audit)?;
    Ok(())
}
