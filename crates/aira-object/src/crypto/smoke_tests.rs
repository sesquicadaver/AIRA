//! Crypto module integration tests (QUEUE #127 — split from `mod.rs`).

use super::*;
use crate::types::{AiraRef, Signature};
use ed25519_dalek::SigningKey;
use std::fs;
use tempfile::tempdir;

/// Hold across any test that mutates process keyring / primary signer.
fn lock_process_crypto() -> std::sync::MutexGuard<'static, ()> {
    process_crypto_test_lock()
}

#[test]
fn local_test_sign_verify_roundtrip() {
    let msg = b"aira:artifact:hash-demo";
    let sig = local_test_signature(msg);
    assert_ne!(sig.signature_value, "TESTSIG");
    assert_eq!(sig.signature_value.len(), 128);
    verify_ed25519(&sig, msg).unwrap();
}

#[test]
fn rejects_testsig_and_tamper() {
    let msg = b"payload";
    let mut sig = local_test_signature(msg);
    assert!(verify_ed25519(
        &Signature {
            algorithm: "ed25519".into(),
            key_ref: sig.key_ref.clone(),
            signature_value: "TESTSIG".into(),
        },
        msg
    )
    .is_err());
    let mut chars: Vec<char> = sig.signature_value.chars().collect();
    chars[0] = if chars[0] == '0' { '1' } else { '0' };
    sig.signature_value = chars.into_iter().collect();
    assert_eq!(verify_ed25519(&sig, msg), Err(CryptoError::VerifyFailed));
}

#[test]
fn public_key_hex_is_stable() {
    let hex = local_test_public_key_hex();
    assert_eq!(hex.len(), 64);
    assert_eq!(hex, local_test_public_key_hex());
}

#[test]
fn with_verifying_hex_detached_roundtrip_and_no_local_test() {
    let sk = SigningKey::from_bytes(&[9u8; 32]);
    let id = AiraRef::parse("aira:identity:fed-signer").unwrap();
    let pub_hex = hex::encode(sk.verifying_key().to_bytes());
    let msg = b"aira:federation:descriptor:v1|detached";
    let sig = sign_with_key(id.clone(), &sk, msg);
    let ring = Keyring::with_verifying_hex(&id, &pub_hex).unwrap();
    ring.verify(&sig, msg).unwrap();
    assert!(ring.verifying_key(LOCAL_TEST_KEY_REF).is_none());
    let local_sig = local_test_signature(msg);
    assert_eq!(
        ring.verify(&local_sig, msg),
        Err(CryptoError::UnknownKey(LOCAL_TEST_KEY_REF.into()))
    );
    assert!(Keyring::with_verifying_hex(&id, "zz").is_err());
}

#[test]
fn node_identity_keyring_sign_verify() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let sk = SigningKey::from_bytes(&[7u8; 32]);
    let id = "aira:identity:node-demo";
    let pub_hex = hex::encode(sk.verifying_key().to_bytes());
    fs::write(
        root.join("identity/local.ed25519"),
        format!("{}\n", hex::encode(sk.to_bytes())),
    )
    .unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-demo",
            "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();

    let (loaded_id, ring) = Keyring::load_node_identity(root).unwrap();
    assert_eq!(loaded_id.as_str(), id);
    let msg = b"node-bound-message";
    let sig = ring.sign(&AiraRef::parse(id).unwrap(), msg).unwrap();
    ring.verify(&sig, msg).unwrap();
    // local-test still present
    ring.verify(&local_test_signature(msg), msg).unwrap();

    register_keyring(&ring);
    verify_ed25519(&sig, msg).unwrap();

    set_primary_signer(loaded_id.clone());
    assert_eq!(active_identity().as_str(), id);
    let active = active_signature(msg).unwrap();
    assert_eq!(active.key_ref.as_str(), id);
    verify_ed25519(&active, msg).unwrap();
    reset_primary_signer();
    assert_eq!(primary_signer().as_str(), LOCAL_TEST_KEY_REF);
}

#[test]
fn active_signature_does_not_fallback_to_local_test() {
    let _lock = lock_process_crypto();
    reset_primary_signer();
    let missing = AiraRef::parse("aira:identity:no-signing-key").unwrap();
    set_primary_signer(missing.clone());
    let err = active_signature(b"must-not-become-local-test").unwrap_err();
    assert!(
        matches!(err, CryptoError::NoSigningKey(ref id) if id == missing.as_str()),
        "{err:?}"
    );
    reset_primary_signer();
    let demo = active_signature(b"explicit-demo-local-test").unwrap();
    assert_eq!(demo.key_ref.as_str(), LOCAL_TEST_KEY_REF);
}

