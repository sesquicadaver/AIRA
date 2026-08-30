//! AIRA core object model (Issue Set Epic 3 / #22–#24).
//!
//! Provides typed references, hashes, signatures, opaque handles, and
//! `ObjectDescriptor` aligned with Schema Pack core schemas.

mod audit;
mod canonical;
mod clock;
mod crypto;
mod descriptor;
mod handle;
mod tenant;
mod types;

pub use audit::{
    record_trust_audit, TrustAuditAction, TrustAuditEntry, TrustAuditLog, TRUST_AUDIT_FILE,
};
pub use canonical::{
    canonical_json_bytes, canonicalize_value, descriptor_signing_hash, descriptor_signing_message,
    sign_canonical_descriptor, strip_top_level_signature, verify_canonical_descriptor,
    verify_producer_signature_binding,
};
pub use clock::{
    now, reset_clock, set_clock, unix_seconds, unix_seconds_str, Clock, FixedClock, SystemClock,
    MVP_FIXED_TIMESTAMP,
};
pub use crypto::{
    active_identity, active_signature, bind_thread_crypto, ensure_trust_defaults,
    is_cryptographic_signature, list_node_secret_backups, local_test_public_key_hex,
    local_test_signature, local_test_signing_key, local_test_verifying_key, primary_signer,
    process_keyring_snapshot, prune_node_secret_backups, register_keyring, register_node_identity,
    register_trust_store, reset_primary_signer, rotate_node_signing_secret, set_primary_signer,
    sign_with_key, signature_for, sync_trust_verifiers, unregister_verifying, utc_now_rfc3339,
    verify_ed25519, CryptoError, Keyring, NodeSecretBackupInfo, NodeSecretPruneReport,
    RevokedEntry, ThreadCryptoGuard, TrustEntry, TrustStore, LOCAL_TEST_DOMAIN_MSG,
    LOCAL_TEST_KEY_REF, NODE_SECRET_BACKUP_FILE, NODE_SECRET_BACKUP_META_FILE,
};
pub use descriptor::{ObjectDescriptor, ObjectType};
pub use handle::object_store_access;
pub use handle::Handle;
pub use tenant::{
    csu_tenant_registered, encode_csu_dir_name, list_csu_tenant_secret_backups,
    list_csu_tenant_signing, load_all_csu_tenant_signing, load_csu_tenant_signing,
    prune_csu_tenant_secret_backups, register_csu_tenant_signing, reset_csu_tenants,
    revoke_csu_tenant_signing, rotate_csu_tenant_signing, save_csu_tenant_signing,
    signature_for_tenant, tenant_publisher_ids, unregister_csu_tenant, CsuTenantBackupInfo,
    CsuTenantInfo, CsuTenantMeta, CSU_TENANTS_DIR, CSU_TENANT_META_FILE,
    CSU_TENANT_SECRET_BACKUP_FILE, CSU_TENANT_SECRET_BACKUP_META_FILE, CSU_TENANT_SECRET_FILE,
};
pub use types::{AiraRef, ContentHash, Signature, Timestamp};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn aira_ref_roundtrip() {
        let r = AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap();
        let s = serde_json::to_string(&r).unwrap();
        let back: AiraRef = serde_json::from_str(&s).unwrap();
        assert_eq!(r, back);
        assert_eq!(r.as_str(), "aira:problem:01TESTPROBLEM");
    }

    #[test]
    fn aira_ref_rejects_invalid() {
        assert!(AiraRef::parse("not-a-ref").is_err());
        assert!(AiraRef::parse("aira:BAD:x").is_err());
    }

    #[test]
    fn hash_roundtrip() {
        let h = ContentHash::parse(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap();
        let s = serde_json::to_string(&h).unwrap();
        let back: ContentHash = serde_json::from_str(&s).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn signature_roundtrip() {
        let sig = local_test_signature(b"roundtrip");
        let v = serde_json::to_value(&sig).unwrap();
        let back: Signature = serde_json::from_value(v).unwrap();
        assert_eq!(sig, back);
        verify_ed25519(&sig, b"roundtrip").unwrap();
    }

    #[test]
    fn handle_is_opaque() {
        let object_ref = AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap();
        let h = Handle::new(object_ref.clone(), 42);
        let dbg = format!("{h:?}");
        assert!(!dbg.contains("sqlite"));
        assert!(!dbg.contains("/"));
        assert!(!dbg.contains("path"));
        assert!(!dbg.contains("42"));
        assert!(dbg.contains("<opaque>"));
        // Public API exposes only the logical ref, not storage internals.
        assert_eq!(h.object_ref(), &object_ref);
        assert!(h.storage_token() > 0);
    }

    #[test]
    fn handle_new_and_storage_token_are_not_public_methods() {
        let src = include_str!("handle.rs");
        assert!(
            src.contains("pub(crate) fn new("),
            "Handle::new must be crate-private"
        );
        assert!(
            src.contains("pub(crate) fn storage_token("),
            "Handle::storage_token must be crate-private"
        );
        assert!(
            !src.contains("pub fn new("),
            "Handle::new must not be a public method"
        );
        assert!(
            !src.contains("pub fn storage_token(&self)"),
            "Handle::storage_token must not be a public method"
        );
    }

    #[test]
    fn object_descriptor_schema_valid() {
        let desc = ObjectDescriptor::example_problem();
        let value = serde_json::to_value(&desc).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:core:object-descriptor:0.1", &value)
            .unwrap();
    }

    #[test]
    fn forbidden_object_types_rejected() {
        for bad in ["GPU", "Node", "Driver", "Scheduler", "Blockchain", "Wallet"] {
            let v = json!({
                "object_id": "aira:problem:01TESTPROBLEM",
                "object_type": bad,
                "schema_version": "0.1",
                "created_at": "2026-07-10T12:00:00Z",
                "producer_identity": "aira:identity:local-test",
                "policy_refs": [],
                "provenance_refs": [],
                "content_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "signature": {
                    "algorithm": "ed25519",
                    "key_ref": "aira:identity:local-test",
                    "signature_value": "TESTSIG"
                }
            });
            let err = serde_json::from_value::<ObjectDescriptor>(v).unwrap_err();
            assert!(
                err.to_string().contains(bad) || err.to_string().contains("object_type"),
                "expected reject for {bad}, got {err}"
            );
        }
    }

    #[test]
    fn canonical_verify_fails_when_object_fields_change() {
        let d = ObjectDescriptor::example_problem();
        d.verify_canonical().unwrap();

        let mut t = d.clone();
        t.object_type = ObjectType::Context;
        assert!(t.verify_canonical().is_err());

        let mut p = d.clone();
        p.policy_refs = vec![AiraRef::parse("aira:policy:private").unwrap()];
        assert!(p.verify_canonical().is_err());

        let mut pr = d.clone();
        pr.provenance_refs = vec![AiraRef::parse("aira:event:01E1").unwrap()];
        assert!(pr.verify_canonical().is_err());

        let mut h = d.clone();
        h.content_hash = ContentHash::parse(
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap();
        assert!(h.verify_canonical().is_err());

        let mut id = d;
        id.object_id = AiraRef::parse("aira:problem:MUTATED").unwrap();
        assert!(id.verify_canonical().is_err());
    }

    #[test]
    fn system_clock_is_not_the_mvp_fixed_timestamp() {
        reset_clock();
        assert_ne!(SystemClock.now().as_str(), MVP_FIXED_TIMESTAMP);
        assert_ne!(now().as_str(), MVP_FIXED_TIMESTAMP);
    }

    #[test]
    fn fixed_clock_now_is_the_installed_time() {
        let fixed = FixedClock::parse("2026-08-30T16:00:00Z").unwrap();
        assert_eq!(fixed.now().as_str(), "2026-08-30T16:00:00Z");
        assert_eq!(FixedClock::mvp().now().as_str(), MVP_FIXED_TIMESTAMP);
        set_clock(std::sync::Arc::new(
            FixedClock::parse("2026-08-30T16:41:00Z").unwrap(),
        ));
        assert_eq!(now().as_str(), "2026-08-30T16:41:00Z");
        reset_clock();
        assert_ne!(now().as_str(), "2026-08-30T16:41:00Z");
    }

    #[test]
    fn verify_rejects_cross_identity_key_ref() {
        let d = ObjectDescriptor::example_problem();
        let mut bad = d.clone();
        bad.signature.key_ref = AiraRef::parse("aira:identity:other-producer").unwrap();
        assert_eq!(
            bad.verify_canonical(),
            Err(CryptoError::ProducerIdentityMismatch {
                producer: d.producer_identity.as_str().to_string(),
                key_ref: "aira:identity:other-producer".into(),
            })
        );
    }
}
