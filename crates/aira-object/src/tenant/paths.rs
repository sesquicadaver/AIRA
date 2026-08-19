//! Tenant filesystem paths and CSU-id encoding (Analyze-83).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::crypto::CryptoError;

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
/// Bijective filesystem encoding of a CSU id (hex of UTF-8 bytes).
pub fn encode_csu_dir_name(csu_id: &str) -> String {
    hex::encode(csu_id.as_bytes())
}

/// Decode [`encode_csu_dir_name`].
pub(super) fn decode_csu_dir_name(encoded: &str) -> Result<String, CryptoError> {
    let bytes = hex::decode(encoded.trim()).map_err(|_| CryptoError::InvalidKey)?;
    String::from_utf8(bytes).map_err(|_| CryptoError::InvalidKey)
}

pub(super) fn tenants_root(root: &Path) -> PathBuf {
    root.join("identity").join(CSU_TENANTS_DIR)
}

pub(super) fn tenant_dir(root: &Path, csu_id: &str) -> PathBuf {
    tenants_root(root).join(encode_csu_dir_name(csu_id))
}

pub(super) fn remove_path_quiet(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

pub(super) fn unix_stamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}