#[test]
fn thread_crypto_scopes_do_not_leak() {
    let _lock = lock_process_crypto();
    reset_primary_signer();
    fn scoped(seed: u8, name: &str) -> (AiraRef, Keyring) {
        let id = AiraRef::parse(format!("aira:identity:{name}")).unwrap();
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let mut ring = Keyring::new();
        ring.insert_signing(id.clone(), sk);
        (id, ring)
    }
    let (id_a, ring_a) = scoped(71, "thread-scope-a");
    let (id_b, ring_b) = scoped(73, "thread-scope-b");
    {
        let _g = bind_thread_crypto(ring_a.clone(), id_a.clone());
        assert_eq!(active_identity().as_str(), id_a.as_str());
        active_signature(b"same-thread").unwrap();
    }
    // Drop restores process primary; under the shared lock it stays local-test.
    assert_eq!(primary_signer().as_str(), LOCAL_TEST_KEY_REF);
    assert_ne!(primary_signer().as_str(), id_a.as_str());
    assert!(process_keyring_snapshot()
        .verifying_key(id_a.as_str())
        .is_none());

    let a = std::thread::spawn({
        let id_a = id_a.clone();
        let ring_a = ring_a.clone();
        let id_b = id_b.clone();
        move || {
            let _g = bind_thread_crypto(ring_a, id_a.clone());
            assert_eq!(active_identity().as_str(), id_a.as_str());
            let sig = active_signature(b"scope-a").unwrap();
            verify_ed25519(&sig, b"scope-a").unwrap();
            let foreign = sign_with_key(id_b.clone(), &SigningKey::from_bytes(&[73u8; 32]), b"x");
            assert!(verify_ed25519(&foreign, b"x").is_err());
            sig
        }
    });
    let b = std::thread::spawn({
        let id_b = id_b.clone();
        let ring_b = ring_b.clone();
        let id_a = id_a.clone();
        move || {
            let _g = bind_thread_crypto(ring_b, id_b.clone());
            assert_eq!(active_identity().as_str(), id_b.as_str());
            let sig = active_signature(b"scope-b").unwrap();
            verify_ed25519(&sig, b"scope-b").unwrap();
            let foreign = sign_with_key(id_a.clone(), &SigningKey::from_bytes(&[71u8; 32]), b"x");
            assert!(verify_ed25519(&foreign, b"x").is_err());
            sig
        }
    });
    let sig_a = a.join().unwrap();
    let sig_b = b.join().unwrap();
    assert_ne!(sig_a.key_ref.as_str(), sig_b.key_ref.as_str());
    assert!(verify_ed25519(&sig_a, b"scope-a").is_err());
    assert!(verify_ed25519(&sig_b, b"scope-b").is_err());
    assert!(process_keyring_snapshot()
        .verifying_key(id_a.as_str())
        .is_none());
    assert!(process_keyring_snapshot()
        .verifying_key(id_b.as_str())
        .is_none());
}

#[test]
fn trust_store_peer_verify_without_signing_key() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let peer_sk = SigningKey::from_bytes(&[13u8; 32]);
    let peer_id = "aira:identity:peer-alice";
    let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());

    let mut store = TrustStore::default();
    store.upsert(peer_id, &peer_pub).unwrap();
    store.save(root).unwrap();

    let loaded = TrustStore::load(root).unwrap();
    assert_eq!(loaded.entries.len(), 1);
    assert!(!loaded
        .entries
        .iter()
        .any(|e| e.identity_id == LOCAL_TEST_KEY_REF));
    let _ = register_trust_store(root).unwrap();

    let msg = b"peer-message";
    let sig = sign_with_key(AiraRef::parse(peer_id).unwrap(), &peer_sk, msg);
    // File-backed ring (process keyring is shared across parallel tests).
    let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
    ring.verify(&sig, msg).unwrap();

    store.remove(peer_id);
    store.save(root).unwrap();
    let _ = sync_trust_verifiers(root).unwrap();
    let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
    assert!(ring.verifying_key(peer_id).is_none());
    assert!(ring.verifying_key(LOCAL_TEST_KEY_REF).is_none());
    assert!(ring.verify(&sig, msg).is_err());
    assert!(ring.verify(&local_test_signature(msg), msg).is_err());
}

#[test]
fn trust_upsert_rejects_local_test() {
    let mut store = TrustStore::default();
    assert_eq!(
        store.upsert(LOCAL_TEST_KEY_REF, &local_test_public_key_hex()),
        Err(CryptoError::ProtectedIdentity(LOCAL_TEST_KEY_REF.into()))
    );
    assert!(store.entries.is_empty());
}

