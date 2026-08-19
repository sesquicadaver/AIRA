//! Per-CSU tenant signing isolation (Analyze-42) + durable secrets (Analyze-62)
//! + rotate/revoke ceremony (Analyze-63) + backup prune (Analyze-71).
//!
//! Signing secrets for CSU publishers live in a process map keyed by `csu_id`.
//! Only verifying keys are merged into the process [`Keyring`] (public material).
//! Durable layout: `identity/tenants/<hex(csu_id)>/{ed25519,meta.json}` (mode `0600`).
//! Invariant: one `publisher_id` maps to at most one CSU in the process tenant map.
//! CSU emit helpers must use [`signature_for_tenant`].

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::audit::{TrustAuditAction, TrustAuditEntry, TrustAuditLog};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::crypto::{
    primary_signer, register_keyring, sign_with_key, signature_for, unregister_verifying,
    utc_now_rfc3339, CryptoError, Keyring, NodeSecretPruneReport, LOCAL_TEST_KEY_REF,
};
use crate::types::{AiraRef, Signature};

/// Relative tenants directory under node `identity/`.
pub const CSU_TENANTS_DIR: &str = "tenants";
/// Secret filename inside a tenant dir.
pub const CSU_TENANT_SECRET_FILE: &str = "ed25519";
/// Metadata sidecar (never contains the secret).
pub const CSU_TENANT_META_FILE: &str = "meta.json";
/// Latest previous secret after rotate `--backup`.
pub const CSU_TENANT_SECRET_BACKUP_FILE: &str = "ed25519.prev";
/// Sidecar for [`CSU_TENANT_SECRET_BACKUP_FILE`] (pubkey only).
pub const CSU_TENANT_SECRET_BACKUP_META_FILE: &str = "ed25519.prev.meta.json";

struct TenantEntry {
    publisher_id: String,
    signing: SigningKey,
}

fn tenants() -> &'static RwLock<HashMap<String, TenantEntry>> {
    static TENANTS: OnceLock<RwLock<HashMap<String, TenantEntry>>> = OnceLock::new();
    TENANTS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Publisher identity ids currently registered in the tenant map (for trust sync preserve).
pub fn tenant_publisher_ids() -> Vec<String> {
    let guard = tenants().read().unwrap_or_else(|e| e.into_inner());
    let mut ids: Vec<String> = guard.values().map(|e| e.publisher_id.clone()).collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Bijective filesystem encoding of a CSU id (hex of UTF-8 bytes).
pub fn encode_csu_dir_name(csu_id: &str) -> String {
    hex::encode(csu_id.as_bytes())
}

/// Decode [`encode_csu_dir_name`].
pub fn decode_csu_dir_name(encoded: &str) -> Result<String, CryptoError> {
    let bytes = hex::decode(encoded.trim()).map_err(|_| CryptoError::InvalidKey)?;
    String::from_utf8(bytes).map_err(|_| CryptoError::InvalidKey)
}

fn tenants_root(root: &Path) -> PathBuf {
    root.join("identity").join(CSU_TENANTS_DIR)
}

fn tenant_dir(root: &Path, csu_id: &str) -> PathBuf {
    tenants_root(root).join(encode_csu_dir_name(csu_id))
}

fn remove_path_quiet(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn unix_stamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

/// On-disk tenant metadata (no secret material).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsuTenantMeta {
    pub csu_id: String,
    pub publisher_id: String,
    pub public_key_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotated_at: Option<String>,
}

/// Listed durable tenant entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsuTenantInfo {
    pub csu_id: String,
    pub publisher_id: String,
    pub public_key_hex: String,
    pub dir: PathBuf,
}

fn publisher_owned_by_other_csu(publisher_id: &str, except_csu: &str) -> Option<String> {
    let guard = tenants().read().unwrap_or_else(|e| e.into_inner());
    for (csu, ent) in guard.iter() {
        if csu != except_csu && ent.publisher_id == publisher_id {
            return Some(csu.clone());
        }
    }
    None
}

fn publisher_on_disk_other(
    root: &Path,
    publisher_id: &str,
    except_csu: &str,
) -> Result<Option<String>, CryptoError> {
    for t in list_csu_tenant_signing(root)? {
        if t.csu_id != except_csu && t.publisher_id == publisher_id {
            return Ok(Some(t.csu_id));
        }
    }
    Ok(None)
}

