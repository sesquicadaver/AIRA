//! Per-CSU tenant signing isolation (Analyze-42) + durable secrets (Analyze-62)
//! + rotate/revoke ceremony (Analyze-63) + backup prune (Analyze-71).
//!
//! Mechanical split (Analyze-83 / QUEUE #48).

mod ceremony;
mod map;
mod paths;
mod persist;
mod prune;

pub use ceremony::{revoke_csu_tenant_signing, rotate_csu_tenant_signing};
pub use map::{
    csu_tenant_registered, register_csu_tenant_signing, reset_csu_tenants, signature_for_tenant,
    tenant_publisher_ids, unregister_csu_tenant,
};
pub use paths::{
    encode_csu_dir_name, CSU_TENANTS_DIR, CSU_TENANT_META_FILE, CSU_TENANT_SECRET_BACKUP_FILE,
    CSU_TENANT_SECRET_BACKUP_META_FILE, CSU_TENANT_SECRET_FILE,
};
pub use persist::{
    list_csu_tenant_signing, load_all_csu_tenant_signing, load_csu_tenant_signing,
    save_csu_tenant_signing, CsuTenantInfo, CsuTenantMeta,
};
pub use prune::{
    list_csu_tenant_secret_backups, prune_csu_tenant_secret_backups, CsuTenantBackupInfo,
};

#[cfg(test)]
mod tests {
    use super::paths::{decode_csu_dir_name, tenant_dir};
    use super::*;
    use crate::audit::{TrustAuditAction, TrustAuditLog};
    use crate::crypto::{
        ensure_trust_defaults, primary_signer, register_node_identity, reset_primary_signer,
        set_primary_signer, signature_for, unregister_verifying, verify_ed25519, CryptoError,
        Keyring, LOCAL_TEST_KEY_REF,
    };
    use crate::types::AiraRef;
    use ed25519_dalek::SigningKey;
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
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