#[test]
fn ensure_trust_defaults_strips_legacy_local_test() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let sk = SigningKey::from_bytes(&[11u8; 32]);
    let id = "aira:identity:node-sec1";
    let pub_hex = hex::encode(sk.verifying_key().to_bytes());
    fs::write(
        root.join("identity/local.ed25519"),
        format!("{}\n", hex::encode(sk.to_bytes())),
    )
    .unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-sec1",
            "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();
    let legacy = TrustStore {
        entries: vec![
            TrustEntry {
                identity_id: LOCAL_TEST_KEY_REF.into(),
                algorithm: "ed25519".into(),
                public_key_hex: local_test_public_key_hex(),
                supersedes: None,
                previous_public_key_hex: None,
                previous_grace_until: None,
            },
            TrustEntry {
                identity_id: id.into(),
                algorithm: "ed25519".into(),
                public_key_hex: pub_hex.clone(),
                supersedes: None,
                previous_public_key_hex: None,
                previous_grace_until: None,
            },
        ],
        revoked: vec![],
    };
    legacy.save(root).unwrap();

    let store = ensure_trust_defaults(root).unwrap();
    assert!(!store
        .entries
        .iter()
        .any(|e| e.identity_id == LOCAL_TEST_KEY_REF));
    assert_eq!(store.entries.len(), 1);
    assert_eq!(store.entries[0].identity_id, id);
    let reloaded = TrustStore::load(root).unwrap();
    assert!(!reloaded
        .entries
        .iter()
        .any(|e| e.identity_id == LOCAL_TEST_KEY_REF));
    reset_primary_signer();
}

#[test]
fn trust_crl_revoke_blocks_readd_and_verify() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let peer_sk = SigningKey::from_bytes(&[17u8; 32]);
    let peer_id = "aira:identity:peer-bob";
    let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());

    let mut store = TrustStore::default();
    store.upsert(peer_id, &peer_pub).unwrap();
    store.save(root).unwrap();
    let _ = register_trust_store(root).unwrap();

    let msg = b"crl-message";
    let sig = sign_with_key(AiraRef::parse(peer_id).unwrap(), &peer_sk, msg);
    TrustStore::load(root)
        .unwrap()
        .to_keyring()
        .unwrap()
        .verify(&sig, msg)
        .unwrap();

    store.revoke(peer_id, Some("compromised")).unwrap();
    store.save(root).unwrap();
    let _ = sync_trust_verifiers(root).unwrap();
    assert!(store.is_revoked(peer_id));
    let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
    assert!(ring.verifying_key(peer_id).is_none());
    assert!(ring.verify(&sig, msg).is_err());
    assert_eq!(
        store.upsert(peer_id, &peer_pub),
        Err(CryptoError::RevokedKey(peer_id.into()))
    );
    assert!(TrustStore::default()
        .revoke(LOCAL_TEST_KEY_REF, None)
        .is_err());
    assert!(ring.verify(&local_test_signature(msg), msg).is_err());
}

#[test]
fn trust_crl_unrevoke_allows_explicit_readd() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let peer_sk = SigningKey::from_bytes(&[19u8; 32]);
    let peer_id = "aira:identity:peer-carol";
    let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());

    let mut store = TrustStore::default();
    store.upsert(peer_id, &peer_pub).unwrap();
    store.save(root).unwrap();
    let _ = register_trust_store(root).unwrap();

    let msg = b"unrevoke-message";
    let sig = sign_with_key(AiraRef::parse(peer_id).unwrap(), &peer_sk, msg);
    TrustStore::load(root)
        .unwrap()
        .to_keyring()
        .unwrap()
        .verify(&sig, msg)
        .unwrap();

    store.revoke(peer_id, Some("temp")).unwrap();
    store.save(root).unwrap();
    let _ = sync_trust_verifiers(root).unwrap();
    assert_eq!(
        store.upsert(peer_id, &peer_pub),
        Err(CryptoError::RevokedKey(peer_id.into()))
    );

    store.unrevoke(peer_id).unwrap();
    assert!(!store.is_revoked(peer_id));
    assert_eq!(
        store.unrevoke(peer_id),
        Err(CryptoError::NotRevoked(peer_id.into()))
    );
    // Unrevoke alone must not restore entries / verifying key.
    assert!(!store.entries.iter().any(|e| e.identity_id == peer_id));
    assert!(TrustStore::load(root)
        .unwrap()
        .to_keyring()
        .unwrap()
        .verifying_key(peer_id)
        .is_none());

    store.upsert(peer_id, &peer_pub).unwrap();
    store.save(root).unwrap();
    let _ = sync_trust_verifiers(root).unwrap();
    TrustStore::load(root)
        .unwrap()
        .to_keyring()
        .unwrap()
        .verify(&sig, msg)
        .unwrap();
}

