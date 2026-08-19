//! Durable tenant secret + meta on disk (Analyze-83).

use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::crypto::{utc_now_rfc3339, CryptoError};
use crate::types::AiraRef;

use super::map::register_csu_tenant_signing;
use super::paths::{
    decode_csu_dir_name, tenant_dir, tenants_root, CSU_TENANT_META_FILE, CSU_TENANT_SECRET_FILE,
};

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
pub(super) fn write_secret_0600(path: &Path, contents: &[u8]) -> Result<(), CryptoError> {
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

pub(super) fn commit_secret_then_meta(
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

pub(super) fn read_meta(dir: &Path) -> Result<CsuTenantMeta, CryptoError> {
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