/// Register (or replace) signing material for a CSU tenant (in-memory).
///
/// - Stores the **signing** key only in the tenant map (not in process Keyring signing).
/// - Merges the **verifying** key into the process Keyring so `verify_ed25519` works.
/// - Refuses if another CSU already owns the same `publisher_id`.
pub fn register_csu_tenant_signing(
    csu_id: &AiraRef,
    publisher: AiraRef,
    signing: SigningKey,
) -> Result<(), CryptoError> {
    let csu = csu_id.as_str().trim();
    let pub_id = publisher.as_str().trim();
    if csu.is_empty() || pub_id.is_empty() {
        return Err(CryptoError::InvalidKey);
    }
    AiraRef::parse(csu).map_err(|_| CryptoError::InvalidKey)?;
    AiraRef::parse(pub_id).map_err(|_| CryptoError::InvalidKey)?;

    if let Some(other) = publisher_owned_by_other_csu(pub_id, csu) {
        return Err(CryptoError::TenantIsolation(format!(
            "publisher {pub_id} already bound to csu {other}"
        )));
    }

    let vk = signing.verifying_key();
    let mut ring = Keyring::new();
    ring.insert_verifying(publisher.clone(), vk);
    register_keyring(&ring);

    let mut guard = tenants().write().unwrap_or_else(|e| e.into_inner());
    guard.insert(
        csu.to_string(),
        TenantEntry {
            publisher_id: pub_id.to_string(),
            signing,
        },
    );
    Ok(())
}

fn write_secret_0600(path: &Path, contents: &[u8]) -> Result<(), CryptoError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true).mode(0o600);
        let mut file = opts
            .open(path)
            .map_err(|e| CryptoError::Io(e.to_string()))?;
        file.write_all(contents)
            .map_err(|e| CryptoError::Io(e.to_string()))?;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    {
        fs::write(path, contents).map_err(|e| CryptoError::Io(e.to_string()))?;
    }
    Ok(())
}