#[test]
fn trust_rotate_revokes_old_trusts_new() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[21u8; 32]);
    let new_sk = SigningKey::from_bytes(&[23u8; 32]);
    let old_id = "aira:identity:peer-old";
    let new_id = "aira:identity:peer-new";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

    let mut store = TrustStore::default();
    store.upsert(old_id, &old_pub).unwrap();
    store.save(root).unwrap();
    let _ = register_trust_store(root).unwrap();

    let msg = b"rotate-message";
    let old_sig = sign_with_key(AiraRef::parse(old_id).unwrap(), &old_sk, msg);
    let new_sig = sign_with_key(AiraRef::parse(new_id).unwrap(), &new_sk, msg);
    TrustStore::load(root)
        .unwrap()
        .to_keyring()
        .unwrap()
        .verify(&old_sig, msg)
        .unwrap();

    store
        .rotate(old_id, new_id, &new_pub, Some("key rollover"), None)
        .unwrap();
    store.save(root).unwrap();
    let _ = sync_trust_verifiers(root).unwrap();

    assert!(store.is_revoked(old_id));
    let revoked = store
        .revoked
        .iter()
        .find(|r| r.identity_id == old_id)
        .unwrap();
    assert_eq!(revoked.superseded_by.as_deref(), Some(new_id));
    let entry = store
        .entries
        .iter()
        .find(|e| e.identity_id == new_id)
        .unwrap();
    assert_eq!(entry.supersedes.as_deref(), Some(old_id));

    let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
    assert!(ring.verifying_key(old_id).is_none());
    assert!(ring.verifying_key(new_id).is_some());
    assert!(ring.verify(&old_sig, msg).is_err());
    ring.verify(&new_sig, msg).unwrap();
    assert_eq!(
        store.upsert(old_id, &old_pub),
        Err(CryptoError::RevokedKey(old_id.into()))
    );
    assert_eq!(
        store.rotate(old_id, new_id, &new_pub, None, None),
        Err(CryptoError::NotTrusted(old_id.into()))
    );
    assert_eq!(
        store.rotate(new_id, new_id, &new_pub, None, None),
        Err(CryptoError::SameIdentity)
    );
    assert!(store
        .rotate(LOCAL_TEST_KEY_REF, new_id, &new_pub, None, None)
        .is_err());
}

#[test]
fn trust_rotate_grace_allows_old_until() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[25u8; 32]);
    let new_sk = SigningKey::from_bytes(&[27u8; 32]);
    let old_id = "aira:identity:peer-grace-old";
    let new_id = "aira:identity:peer-grace-new";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

    let mut store = TrustStore::default();
    store.upsert(old_id, &old_pub).unwrap();
    store.save(root).unwrap();

    let msg = b"grace-message";
    let old_sig = sign_with_key(AiraRef::parse(old_id).unwrap(), &old_sk, msg);
    let new_sig = sign_with_key(AiraRef::parse(new_id).unwrap(), &new_sk, msg);

    store
        .rotate(
            old_id,
            new_id,
            &new_pub,
            Some("grace rollover"),
            Some("2099-01-01T00:00:00Z"),
        )
        .unwrap();
    store.save(root).unwrap();

    let during = store.to_keyring_at("2026-07-16T12:00:00Z").unwrap();
    during.verify(&old_sig, msg).unwrap();
    during.verify(&new_sig, msg).unwrap();
    assert_eq!(
        store.upsert(old_id, &old_pub),
        Err(CryptoError::RevokedKey(old_id.into()))
    );

    let after = store.to_keyring_at("2099-01-01T00:00:01Z").unwrap();
    assert!(after.verifying_key(old_id).is_none());
    assert!(after.verify(&old_sig, msg).is_err());
    after.verify(&new_sig, msg).unwrap();

    assert!(store
        .rotate(old_id, new_id, &new_pub, None, Some("not-a-timestamp"))
        .is_err());
}

