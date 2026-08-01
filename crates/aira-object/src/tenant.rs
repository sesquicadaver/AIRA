//! Per-CSU tenant signing isolation (Analyze-42).
//!
//! Signing secrets for CSU publishers live in a process map keyed by `csu_id`.
//! Only verifying keys are merged into the process [`Keyring`] (public material).
//! CSU emit helpers must use [`signature_for_tenant`].

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use ed25519_dalek::SigningKey;

use crate::crypto::{
    primary_signer, register_keyring, sign_with_key, signature_for, CryptoError, Keyring,
    LOCAL_TEST_KEY_REF,
};
use crate::types::{AiraRef, Signature};

struct TenantEntry {
    publisher_id: String,
    signing: SigningKey,
}

fn tenants() -> &'static RwLock<HashMap<String, TenantEntry>> {
    static TENANTS: OnceLock<RwLock<HashMap<String, TenantEntry>>> = OnceLock::new();
    TENANTS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register (or replace) signing material for a CSU tenant.
///
/// - Stores the **signing** key only in the tenant map (not in process Keyring signing).
/// - Merges the **verifying** key into the process Keyring so `verify_ed25519` works.
///
/// `publisher` must be the identity this CSU is allowed to sign as.
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
///
/// - If the CSU has a tenant registration: publisher must match; sign with tenant secret.
/// - If unregistered: only `primary_signer` or [`LOCAL_TEST_KEY_REF`] via process Keyring.
/// - Otherwise: [`CryptoError::TenantIsolation`].
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
    // Stock / default path: process keyring for primary or local-test only.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{
        reset_primary_signer, set_primary_signer, signature_for, Keyring, LOCAL_TEST_KEY_REF,
    };

    #[test]
    fn tenant_isolation_blocks_cross_csu_publisher() {
        set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap());

        let csu_a = AiraRef::parse("aira:csu:tenant.a").unwrap();
        let csu_b = AiraRef::parse("aira:csu:tenant.b").unwrap();
        let pub_a = AiraRef::parse("aira:identity:pub-a").unwrap();
        let pub_b = AiraRef::parse("aira:identity:pub-b").unwrap();
        let sk_a = SigningKey::from_bytes(&[71u8; 32]);
        let sk_b = SigningKey::from_bytes(&[72u8; 32]);

        register_csu_tenant_signing(&csu_a, pub_a.clone(), sk_a).unwrap();
        register_csu_tenant_signing(&csu_b, pub_b.clone(), sk_b).unwrap();

        let msg = b"tenant-isolation";
        let sig_a = signature_for_tenant(&csu_a, &pub_a, msg).unwrap();
        assert_eq!(sig_a.key_ref.as_str(), pub_a.as_str());
        // Verify with a local ring (process keyring is shared across parallel tests).
        let mut check = Keyring::new();
        check.insert_verifying(
            pub_a.clone(),
            SigningKey::from_bytes(&[71u8; 32]).verifying_key(),
        );
        check.verify(&sig_a, msg).unwrap();

        // A cannot sign as B's publisher.
        let err = signature_for_tenant(&csu_a, &pub_b, msg).unwrap_err();
        assert!(matches!(err, CryptoError::TenantIsolation(_)));

        // Process signature_for must not have B's signing secret.
        assert!(matches!(
            signature_for(&pub_b, msg),
            Err(CryptoError::NoSigningKey(_))
        ));

        unregister_csu_tenant(&csu_a);
        unregister_csu_tenant(&csu_b);
        reset_primary_signer();
    }

    #[test]
    fn unregistered_non_primary_publisher_fails_closed() {
        set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap());
        let csu = AiraRef::parse("aira:csu:tenant.stock").unwrap();
        let foreign = AiraRef::parse("aira:identity:foreign-pub").unwrap();
        let err = signature_for_tenant(&csu, &foreign, b"x").unwrap_err();
        assert!(matches!(err, CryptoError::TenantIsolation(_)));
        // local-test still allowed without registration.
        signature_for_tenant(&csu, &AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap(), b"x").unwrap();
        reset_primary_signer();
    }
}