fn commit_secret_then_meta(
    dir: &Path,
    secret_hex: &str,
    meta: &CsuTenantMeta,
) -> Result<(), CryptoError> {
    let secret_path = dir.join(CSU_TENANT_SECRET_FILE);
    let meta_path = dir.join(CSU_TENANT_META_FILE);
    let secret_tmp = dir.join("ed25519.tmp");
    let meta_tmp = dir.join("meta.json.tmp");
    let meta_out =
        serde_json::to_string_pretty(meta).map_err(|e| CryptoError::Io(e.to_string()))?;
    write_secret_0600(&secret_tmp, format!("{secret_hex}\n").as_bytes())?;
    fs::write(&meta_tmp, format!("{meta_out}\n")).map_err(|e| CryptoError::Io(e.to_string()))?;
    // Secret first (A-62 M2): meta is the durable marker that a complete tenant exists.
    fs::rename(&secret_tmp, &secret_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    fs::rename(&meta_tmp, &meta_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    Ok(())
}

fn read_signing_hex(path: &Path) -> Result<SigningKey, CryptoError> {
    let raw = fs::read_to_string(path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let hex_s = raw.trim();
    let bytes = hex::decode(hex_s).map_err(|_| CryptoError::InvalidKey)?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::InvalidKey)?;
    Ok(SigningKey::from_bytes(&arr))
}

fn read_meta(dir: &Path) -> Result<CsuTenantMeta, CryptoError> {
    let meta_path = dir.join(CSU_TENANT_META_FILE);
    let meta_raw = fs::read_to_string(&meta_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    serde_json::from_str(&meta_raw).map_err(|e| CryptoError::Io(e.to_string()))
}

/// Persist tenant signing secret under `identity/tenants/<hex>/` and register in memory.
///
/// When `force` is false, refuses if the tenant dir already exists (use rotate / `--force`).
pub fn save_csu_tenant_signing(
    root: impl AsRef<Path>,
    csu_id: &AiraRef,
    publisher: AiraRef,
    signing: SigningKey,
    force: bool,
) -> Result<PathBuf, CryptoError> {
    let root = root.as_ref();
    let csu = csu_id.as_str().trim();
    let pub_id = publisher.as_str().trim();
    AiraRef::parse(csu).map_err(|_| CryptoError::InvalidKey)?;
    AiraRef::parse(pub_id).map_err(|_| CryptoError::InvalidKey)?;

    if let Some(other) = publisher_on_disk_other(root, pub_id, csu)? {
        return Err(CryptoError::TenantIsolation(format!(
            "publisher {pub_id} already bound on disk to csu {other}"
        )));
    }

    let dir = tenant_dir(root, csu);
    if dir.is_dir() && !force {
        return Err(CryptoError::Io(format!(
            "csu tenant already exists at {} — use `identity csu-tenant rotate` or `--force`",
            dir.display()
        )));
    }
    fs::create_dir_all(&dir).map_err(|e| CryptoError::Io(e.to_string()))?;

    let pub_hex = hex::encode(signing.verifying_key().to_bytes());
    let secret_hex = hex::encode(signing.to_bytes());
    let meta = CsuTenantMeta {
        csu_id: csu.to_string(),
        publisher_id: pub_id.to_string(),
        public_key_hex: pub_hex,
        created_at: utc_now_rfc3339().ok(),
        rotated_at: None,
    };
    commit_secret_then_meta(&dir, &secret_hex, &meta)?;
    register_csu_tenant_signing(csu_id, publisher, signing)?;
    Ok(dir)
}

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

/// Load one durable tenant from disk into memory. Missing dir → Err.
pub fn load_csu_tenant_signing(
    root: impl AsRef<Path>,
    csu_id: &AiraRef,
) -> Result<(), CryptoError> {
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
    let signing = read_signing_hex(&dir.join(CSU_TENANT_SECRET_FILE))?;
    let got = hex::encode(signing.verifying_key().to_bytes());
    if got != meta.public_key_hex.trim() {
        return Err(CryptoError::Io(
            "csu tenant secret does not match meta public_key_hex".into(),
        ));
    }
    register_csu_tenant_signing(csu_id, publisher, signing)
}

/// Load all durable tenants. Missing `identity/tenants/` → Ok(0).
pub fn load_all_csu_tenant_signing(root: impl AsRef<Path>) -> Result<usize, CryptoError> {
    let root = root.as_ref();
    let base = tenants_root(root);
    if !base.is_dir() {
        return Ok(0);
    }
    let mut n = 0usize;
    let rd = fs::read_dir(&base).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        if !ent.path().is_dir() {
            continue;
        }
        let name = ent.file_name();
        let enc = name.to_string_lossy();
        let csu_s = match decode_csu_dir_name(&enc) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let csu = match AiraRef::parse(&csu_s) {
            Ok(r) => r,
            Err(_) => continue,
        };
        load_csu_tenant_signing(root, &csu)?;
        n += 1;
    }
    Ok(n)
}

/// List durable tenant dirs (does not register).
pub fn list_csu_tenant_signing(root: impl AsRef<Path>) -> Result<Vec<CsuTenantInfo>, CryptoError> {
    let root = root.as_ref();
    let base = tenants_root(root);
    if !base.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let rd = fs::read_dir(&base).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        let dir = ent.path();
        if !dir.is_dir() {
            continue;
        }
        let meta_path = dir.join(CSU_TENANT_META_FILE);
        if !meta_path.is_file() {
            continue;
        }
        let meta_raw =
            fs::read_to_string(&meta_path).map_err(|e| CryptoError::Io(e.to_string()))?;
        let meta: CsuTenantMeta =
            serde_json::from_str(&meta_raw).map_err(|e| CryptoError::Io(e.to_string()))?;
        out.push(CsuTenantInfo {
            csu_id: meta.csu_id,
            publisher_id: meta.publisher_id,
            public_key_hex: meta.public_key_hex,
            dir,
        });
    }
    out.sort_by(|a, b| a.csu_id.cmp(&b.csu_id));
    Ok(out)
}

/// Remove a CSU tenant signing entry (tests / unload).
pub fn unregister_csu_tenant(csu_id: &AiraRef) {
    let mut guard = tenants().write().unwrap_or_else(|e| e.into_inner());
    guard.remove(csu_id.as_str());
}

/// Clear all CSU tenant signing entries (tests).
pub fn reset_csu_tenants() {
    let mut guard = tenants().write().unwrap_or_else(|e| e.into_inner());
    guard.clear();
}

/// Sign as `publisher` under CSU tenant isolation.
pub fn signature_for_tenant(
    csu_id: &AiraRef,
    publisher: &AiraRef,
    message: &[u8],
) -> Result<Signature, CryptoError> {
    let csu = csu_id.as_str();
    let pub_id = publisher.as_str();
    {
        let guard = tenants().read().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get(csu) {
            if entry.publisher_id != pub_id {
                return Err(CryptoError::TenantIsolation(format!(
                    "csu {csu} registered publisher {} but emit requested {pub_id}",
                    entry.publisher_id
                )));
            }
            return Ok(sign_with_key(publisher.clone(), &entry.signing, message));
        }
    }
    if pub_id == LOCAL_TEST_KEY_REF || pub_id == primary_signer().as_str() {
        return signature_for(publisher, message);
    }
    Err(CryptoError::TenantIsolation(format!(
        "csu {csu} has no tenant signing key for publisher {pub_id} — call register_csu_tenant_signing"
    )))
}

/// Whether a CSU currently has a tenant signing registration.
pub fn csu_tenant_registered(csu_id: &AiraRef) -> bool {
    let guard = tenants().read().unwrap_or_else(|e| e.into_inner());
    guard.contains_key(csu_id.as_str())
}

/// One listed tenant signing-secret backup (latest slot or archived stamp).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{
        ensure_trust_defaults, register_node_identity, reset_primary_signer, set_primary_signer,
        signature_for, unregister_verifying, verify_ed25519, Keyring, LOCAL_TEST_KEY_REF,
    };
    use std::sync::Mutex;
    use tempfile::tempdir;

    /// Serialize tests that mutate the process-wide tenant map / primary signer.
    fn tenant_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn write_min_node(root: &Path, name: &str, seed: [u8; 32]) {
        let idir = root.join("identity");
        fs::create_dir_all(&idir).unwrap();
        let sk = SigningKey::from_bytes(&seed);
        let pub_hex = hex::encode(sk.verifying_key().to_bytes());
        let id = format!("aira:identity:{name}");
        fs::write(
            idir.join("local.ed25519"),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        fs::write(
            idir.join("local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": name,
                "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        fs::write(root.join("config.json"), "{}\n").unwrap();
    }

    #[test]
    fn tenant_isolation_blocks_cross_csu_publisher() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap());

        let csu_a = AiraRef::parse("aira:csu:iso.a71").unwrap();
        let csu_b = AiraRef::parse("aira:csu:iso.b72").unwrap();
        let pub_a = AiraRef::parse("aira:identity:iso-pub-a71").unwrap();
        let pub_b = AiraRef::parse("aira:identity:iso-pub-b72").unwrap();
        let sk_a = SigningKey::from_bytes(&[71u8; 32]);
        let sk_b = SigningKey::from_bytes(&[72u8; 32]);

        register_csu_tenant_signing(&csu_a, pub_a.clone(), sk_a).unwrap();
        register_csu_tenant_signing(&csu_b, pub_b.clone(), sk_b).unwrap();

        let msg = b"tenant-isolation";
        let sig_a = signature_for_tenant(&csu_a, &pub_a, msg).unwrap();
        assert_eq!(sig_a.key_ref.as_str(), pub_a.as_str());
        let mut check = Keyring::new();
        check.insert_verifying(
            pub_a.clone(),
            SigningKey::from_bytes(&[71u8; 32]).verifying_key(),
        );
        check.verify(&sig_a, msg).unwrap();

        let err = signature_for_tenant(&csu_a, &pub_b, msg).unwrap_err();
        assert!(matches!(err, CryptoError::TenantIsolation(_)));

        assert!(matches!(
            signature_for(&pub_b, msg),
            Err(CryptoError::NoSigningKey(_))
        ));

        unregister_csu_tenant(&csu_a);
        unregister_csu_tenant(&csu_b);
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn unregistered_non_primary_publisher_fails_closed() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap());
        let csu = AiraRef::parse("aira:csu:tenant.stock.u91").unwrap();
        let foreign = AiraRef::parse("aira:identity:foreign-pub-u91").unwrap();
        let err = signature_for_tenant(&csu, &foreign, b"x").unwrap_err();
        assert!(matches!(err, CryptoError::TenantIsolation(_)));
        signature_for_tenant(&csu, &AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap(), b"x").unwrap();
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn save_load_survives_reset() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-t81", [61u8; 32]);
        let csu = AiraRef::parse("aira:csu:durable.a81").unwrap();
        let pub_id = AiraRef::parse("aira:identity:pub-durable-81").unwrap();
        let sk = SigningKey::from_bytes(&[81u8; 32]);
        let path = save_csu_tenant_signing(root, &csu, pub_id.clone(), sk, false).unwrap();
        assert!(path.join(CSU_TENANT_SECRET_FILE).is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(path.join(CSU_TENANT_SECRET_FILE))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
        reset_csu_tenants();
        assert!(!csu_tenant_registered(&csu));
        load_csu_tenant_signing(root, &csu).unwrap();
        let sig = signature_for_tenant(&csu, &pub_id, b"reload").unwrap();
        assert_eq!(sig.key_ref.as_str(), pub_id.as_str());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn load_all_isolation_and_empty_ok() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        assert_eq!(load_all_csu_tenant_signing(root).unwrap(), 0);
        write_min_node(root, "node-m83", [62u8; 32]);
        let csu_a = AiraRef::parse("aira:csu:multi.a83").unwrap();
        let csu_b = AiraRef::parse("aira:csu:multi.b84").unwrap();
        let pub_a = AiraRef::parse("aira:identity:pa83").unwrap();
        let pub_b = AiraRef::parse("aira:identity:pb84").unwrap();
        save_csu_tenant_signing(
            root,
            &csu_a,
            pub_a.clone(),
            SigningKey::from_bytes(&[83u8; 32]),
            false,
        )
        .unwrap();
        save_csu_tenant_signing(
            root,
            &csu_b,
            pub_b.clone(),
            SigningKey::from_bytes(&[84u8; 32]),
            false,
        )
        .unwrap();
        reset_csu_tenants();
        assert_eq!(load_all_csu_tenant_signing(root).unwrap(), 2);
        assert!(signature_for_tenant(&csu_a, &pub_b, b"x").is_err());
        signature_for_tenant(&csu_a, &pub_a, b"x").unwrap();
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn meta_pubkey_mismatch_fails_closed() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-bad85", [63u8; 32]);
        let csu = AiraRef::parse("aira:csu:bad.meta85").unwrap();
        let pub_id = AiraRef::parse("aira:identity:pub-bad85").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id,
            SigningKey::from_bytes(&[85u8; 32]),
            false,
        )
        .unwrap();
        let meta_path = tenant_dir(root, csu.as_str()).join(CSU_TENANT_META_FILE);
        let mut meta: CsuTenantMeta =
            serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();
        meta.public_key_hex = "00".repeat(32);
        fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).unwrap();
        reset_csu_tenants();
        let err = load_csu_tenant_signing(root, &csu).unwrap_err();
        assert!(err.to_string().contains("public_key_hex"));
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn trust_sync_then_load_all_restores_verifier() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-sync86", [64u8; 32]);
        let csu = AiraRef::parse("aira:csu:sync.t86").unwrap();
        let pub_id = AiraRef::parse("aira:identity:pub-sync86").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id.clone(),
            SigningKey::from_bytes(&[86u8; 32]),
            false,
        )
        .unwrap();
        reset_csu_tenants();
        register_node_identity(root).unwrap();
        ensure_trust_defaults(root).unwrap();
        load_all_csu_tenant_signing(root).unwrap();
        signature_for_tenant(&csu, &pub_id, b"after-sync").unwrap();
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn encode_decode_roundtrip() {
        let id = "aira:csu:foo.bar";
        let enc = encode_csu_dir_name(id);
        assert_eq!(decode_csu_dir_name(&enc).unwrap(), id);
    }

    #[test]
    fn register_refuses_duplicate_publisher() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let csu_a = AiraRef::parse("aira:csu:dup.a01").unwrap();
        let csu_b = AiraRef::parse("aira:csu:dup.b01").unwrap();
        let pub_id = AiraRef::parse("aira:identity:dup-pub-01").unwrap();
        register_csu_tenant_signing(&csu_a, pub_id.clone(), SigningKey::from_bytes(&[11u8; 32]))
            .unwrap();
        let err = register_csu_tenant_signing(&csu_b, pub_id, SigningKey::from_bytes(&[12u8; 32]))
            .unwrap_err();
        assert!(matches!(err, CryptoError::TenantIsolation(_)));
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn register_default_refuses_overwrite_and_force_allows() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-force01", [65u8; 32]);
        let csu = AiraRef::parse("aira:csu:force.01").unwrap();
        let pub_id = AiraRef::parse("aira:identity:force-pub-01").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id.clone(),
            SigningKey::from_bytes(&[91u8; 32]),
            false,
        )
        .unwrap();
        let err = save_csu_tenant_signing(
            root,
            &csu,
            pub_id.clone(),
            SigningKey::from_bytes(&[92u8; 32]),
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("--force") || err.to_string().contains("already exists"));
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id.clone(),
            SigningKey::from_bytes(&[92u8; 32]),
            true,
        )
        .unwrap();
        let sig = signature_for_tenant(&csu, &pub_id, b"forced").unwrap();
        verify_ed25519(&sig, b"forced").unwrap();
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn rotate_happy_path_and_audit() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-rot01", [66u8; 32]);
        let csu = AiraRef::parse("aira:csu:rot.01").unwrap();
        let pub_id = AiraRef::parse("aira:identity:rot-pub-01").unwrap();
        let old_sk = SigningKey::from_bytes(&[93u8; 32]);
        save_csu_tenant_signing(root, &csu, pub_id.clone(), old_sk, false).unwrap();
        let old_sig = signature_for_tenant(&csu, &pub_id, b"before").unwrap();
        verify_ed25519(&old_sig, b"before").unwrap();

        let (publisher, new_pub, old_pub, backup) =
            rotate_csu_tenant_signing(root, &csu, SigningKey::from_bytes(&[94u8; 32]), false)
                .unwrap();
        assert_eq!(publisher.as_str(), pub_id.as_str());
        assert_ne!(new_pub, old_pub);
        assert!(backup.is_none());
        assert!(verify_ed25519(&old_sig, b"before").is_err());
        let new_sig = signature_for_tenant(&csu, &pub_id, b"after").unwrap();
        verify_ed25519(&new_sig, b"after").unwrap();

        let audit = TrustAuditLog::load(root).unwrap();
        assert!(audit
            .iter()
            .any(|e| e.action == TrustAuditAction::TenantRotate && e.subject_id == csu.as_str()));
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn rotate_refuses_missing_and_backup_archives() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-rot02", [67u8; 32]);
        let csu = AiraRef::parse("aira:csu:rot.02").unwrap();
        let pub_id = AiraRef::parse("aira:identity:rot-pub-02").unwrap();
        assert!(
            rotate_csu_tenant_signing(root, &csu, SigningKey::from_bytes(&[95u8; 32]), false)
                .is_err()
        );
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id,
            SigningKey::from_bytes(&[95u8; 32]),
            false,
        )
        .unwrap();
        let (_, _, _, b1) =
            rotate_csu_tenant_signing(root, &csu, SigningKey::from_bytes(&[96u8; 32]), true)
                .unwrap();
        assert!(b1.unwrap().ends_with(CSU_TENANT_SECRET_BACKUP_FILE));
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let (_, _, _, b2) =
            rotate_csu_tenant_signing(root, &csu, SigningKey::from_bytes(&[97u8; 32]), true)
                .unwrap();
        assert!(b2.unwrap().ends_with(CSU_TENANT_SECRET_BACKUP_FILE));
        let tdir = tenant_dir(root, csu.as_str());
        let archived: Vec<_> = fs::read_dir(&tdir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with("ed25519.prev.") && !n.contains("meta"))
            .collect();
        assert!(!archived.is_empty(), "expected archived .prev.<stamp>");
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn revoke_removes_dir_map_and_audits() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-rev01", [68u8; 32]);
        let csu = AiraRef::parse("aira:csu:rev.01").unwrap();
        let pub_id = AiraRef::parse("aira:identity:rev-pub-01").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id.clone(),
            SigningKey::from_bytes(&[98u8; 32]),
            false,
        )
        .unwrap();
        assert!(csu_tenant_registered(&csu));
        revoke_csu_tenant_signing(root, &csu, "compromised").unwrap();
        assert!(!csu_tenant_registered(&csu));
        assert!(!tenant_dir(root, csu.as_str()).exists());
        assert!(signature_for_tenant(&csu, &pub_id, b"x").is_err());
        let audit = TrustAuditLog::load(root).unwrap();
        assert!(audit.iter().any(|e| {
            e.action == TrustAuditAction::TenantRevoke && e.reason.as_deref() == Some("compromised")
        }));
        assert!(revoke_csu_tenant_signing(root, &csu, "").is_err());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn revoke_never_drops_primary_signer() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-prim01", [69u8; 32]);
        register_node_identity(root).unwrap();
        let primary = primary_signer();
        assert!(!unregister_verifying(&primary));
        assert!(!unregister_verifying(
            &AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap()
        ));
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn save_secret_first_partial_commit_fail_closed() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-partial01", [70u8; 32]);
        let csu = AiraRef::parse("aira:csu:partial.01").unwrap();
        let pub_id = AiraRef::parse("aira:identity:partial-pub-01").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id,
            SigningKey::from_bytes(&[99u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        // Simulate crash after secret rename, before meta: remove meta only.
        fs::remove_file(tdir.join(CSU_TENANT_META_FILE)).unwrap();
        reset_csu_tenants();
        assert!(load_csu_tenant_signing(root, &csu).is_err());
        // list skips dirs without meta
        assert!(list_csu_tenant_signing(root).unwrap().is_empty());
        reset_csu_tenants();
        reset_primary_signer();
    }

    fn plant_archive(tdir: &Path, stamp: &str, body: &[u8]) {
        fs::write(
            tdir.join(format!("{CSU_TENANT_SECRET_BACKUP_FILE}.{stamp}")),
            body,
        )
        .unwrap();
    }

    fn plant_latest_prev(tdir: &Path, body: &[u8]) {
        fs::write(tdir.join(CSU_TENANT_SECRET_BACKUP_FILE), body).unwrap();
    }

    #[test]
    fn prune_keep_one_isolates_two_tenants() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr01", [71u8; 32]);
        let csu_a = AiraRef::parse("aira:csu:pr.a").unwrap();
        let csu_b = AiraRef::parse("aira:csu:pr.b").unwrap();
        save_csu_tenant_signing(
            root,
            &csu_a,
            AiraRef::parse("aira:identity:pr-a").unwrap(),
            SigningKey::from_bytes(&[21u8; 32]),
            false,
        )
        .unwrap();
        save_csu_tenant_signing(
            root,
            &csu_b,
            AiraRef::parse("aira:identity:pr-b").unwrap(),
            SigningKey::from_bytes(&[22u8; 32]),
            false,
        )
        .unwrap();
        let da = tenant_dir(root, csu_a.as_str());
        let db = tenant_dir(root, csu_b.as_str());
        plant_latest_prev(&da, b"la\n");
        plant_latest_prev(&db, b"lb\n");
        plant_archive(&da, "100", b"a-old\n");
        plant_archive(&da, "200", b"a-new\n");
        plant_archive(&db, "100", b"b-old\n");
        plant_archive(&db, "200", b"b-new\n");

        let report = prune_csu_tenant_secret_backups(root, Some(1), None, false).unwrap();
        assert_eq!(report.deleted.len(), 2);
        assert!(!da.join("ed25519.prev.100").is_file());
        assert!(da.join("ed25519.prev.200").is_file());
        assert!(da.join(CSU_TENANT_SECRET_BACKUP_FILE).is_file());
        assert!(!db.join("ed25519.prev.100").is_file());
        assert!(db.join("ed25519.prev.200").is_file());
        assert!(db.join(CSU_TENANT_SECRET_BACKUP_FILE).is_file());
        assert!(da.join(CSU_TENANT_SECRET_FILE).is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn prune_keep_zero_drops_archives_keeps_latest() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr02", [72u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.keep0").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-k0").unwrap(),
            SigningKey::from_bytes(&[23u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        plant_latest_prev(&tdir, b"latest\n");
        plant_archive(&tdir, "1", b"old\n");
        plant_archive(&tdir, "2", b"mid\n");
        prune_csu_tenant_secret_backups(root, Some(0), None, false).unwrap();
        assert!(!tdir.join("ed25519.prev.1").is_file());
        assert!(!tdir.join("ed25519.prev.2").is_file());
        assert!(tdir.join(CSU_TENANT_SECRET_BACKUP_FILE).is_file());
        assert!(tdir.join(CSU_TENANT_SECRET_FILE).is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn prune_older_than_skips_unparseable_keep_still_ranks() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr03", [73u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.age").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-age").unwrap(),
            SigningKey::from_bytes(&[24u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        plant_archive(&tdir, "notanumber", b"bad\n");
        plant_archive(&tdir, "50", b"ok\n");
        let skipped = prune_csu_tenant_secret_backups(root, None, Some(1), false).unwrap();
        assert!(tdir.join("ed25519.prev.notanumber").is_file());
        assert!(skipped
            .skipped
            .iter()
            .any(|(_, w)| w.contains("unparseable")));
        prune_csu_tenant_secret_backups(root, Some(0), None, false).unwrap();
        assert!(!tdir.join("ed25519.prev.notanumber").is_file());
        assert!(!tdir.join("ed25519.prev.50").is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn prune_dry_run_and_requires_policy() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr04", [74u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.dry").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-dry").unwrap(),
            SigningKey::from_bytes(&[25u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        plant_archive(&tdir, "3", b"x\n");
        let dry = prune_csu_tenant_secret_backups(root, Some(0), None, true).unwrap();
        assert!(dry.dry_run);
        assert!(!dry.deleted.is_empty());
        assert!(tdir.join("ed25519.prev.3").is_file());
        assert!(prune_csu_tenant_secret_backups(root, None, None, false).is_err());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn prune_never_deletes_orphan_meta_latest_or_live() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr05", [75u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.orphan").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-or").unwrap(),
            SigningKey::from_bytes(&[26u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        plant_latest_prev(&tdir, b"lat\n");
        let orphan = tdir.join("ed25519.prev.99.meta.json");
        fs::write(&orphan, "{}\n").unwrap();
        prune_csu_tenant_secret_backups(root, Some(0), None, false).unwrap();
        assert!(orphan.is_file());
        assert!(tdir.join(CSU_TENANT_SECRET_BACKUP_FILE).is_file());
        assert!(tdir.join(CSU_TENANT_SECRET_FILE).is_file());
        assert!(tdir.join(CSU_TENANT_META_FILE).is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn node_prune_does_not_touch_tenant_archives() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr06", [76u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.node").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-node").unwrap(),
            SigningKey::from_bytes(&[27u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        plant_archive(&tdir, "8", b"t\n");
        crate::crypto::prune_node_secret_backups(root, Some(0), None, false).unwrap();
        assert!(tdir.join("ed25519.prev.8").is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn list_includes_latest_after_rotate_backup() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr07", [77u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.list").unwrap();
        let pub_id = AiraRef::parse("aira:identity:pr-list").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            pub_id,
            SigningKey::from_bytes(&[28u8; 32]),
            false,
        )
        .unwrap();
        rotate_csu_tenant_signing(root, &csu, SigningKey::from_bytes(&[29u8; 32]), true).unwrap();
        let list = list_csu_tenant_secret_backups(root).unwrap();
        assert!(list.iter().any(|b| b.is_latest && b.csu_id == csu.as_str()));
        assert_eq!(list[0].csu_id, csu.as_str());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn prune_numeric_rank_prefers_10_over_9() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr08", [78u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.lex").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-lex").unwrap(),
            SigningKey::from_bytes(&[30u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        plant_archive(&tdir, "9", b"nine\n");
        plant_archive(&tdir, "10", b"ten\n");
        prune_csu_tenant_secret_backups(root, Some(1), None, false).unwrap();
        assert!(!tdir.join("ed25519.prev.9").is_file());
        assert!(tdir.join("ed25519.prev.10").is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }

    #[test]
    fn prune_and_list_ignore_tmp_staging() {
        let _lock = tenant_test_lock();
        reset_csu_tenants();
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "node-pr09", [79u8; 32]);
        let csu = AiraRef::parse("aira:csu:pr.tmp").unwrap();
        save_csu_tenant_signing(
            root,
            &csu,
            AiraRef::parse("aira:identity:pr-tmp").unwrap(),
            SigningKey::from_bytes(&[31u8; 32]),
            false,
        )
        .unwrap();
        let tdir = tenant_dir(root, csu.as_str());
        let tmp = tdir.join("ed25519.prev.tmp");
        fs::write(&tmp, b"staging\n").unwrap();
        plant_archive(&tdir, "4", b"real\n");
        let listed = list_csu_tenant_secret_backups(root).unwrap();
        assert!(!listed.iter().any(|b| b.secret_path.ends_with(".tmp")));
        prune_csu_tenant_secret_backups(root, Some(0), None, false).unwrap();
        assert!(tmp.is_file());
        assert!(!tdir.join("ed25519.prev.4").is_file());
        reset_csu_tenants();
        reset_primary_signer();
    }
}