#[test]
fn trust_rekey_grace_allows_old_same_id() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[41u8; 32]);
    let new_sk = SigningKey::from_bytes(&[43u8; 32]);
    let id = "aira:identity:peer-rekey-grace";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

    let mut store = TrustStore::default();
    store.upsert(id, &old_pub).unwrap();
    store.save(root).unwrap();

    let msg = b"same-id-grace";
    let old_sig = sign_with_key(AiraRef::parse(id).unwrap(), &old_sk, msg);
    let new_sig = sign_with_key(AiraRef::parse(id).unwrap(), &new_sk, msg);

    store
        .rekey(id, &new_pub, Some("2099-06-01T00:00:00Z"))
        .unwrap();
    store.save(root).unwrap();

    let entry = store.entries.iter().find(|e| e.identity_id == id).unwrap();
    assert_eq!(entry.public_key_hex, new_pub);
    assert_eq!(
        entry.previous_public_key_hex.as_deref(),
        Some(old_pub.as_str())
    );
    assert_eq!(
        entry.previous_grace_until.as_deref(),
        Some("2099-06-01T00:00:00Z")
    );

    let during = store.to_keyring_at("2026-07-28T12:00:00Z").unwrap();
    assert_eq!(during.verifying_keys(id).len(), 2);
    during.verify(&old_sig, msg).unwrap();
    during.verify(&new_sig, msg).unwrap();

    let after = store.to_keyring_at("2099-06-01T00:00:01Z").unwrap();
    assert_eq!(after.verifying_keys(id).len(), 1);
    assert!(after.verify(&old_sig, msg).is_err());
    after.verify(&new_sig, msg).unwrap();

    // Immediate cutover clears previous_*.
    store.rekey(id, &old_pub, None).unwrap();
    let e2 = store.entries.iter().find(|e| e.identity_id == id).unwrap();
    assert_eq!(e2.public_key_hex, old_pub);
    assert!(e2.previous_public_key_hex.is_none());
    assert!(e2.previous_grace_until.is_none());
}

#[test]
fn node_signing_secret_rotate_cutover() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[31u8; 32]);
    let new_sk = SigningKey::from_bytes(&[33u8; 32]);
    let id = "aira:identity:node-rotate";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    fs::write(
        root.join("identity/local.ed25519"),
        format!("{}\n", hex::encode(old_sk.to_bytes())),
    )
    .unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-rotate",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();

    let msg = b"node-rotate-message";
    let old_sig = sign_with_key(AiraRef::parse(id).unwrap(), &old_sk, msg);
    let (loaded, ring) = Keyring::load_node_identity(root).unwrap();
    assert_eq!(loaded.as_str(), id);
    ring.verify(&old_sig, msg).unwrap();

    let (rotated_id, new_pub, reported_old, backup_path) =
        rotate_node_signing_secret(root, new_sk.clone(), false, None).unwrap();
    assert_eq!(rotated_id.as_str(), id);
    assert_eq!(reported_old, old_pub);
    assert!(backup_path.is_none());
    assert!(!root.join("identity").join(NODE_SECRET_BACKUP_FILE).exists());
    assert_eq!(new_pub, hex::encode(new_sk.verifying_key().to_bytes()));
    // File-backed cutover (process keyring is shared across parallel tests).
    let (reloaded, new_ring) = Keyring::load_node_identity(root).unwrap();
    assert_eq!(reloaded.as_str(), id);
    let new_sig = new_ring.sign(&reloaded, msg).unwrap();
    new_ring.verify(&new_sig, msg).unwrap();
    assert!(new_ring.verify(&old_sig, msg).is_err());

    let store = TrustStore::load(root).unwrap();
    let entry = store
        .entries
        .iter()
        .find(|e| e.identity_id == id)
        .expect("node trust entry");
    assert_eq!(entry.public_key_hex, new_pub);
    assert!(!store.is_revoked(id));
    let audit = crate::audit::TrustAuditLog::load(root).unwrap();
    assert!(audit.iter().any(|e| {
        e.action == crate::audit::TrustAuditAction::NodeRotate
            && e.subject_id == id
            && e.public_key_hex.as_deref() == Some(new_pub.as_str())
            && e.source.as_deref() == Some("node-rotate")
    }));

    let desc_raw = fs::read_to_string(root.join("identity/local.identity.json")).unwrap();
    let desc: serde_json::Value = serde_json::from_str(&desc_raw).unwrap();
    assert_eq!(desc["identity_id"], id);
    assert_eq!(desc["display_name"], "node-rotate");
    assert!(desc.get("rotated_at").is_some());

    reset_primary_signer();
}

#[test]
fn node_rotate_rolls_back_when_node_revoked() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[37u8; 32]);
    let new_sk = SigningKey::from_bytes(&[39u8; 32]);
    let id = "aira:identity:node-rollback";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
    fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
    let old_json = serde_json::json!({
        "identity_id": id,
        "identity_type": "local",
        "display_name": "node-rollback",
        "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
        "created_at": "2026-07-16T00:00:00Z",
        "key_path": "identity/local.ed25519"
    })
    .to_string();
    fs::write(root.join("identity/local.identity.json"), &old_json).unwrap();
    let mut store = TrustStore::default();
    store.upsert(id, &old_pub).unwrap();
    store.revoke(id, Some("block rotate")).unwrap();
    store.save(root).unwrap();

    let err = rotate_node_signing_secret(root, new_sk, false, None).unwrap_err();
    assert_eq!(err, CryptoError::RevokedKey(id.into()));
    assert_eq!(
        fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
        old_secret
    );
    let restored: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("identity/local.identity.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(restored["public_key"]["key_hex"], old_pub);
    assert!(restored.get("rotated_at").is_none());
    reset_primary_signer();
}

