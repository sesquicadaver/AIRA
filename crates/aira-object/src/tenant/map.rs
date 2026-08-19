//! In-process CSU tenant signing map (Analyze-83).

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use ed25519_dalek::SigningKey;

use crate::crypto::{
    primary_signer, register_keyring, sign_with_key, signature_for, CryptoError, Keyring,
    LOCAL_TEST_KEY_REF,
};
use crate::types::{AiraRef, Signature};

pub(super) struct TenantEntry {
    pub(super) publisher_id: String,
    pub(super) signing: SigningKey,
}

pub(super) fn tenants() -> &'static RwLock<HashMap<String, TenantEntry>> {
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
pub(super) fn publisher_owned_by_other_csu(publisher_id: &str, except_csu: &str) -> Option<String> {
    let guard = tenants().read().unwrap_or_else(|e| e.into_inner());
    for (csu, ent) in guard.iter() {
        if csu != except_csu && ent.publisher_id == publisher_id {
            return Some(csu.clone());
        }
    }
    None
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