#[test]
fn node_rotate_requires_existing_identity() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let err = rotate_node_signing_secret(root, SigningKey::from_bytes(&[35u8; 32]), false, None)
        .unwrap_err();
    assert!(matches!(err, CryptoError::Io(_)));
}

#[test]
fn node_rotate_backup_writes_prev() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[41u8; 32]);
    let new_sk = SigningKey::from_bytes(&[43u8; 32]);
    let id = "aira:identity:node-backup";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
    fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-backup",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();

    let (rotated_id, new_pub, reported_old, backup_path) =
        rotate_node_signing_secret(root, new_sk.clone(), true, None).unwrap();
    assert_eq!(rotated_id.as_str(), id);
    assert_eq!(reported_old, old_pub);
    let backup = backup_path.expect("backup path");
    assert_eq!(backup, root.join("identity").join(NODE_SECRET_BACKUP_FILE));
    assert_eq!(fs::read_to_string(&backup).unwrap(), old_secret);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&backup).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    let meta: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_META_FILE)).unwrap(),
    )
    .unwrap();
    assert_eq!(meta["old_public_key_hex"], old_pub);
    assert_eq!(meta["identity_id"], id);
    assert_ne!(
        fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
        old_secret
    );
    assert_eq!(new_pub, hex::encode(new_sk.verifying_key().to_bytes()));
    reset_primary_signer();
}

#[test]
fn node_rotate_backup_archives_prior_slot() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let sk1 = SigningKey::from_bytes(&[61u8; 32]);
    let sk2 = SigningKey::from_bytes(&[62u8; 32]);
    let sk3 = SigningKey::from_bytes(&[63u8; 32]);
    let id = "aira:identity:node-backup-hist";
    let pub1 = hex::encode(sk1.verifying_key().to_bytes());
    let secret1 = format!("{}\n", hex::encode(sk1.to_bytes()));
    fs::write(root.join("identity/local.ed25519"), &secret1).unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-backup-hist",
            "public_key": { "algorithm": "ed25519", "key_hex": pub1 },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();

    rotate_node_signing_secret(root, sk2.clone(), true, None).unwrap();
    let first_prev =
        fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap();
    assert_eq!(first_prev, secret1);

    let secret2 = format!("{}\n", hex::encode(sk2.to_bytes()));
    rotate_node_signing_secret(root, sk3.clone(), true, None).unwrap();
    let latest = fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap();
    assert_eq!(latest, secret2);
    assert_ne!(latest, secret1);

    let list = list_node_secret_backups(root).unwrap();
    assert!(list.iter().any(|b| b.is_latest));
    assert!(
        list.iter().any(|b| !b.is_latest),
        "expected archived timestamped backup"
    );
    let archived = list.iter().find(|b| !b.is_latest).unwrap();
    assert_eq!(fs::read_to_string(&archived.secret_path).unwrap(), secret1);
    assert_eq!(archived.old_public_key_hex.as_deref(), Some(pub1.as_str()));
    // Both secrets still recoverable.
    let secrets: Vec<_> = list
        .iter()
        .map(|b| fs::read_to_string(&b.secret_path).unwrap())
        .collect();
    assert!(secrets.iter().any(|s| s == &secret1));
    assert!(secrets.iter().any(|s| s == &secret2));
    reset_primary_signer();
}

#[test]
fn node_rotate_backup_fail_closed() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[45u8; 32]);
    let new_sk = SigningKey::from_bytes(&[47u8; 32]);
    let id = "aira:identity:node-backup-fail";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
    fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-backup-fail",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();
    // Make backup staging path a directory so stage write fails.
    fs::create_dir_all(root.join("identity").join("local.ed25519.prev.tmp")).unwrap();

    let err = rotate_node_signing_secret(root, new_sk, true, None).unwrap_err();
    assert!(matches!(err, CryptoError::Io(_)));
    assert_eq!(
        fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
        old_secret
    );
    assert!(!root.join("identity").join(NODE_SECRET_BACKUP_FILE).exists());
    assert!(!root.join("identity/local.ed25519.prev.tmp").exists());
    let desc: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("identity/local.identity.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(desc["public_key"]["key_hex"], old_pub);
    assert!(desc.get("rotated_at").is_none());
    reset_primary_signer();
}

#[test]
fn node_rotate_backup_preserves_prev_slot_on_trust_fail() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[49u8; 32]);
    let new_sk = SigningKey::from_bytes(&[51u8; 32]);
    let id = "aira:identity:node-backup-keep";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
    fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-backup-keep",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();
    let prior = b"prior-backup-secret\n";
    fs::write(root.join("identity").join(NODE_SECRET_BACKUP_FILE), prior).unwrap();
    let mut store = TrustStore::default();
    store.upsert(id, &old_pub).unwrap();
    store.revoke(id, Some("block")).unwrap();
    store.save(root).unwrap();

    let err = rotate_node_signing_secret(root, new_sk, true, None).unwrap_err();
    assert_eq!(err, CryptoError::RevokedKey(id.into()));
    assert_eq!(
        fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap(),
        String::from_utf8_lossy(prior)
    );
    assert!(!root.join("identity/local.ed25519.prev.tmp").exists());
    assert_eq!(
        fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
        old_secret
    );
    reset_primary_signer();
}

#[test]
fn node_rotate_backup_commit_clears_prev_dir_trap() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[53u8; 32]);
    let new_sk = SigningKey::from_bytes(&[55u8; 32]);
    let id = "aira:identity:node-backup-dirtrap";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
    fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-backup-dirtrap",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();
    fs::create_dir_all(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap();

    let (rotated_id, new_pub, _, backup_path) =
        rotate_node_signing_secret(root, new_sk.clone(), true, None).unwrap();
    assert_eq!(rotated_id.as_str(), id);
    assert_eq!(new_pub, hex::encode(new_sk.verifying_key().to_bytes()));
    let backup = backup_path.expect("backup path");
    assert_eq!(backup, root.join("identity").join(NODE_SECRET_BACKUP_FILE));
    assert!(backup.is_file());
    assert_eq!(fs::read_to_string(&backup).unwrap(), old_secret);
    let store = TrustStore::load(root).unwrap();
    let entry = store.entries.iter().find(|e| e.identity_id == id).unwrap();
    assert_eq!(entry.public_key_hex, new_pub);
    let (loaded, ring) = Keyring::load_node_identity(root).unwrap();
    assert_eq!(loaded.as_str(), id);
    let msg = b"dirtrap-ok";
    let sig = ring.sign(&loaded, msg).unwrap();
    ring.verify(&sig, msg).unwrap();
    reset_primary_signer();
}

#[test]
fn node_rotate_grace_allows_old_until() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[57u8; 32]);
    let new_sk = SigningKey::from_bytes(&[59u8; 32]);
    let id = "aira:identity:node-grace";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    fs::write(
        root.join("identity/local.ed25519"),
        format!("{}\n", hex::encode(old_sk.to_bytes())),
    )
    .unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-grace",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();
    let _ = ensure_trust_defaults(root).unwrap();

    let msg = b"node-grace-message";
    let old_sig = sign_with_key(AiraRef::parse(id).unwrap(), &old_sk, msg);
    let until = "2099-01-01T00:00:00Z";
    let (rotated_id, _new_pub, _, _) =
        rotate_node_signing_secret(root, new_sk.clone(), false, Some(until)).unwrap();
    assert_eq!(rotated_id.as_str(), id);

    let (reloaded, ring) = Keyring::load_node_identity(root).unwrap();
    assert_eq!(ring.verifying_keys(id).len(), 2);
    ring.verify(&old_sig, msg).unwrap();
    let new_sig = ring.sign(&reloaded, msg).unwrap();
    ring.verify(&new_sig, msg).unwrap();

    let desc_raw = fs::read_to_string(root.join("identity/local.identity.json")).unwrap();
    let desc: serde_json::Value = serde_json::from_str(&desc_raw).unwrap();
    assert_eq!(desc["previous_public_key"]["key_hex"], old_pub);
    assert_eq!(desc["previous_grace_until"], until);

    // Expired grace: rewrite until to the past and reload.
    let mut desc_obj = desc.as_object().unwrap().clone();
    desc_obj.insert(
        "previous_grace_until".into(),
        serde_json::json!("2020-01-01T00:00:00Z"),
    );
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::to_string_pretty(&desc_obj).unwrap(),
    )
    .unwrap();
    let (_, expired) = Keyring::load_node_identity(root).unwrap();
    assert_eq!(expired.verifying_keys(id).len(), 1);
    assert!(expired.verify(&old_sig, msg).is_err());
    reset_primary_signer();
}

#[test]
fn node_rotate_rejects_bad_grace_until() {
    let _lock = lock_process_crypto();
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("identity")).unwrap();
    let old_sk = SigningKey::from_bytes(&[61u8; 32]);
    let new_sk = SigningKey::from_bytes(&[63u8; 32]);
    let id = "aira:identity:node-bad-grace";
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    fs::write(
        root.join("identity/local.ed25519"),
        format!("{}\n", hex::encode(old_sk.to_bytes())),
    )
    .unwrap();
    fs::write(
        root.join("identity/local.identity.json"),
        serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-bad-grace",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string(),
    )
    .unwrap();
    let err = rotate_node_signing_secret(root, new_sk, false, Some("not-a-time")).unwrap_err();
    assert!(matches!(err, CryptoError::InvalidTimestamp(_)));
    reset_primary_signer();
}

#[test]
fn prune_keep_one_retains_newest_archive_and_latest() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let idir = root.join("identity");
    fs::create_dir_all(&idir).unwrap();
    fs::write(idir.join(NODE_SECRET_BACKUP_FILE), b"latest-secret\n").unwrap();
    fs::write(
        idir.join(NODE_SECRET_BACKUP_META_FILE),
        serde_json::json!({"backed_up_at":"2026-08-01T12:00:00Z"}).to_string(),
    )
    .unwrap();
    fs::write(idir.join("local.ed25519.prev.20260101T000000Z"), b"old\n").unwrap();
    fs::write(
        idir.join("local.ed25519.prev.20260101T000000Z.meta.json"),
        serde_json::json!({"backed_up_at":"2026-01-01T00:00:00Z"}).to_string(),
    )
    .unwrap();
    fs::write(idir.join("local.ed25519.prev.20260701T000000Z"), b"mid\n").unwrap();
    fs::write(
        idir.join("local.ed25519.prev.20260701T000000Z.meta.json"),
        serde_json::json!({"backed_up_at":"2026-07-01T00:00:00Z"}).to_string(),
    )
    .unwrap();

    let r = prune_node_secret_backups(root, Some(1), None, false).unwrap();
    assert!(idir.join(NODE_SECRET_BACKUP_FILE).is_file());
    assert!(idir.join("local.ed25519.prev.20260701T000000Z").is_file());
    assert!(!idir.join("local.ed25519.prev.20260101T000000Z").is_file());
    assert!(!idir
        .join("local.ed25519.prev.20260101T000000Z.meta.json")
        .is_file());
    assert!(r
        .deleted
        .iter()
        .any(|p| p.ends_with("local.ed25519.prev.20260101T000000Z")));
}

#[test]
fn prune_older_than_days_skips_unparseable_when_ttl_set() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let idir = root.join("identity");
    fs::create_dir_all(&idir).unwrap();
    fs::write(idir.join(NODE_SECRET_BACKUP_FILE), b"latest\n").unwrap();
    fs::write(idir.join("local.ed25519.prev.notastamp"), b"bad\n").unwrap();
    // stamp with dot is ignored by list; use valid-looking but unparseable age via no meta
    // and stamp that fails compact parse: use letters in stamp after filtering — list requires
    // no '.' in stamp. "ABCDEFGHIJKLMNZ" length wrong. Use 16-char invalid month.
    fs::write(
        idir.join("local.ed25519.prev.20261301T000000Z"),
        b"bad-month\n",
    )
    .unwrap();

    let r = prune_node_secret_backups(root, None, Some(31), false).unwrap();
    assert!(idir.join("local.ed25519.prev.20261301T000000Z").is_file());
    assert!(r.skipped.iter().any(|(_, w)| w.contains("unparseable")));
    assert!(idir.join(NODE_SECRET_BACKUP_FILE).is_file());
}

#[test]
fn prune_dry_run_deletes_nothing() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let idir = root.join("identity");
    fs::create_dir_all(&idir).unwrap();
    fs::write(idir.join(NODE_SECRET_BACKUP_FILE), b"latest\n").unwrap();
    fs::write(idir.join("local.ed25519.prev.20260101T000000Z"), b"old\n").unwrap();
    let r = prune_node_secret_backups(root, Some(0), None, true).unwrap();
    assert!(r.dry_run);
    assert!(idir.join("local.ed25519.prev.20260101T000000Z").is_file());
    assert!(!r.deleted.is_empty());
}

#[test]
fn prune_requires_policy_flag() {
    let dir = tempdir().unwrap();
    let err = prune_node_secret_backups(dir.path(), None, None, false).unwrap_err();
    assert!(err.to_string().contains("requires"));
}

#[test]
fn prune_never_deletes_orphan_meta() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let idir = root.join("identity");
    fs::create_dir_all(&idir).unwrap();
    let orphan = idir.join("local.ed25519.prev.20260101T000000Z.meta.json");
    fs::write(&orphan, "{}").unwrap();
    let r = prune_node_secret_backups(root, Some(0), None, false).unwrap();
    assert!(orphan.is_file());
    assert!(r.skipped.iter().any(|(_, w)| w == "orphan-meta"));
}
