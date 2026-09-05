//! Peer module integration tests (QUEUE #128 — split from `lib.rs`).

use super::*;
use std::fs;

use aira_flow::{init_node, NodePaths};
use aira_object::{
    ensure_trust_defaults, sign_with_key, AiraRef, ContentHash, Keyring, Timestamp, TrustStore,
};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;
use tempfile::tempdir;

fn write_node_identity(root: &std::path::Path, name: &str, seed: [u8; 32]) -> (AiraRef, String) {
    let paths = NodePaths::new(root);
    fs::create_dir_all(paths.identity_dir()).unwrap();
    let sk = SigningKey::from_bytes(&seed);
    let pub_hex = hex::encode(sk.verifying_key().to_bytes());
    let id = format!("aira:identity:{name}");
    let id_ref = AiraRef::parse(&id).unwrap();
    fs::write(
        paths.identity_key(),
        format!("{}\n", hex::encode(sk.to_bytes())),
    )
    .unwrap();
    let sig = sign_with_key(id_ref.clone(), &sk, id.as_bytes());
    let desc = serde_json::json!({
        "identity_id": id,
        "identity_type": "local",
        "display_name": name,
        "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
        "created_at": "2026-07-16T00:00:00Z",
        "key_path": "identity/local.ed25519",
        "signature": sig
    });
    fs::write(
        paths.identity_json(),
        serde_json::to_string_pretty(&desc).unwrap(),
    )
    .unwrap();
    let _ = ensure_trust_defaults(root).unwrap();
    (id_ref, pub_hex)
}

fn mutual_trust(
    a_root: &std::path::Path,
    a_id: &str,
    a_pub: &str,
    b_root: &std::path::Path,
    b_id: &str,
    b_pub: &str,
) {
    let mut ta = TrustStore::load(a_root).unwrap();
    ta.upsert(b_id, b_pub).unwrap();
    ta.save(a_root).unwrap();
    let mut tb = TrustStore::load(b_root).unwrap();
    tb.upsert(a_id, a_pub).unwrap();
    tb.save(b_root).unwrap();
}

fn make_envelope(issuer: &AiraRef, ring: &Keyring, payload: &str) -> ProtocolEnvelope {
    let hash = ContentHash::sha256_bytes(payload.as_bytes());
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id =
        AiraRef::parse(format!("aira:message:peer-ping-{}", hex::encode(nonce))).unwrap();
    ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: "peer.ping".into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: issuer.clone(),
        target_scope: ScopeDescriptor::local("peer-p0"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: None,
        created_at: aira_object::now(),
        expires_at: None,
        signature: ProtocolEnvelope::placeholder_signature(issuer),
    }
    .attach_canonical_signature_with_keyring(ring, issuer)
    .unwrap()
}

#[tokio::test]
async fn trusted_peers_hello_and_envelope_roundtrip() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice", [11u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob", [13u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });

    let mut client = dial(root_a, id_b.as_str()).await.unwrap();
    assert_eq!(client.peer_id, id_b);
    let mut server = accept_task.await.unwrap().unwrap();
    assert_eq!(server.peer_id, id_a);

    let (_ida, ring_a) = Keyring::load_node_identity(root_a).unwrap();
    let env = make_envelope(&id_a, &ring_a, "ping-payload");
    client.send_envelope(&env).await.unwrap();
    let got = server.recv_envelope().await.unwrap();
    assert_eq!(got.issuer_identity, id_a);
    assert_eq!(got.message_type, "peer.ping");
}

#[tokio::test]
async fn recv_envelope_rejects_expired() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-exp", [41u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-exp", [43u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });

    let mut client = dial(root_a, id_b.as_str()).await.unwrap();
    let mut server = accept_task.await.unwrap().unwrap();

    let (_ida, ring_a) = Keyring::load_node_identity(root_a).unwrap();
    let hash = ContentHash::sha256_bytes(b"expired");
    let env = ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: "peer.ping".into(),
        message_id: AiraRef::parse("aira:message:peer-expired-1").unwrap(),
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: id_a.clone(),
        target_scope: ScopeDescriptor::local("peer-p0"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: None,
        created_at: aira_object::now(),
        expires_at: Some("2020-01-01T00:00:00Z".into()),
        signature: ProtocolEnvelope::placeholder_signature(&id_a),
    }
    .attach_canonical_signature_with_keyring(&ring_a, &id_a)
    .unwrap();
    client.send_envelope(&env).await.unwrap();
    let err = server.recv_envelope().await.unwrap_err();
    assert!(
        matches!(err, PeerError::Expired),
        "expected Expired, got {err}"
    );
}

#[tokio::test]
async fn recv_envelope_rejects_replayed_message_id() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-rp", [51u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-rp", [53u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });

    let mut client = dial(root_a, id_b.as_str()).await.unwrap();
    let mut server = accept_task.await.unwrap().unwrap();

    let (_ida, ring_a) = Keyring::load_node_identity(root_a).unwrap();
    let env = make_envelope(&id_a, &ring_a, "replay-payload");
    client.send_envelope(&env).await.unwrap();
    server.recv_envelope().await.unwrap();
    client.send_envelope(&env).await.unwrap();
    let err = server.recv_envelope().await.unwrap_err();
    assert!(
        matches!(err, PeerError::Replay(_)),
        "expected Replay, got {err}"
    );
}

#[tokio::test]
async fn listen_accepts_multiple_hello_only_dials() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-d", [31u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-d", [33u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_loop = tokio::spawn(async move {
        let mut n = 0usize;
        while n < 2 {
            let peer = accept(&listener, &root_b2).await.unwrap();
            assert_eq!(peer.peer_id, id_a);
            // Hello-only: drop without recv_envelope (dial smoke).
            drop(peer);
            n += 1;
        }
        n
    });

    let d1 = dial(root_a, id_b.as_str()).await.unwrap();
    assert_eq!(d1.peer_id, id_b);
    drop(d1);
    let d2 = dial(root_a, id_b.as_str()).await.unwrap();
    assert_eq!(d2.peer_id, id_b);
    drop(d2);

    assert_eq!(accept_loop.await.unwrap(), 2);
}

#[test]
fn noise_static_bind_rejects_mismatch() {
    let expected = [1u8; 32];
    let actual = [2u8; 32];
    let err = crate::session::ensure_noise_static_bind(&expected, &actual).unwrap_err();
    assert!(matches!(err, PeerError::Handshake(_)));
    crate::session::ensure_noise_static_bind(&expected, &expected).unwrap();
}

#[test]
fn noise_static_file_created_mode_600() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let _ = load_or_create_noise_static(root).unwrap();
    let path = root.join("identity").join("local.x25519");
    assert!(path.is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    // Idempotent reload.
    let a = load_or_create_noise_static(root).unwrap();
    let b = load_or_create_noise_static(root).unwrap();
    assert_eq!(a, b);
}

#[test]
fn noise_static_rotate_changes_secret_and_backup() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let before = load_or_create_noise_static(root).unwrap();
    let expected_old = hex::encode(x25519_public(&before));
    let r1 = rotate_noise_static(root, true).unwrap();
    let after = load_or_create_noise_static(root).unwrap();
    assert_ne!(before, after);
    assert_eq!(r1.old_public_hex.as_deref(), Some(expected_old.as_str()));
    assert_eq!(r1.new_public_hex, hex::encode(x25519_public(&after)));
    let prev = root.join("identity").join(NODE_X25519_BACKUP_FILE);
    assert!(prev.is_file());
    assert_eq!(r1.backup_path.as_deref(), Some(prev.as_path()));
    let prev_bytes = hex::decode(fs::read_to_string(&prev).unwrap().trim()).unwrap();
    assert_eq!(prev_bytes, before);

    // Second rotate with backup archives the prior .prev.
    let r2 = rotate_noise_static(root, true).unwrap();
    assert!(prev.is_file());
    let archived: Vec<_> = fs::read_dir(root.join("identity"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("local.x25519.prev.") && !n.ends_with(".meta.json"))
        .collect();
    assert_eq!(archived.len(), 1, "{archived:?}");
    assert!(r2.backup_path.is_some());
}

#[test]
fn prune_noise_static_keep_zero_drops_archives_keeps_latest() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let idir = root.join("identity");
    fs::write(idir.join(NODE_X25519_BACKUP_FILE), b"aa\n").unwrap();
    fs::write(idir.join("local.x25519.prev.1000Z"), b"old\n").unwrap();
    fs::write(idir.join("local.x25519.prev.2000Z"), b"newer\n").unwrap();
    let r = prune_noise_static_backups(root, Some(0), None, false).unwrap();
    assert!(idir.join(NODE_X25519_BACKUP_FILE).is_file());
    assert!(!idir.join("local.x25519.prev.1000Z").is_file());
    assert!(!idir.join("local.x25519.prev.2000Z").is_file());
    assert_eq!(r.deleted.len(), 2);
}

#[test]
fn prune_noise_static_older_than_days_and_dry_run() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let idir = root.join("identity");
    fs::write(idir.join(NODE_X25519_BACKUP_FILE), b"latest\n").unwrap();
    // Very old unix stamp.
    fs::write(idir.join("local.x25519.prev.1Z"), b"ancient\n").unwrap();
    let dry = prune_noise_static_backups(root, None, Some(1), true).unwrap();
    assert!(dry.dry_run);
    assert!(idir.join("local.x25519.prev.1Z").is_file());
    assert!(!dry.deleted.is_empty());
    let r = prune_noise_static_backups(root, None, Some(1), false).unwrap();
    assert!(!idir.join("local.x25519.prev.1Z").is_file());
    assert!(idir.join(NODE_X25519_BACKUP_FILE).is_file());
    assert!(!r.deleted.is_empty());
}

#[test]
fn prune_noise_static_unparseable_age_skipped() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let idir = root.join("identity");
    fs::write(idir.join(NODE_X25519_BACKUP_FILE), b"latest\n").unwrap();
    fs::write(idir.join("local.x25519.prev.notdigitsZ"), b"bad\n").unwrap();
    let r = prune_noise_static_backups(root, None, Some(1), false).unwrap();
    assert!(idir.join("local.x25519.prev.notdigitsZ").is_file());
    assert!(r.skipped.iter().any(|(_, w)| w.contains("unparseable")));
}

#[test]
fn prune_noise_static_requires_policy() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let err = prune_noise_static_backups(root, None, None, false).unwrap_err();
    assert!(err.to_string().contains("requires"));
}

#[test]
fn noise_static_rotate_without_prior_creates_fresh() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let r = rotate_noise_static(root, true).unwrap();
    assert!(r.old_public_hex.is_none());
    assert!(r.backup_path.is_none());
    assert!(root.join("identity/local.x25519").is_file());
    assert!(!root.join("identity").join(NODE_X25519_BACKUP_FILE).exists());
}

#[tokio::test]
async fn local_test_identity_rejected_at_handshake() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (_id, _pub) = write_node_identity(root, "victim-lt", [29u8; 32]);

    // Legacy trust.json that still lists local-test (pre-SEC-1 installs).
    let legacy = serde_json::json!({
        "entries": [{
            "identity_id": aira_object::LOCAL_TEST_KEY_REF,
            "algorithm": "ed25519",
            "public_key_hex": aira_object::local_test_public_key_hex()
        }],
        "revoked": []
    });
    fs::write(
        NodePaths::new(root).trust_json(),
        format!("{}\n", serde_json::to_string_pretty(&legacy).unwrap()),
    )
    .unwrap();

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let root_v = root.to_path_buf();
    let accept_task = tokio::spawn(async move {
        let stream = accept_tcp(&listener).await.unwrap();
        complete_accept(stream, &root_v).await
    });

    let nonce_hex = "00112233445566778899001122334455";
    let x25519_pub_hex = "00".repeat(64);
    let hello_bytes = format!(
        "{}|client|{}|{}|{}",
        crate::handshake::HELLO_DOMAIN,
        aira_object::LOCAL_TEST_KEY_REF,
        nonce_hex,
        x25519_pub_hex
    );
    let hello = crate::handshake::HelloMessage {
        role: "client".into(),
        identity_id: aira_object::LOCAL_TEST_KEY_REF.into(),
        nonce_hex: nonce_hex.into(),
        peer_nonce_hex: None,
        x25519_pub_hex,
        signature: aira_object::local_test_signature(hello_bytes.as_bytes()),
    };

    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    crate::frame::write_json(&mut stream, &hello).await.unwrap();

    let err = accept_task.await.unwrap().unwrap_err();
    assert!(
        matches!(err, PeerError::Untrusted(_)),
        "legacy local-test trust must not admit handshake: {err:?}"
    );
}

#[tokio::test]
async fn untrusted_peer_rejected_at_handshake() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, _pub_a) = write_node_identity(root_a, "alice-u", [17u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-u", [19u8; 32]);
    // A trusts B, but B does NOT trust A.
    let mut ta = TrustStore::load(root_a).unwrap();
    ta.upsert(id_b.as_str(), &pub_b).unwrap();
    ta.save(root_a).unwrap();

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });
    let dial_res = dial(root_a, id_b.as_str()).await;
    let accept_res = accept_task.await.unwrap();
    assert!(dial_res.is_err(), "dial must fail when responder rejects");
    assert!(
        accept_res.is_err(),
        "accept must fail when initiator is untrusted"
    );
    match accept_res.unwrap_err() {
        PeerError::Untrusted(_) | PeerError::Handshake(_) | PeerError::InvalidSignature => {}
        other => panic!("unexpected accept error: {other}"),
    }
    let _ = id_a;
}

#[tokio::test]
async fn revoked_peer_cannot_dial() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-r", [21u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-r", [23u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);
    let mut ta = TrustStore::load(root_a).unwrap();
    ta.revoke(id_b.as_str(), Some("compromised")).unwrap();
    ta.save(root_a).unwrap();

    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), "127.0.0.1:49157").unwrap();
    book.save(root_a).unwrap();
    let err = dial(root_a, id_b.as_str()).await.unwrap_err();
    assert!(matches!(err, PeerError::Revoked(_)));
}

#[tokio::test]
async fn envelope_issuer_mismatch_rejected() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-m", [25u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-m", [27u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });
    let mut client = dial(root_a, id_b.as_str()).await.unwrap();
    let mut server = accept_task.await.unwrap().unwrap();

    let (_ida, ring_a) = Keyring::load_node_identity(root_a).unwrap();
    let mut env = make_envelope(&id_a, &ring_a, "bad-issuer");
    // Forge issuer to local-test while keeping alice signature material shape —
    // send_envelope should refuse issuer != local_id.
    env.issuer_identity = AiraRef::parse(aira_object::LOCAL_TEST_KEY_REF).unwrap();
    let err = client.send_envelope(&env).await.unwrap_err();
    assert!(matches!(err, PeerError::IdentityMismatch));

    // Cleartext frame after Noise must fail closed on decrypt.
    crate::frame::write_frame(&mut client.stream, b"{\"not\":\"noise\"}")
        .await
        .unwrap();
    let err = server.recv_envelope().await.unwrap_err();
    assert!(
        matches!(err, PeerError::Crypto(_) | PeerError::Protocol(_)),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn frame_too_large_rejected() {
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        crate::frame::read_frame(&mut stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    let len = ((MAX_FRAME_BYTES as u32) + 1).to_be_bytes();
    client.write_all(&len).await.unwrap();
    let err = server.await.unwrap().unwrap_err();
    assert!(matches!(err, PeerError::FrameTooLarge(_)));
}

#[tokio::test]
async fn truncated_frame_rejected() {
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        crate::frame::read_frame(&mut stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
    // Claim 8 bytes, send only 2, then close.
    client.write_all(&8u32.to_be_bytes()).await.unwrap();
    client.write_all(&[0xaa, 0xbb]).await.unwrap();
    drop(client);
    let err = server.await.unwrap().unwrap_err();
    assert!(matches!(err, PeerError::TruncatedFrame | PeerError::Io(_)));
}

#[tokio::test]
async fn listen_rejects_non_loopback() {
    let err = listen("0.0.0.0:49157").await.unwrap_err();
    assert!(matches!(err, PeerError::Io(_)), "{err}");
}

#[tokio::test]
async fn listen_rejects_non_prime_port() {
    let err = listen("127.0.0.1:9797").await.unwrap_err();
    assert!(matches!(err, PeerError::InvalidPort(_)), "{err}");
    let err0 = listen("127.0.0.1:0").await.unwrap_err();
    assert!(matches!(err0, PeerError::InvalidPort(_)), "{err0}");
}

#[test]
fn make_peer_ping_signs_canonical_descriptor() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (id, _) = write_node_identity(root, "ping-builder", [61u8; 32]);
    let env = make_peer_ping(root, "hello-ping").unwrap();
    assert_eq!(env.message_type, "peer.ping");
    assert_eq!(env.issuer_identity, id);
    assert_eq!(env.signature.key_ref, id);
    assert_eq!(env.payload_ref.as_deref(), Some("hello-ping"));
    let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
    env.validate_signature_with_keyring(&ring).unwrap();
}

#[tokio::test]
async fn trust_delta_revoke_roundtrip_applies() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-td", [41u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-td", [43u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    // Alice self-announces revoke of her own identity (Analyze-52).
    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });

    let delta = TrustDelta::revoke(id_a.as_str(), Some("compromised".into()));
    let env = make_trust_delta_envelope(root_a, &delta).unwrap();
    let mut client = dial(root_a, id_b.as_str()).await.unwrap();
    client.send_envelope(&env).await.unwrap();
    let mut server = accept_task.await.unwrap().unwrap();
    let got = server.recv_envelope().await.unwrap();
    let parsed = parse_trust_delta(&got).unwrap();
    assert_eq!(parsed.op, TrustDeltaOp::Revoke);
    assert_eq!(parsed.subject_id, id_a.as_str());
    apply_trust_delta(root_b, &got.issuer_identity, &parsed).unwrap();

    let tb = TrustStore::load(root_b).unwrap();
    assert!(tb.is_revoked(id_a.as_str()));
    assert!(!tb.entries.iter().any(|e| e.identity_id == id_a.as_str()));
}

#[test]
fn trust_delta_refuses_local_test_and_local_node() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (id, id_pub) = write_node_identity(root, "solo-td", [51u8; 32]);
    let peer = "aira:identity:peer-td";
    let peer_sk = SigningKey::from_bytes(&[73u8; 32]);
    let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());
    let lt_pub = hex::encode(
        SigningKey::from_bytes(&[74u8; 32])
            .verifying_key()
            .to_bytes(),
    );
    let mut t = TrustStore::load(root).unwrap();
    t.upsert(peer, &peer_pub).unwrap();
    assert_eq!(
        t.upsert(aira_object::LOCAL_TEST_KEY_REF, &lt_pub),
        Err(aira_object::CryptoError::ProtectedIdentity(
            aira_object::LOCAL_TEST_KEY_REF.into()
        ))
    );
    t.save(root).unwrap();
    let _ = id_pub;

    // Third-party subject → IdentityMismatch (Analyze-52).
    let peer_issuer = AiraRef::parse(peer).unwrap();
    let err = apply_trust_delta(
        root,
        &peer_issuer,
        &TrustDelta::revoke(aira_object::LOCAL_TEST_KEY_REF, None),
    )
    .unwrap_err();
    assert!(matches!(err, PeerError::IdentityMismatch));
    let err =
        apply_trust_delta(root, &peer_issuer, &TrustDelta::revoke(id.as_str(), None)).unwrap_err();
    assert!(matches!(err, PeerError::IdentityMismatch));

    // Self-sovereign local-test issuer is untrusted (SEC-1); local-node still refused as protected.
    let lt_issuer = AiraRef::parse(aira_object::LOCAL_TEST_KEY_REF).unwrap();
    let err = apply_trust_delta(
        root,
        &lt_issuer,
        &TrustDelta::revoke(aira_object::LOCAL_TEST_KEY_REF, None),
    )
    .unwrap_err();
    assert!(matches!(err, PeerError::Untrusted(_)));
    let err = apply_trust_delta(root, &id, &TrustDelta::revoke(id.as_str(), None)).unwrap_err();
    assert!(matches!(err, PeerError::Protocol(_)));
}

#[test]
fn trust_delta_rotate_shape_and_apply() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (_id, _) = write_node_identity(root, "rot-host", [53u8; 32]);
    let old = "aira:identity:old-rot";
    let new = "aira:identity:new-rot";
    let old_sk = SigningKey::from_bytes(&[79u8; 32]);
    let new_sk = SigningKey::from_bytes(&[81u8; 32]);
    let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
    let new_pk = hex::encode(new_sk.verifying_key().to_bytes());
    let mut t = TrustStore::load(root).unwrap();
    t.upsert(old, &old_pub).unwrap();
    t.save(root).unwrap();
    let issuer = AiraRef::parse(old).unwrap();

    // Third-party rotate rejected.
    let stranger = "aira:identity:stranger-rot";
    let stranger_sk = SigningKey::from_bytes(&[77u8; 32]);
    let stranger_pub = hex::encode(stranger_sk.verifying_key().to_bytes());
    let mut t = TrustStore::load(root).unwrap();
    t.upsert(stranger, &stranger_pub).unwrap();
    t.save(root).unwrap();
    let stranger_issuer = AiraRef::parse(stranger).unwrap();
    let err = apply_trust_delta(
        root,
        &stranger_issuer,
        &TrustDelta::rotate(old, new, &new_pk, Some("rollover".into()), None),
    )
    .unwrap_err();
    assert!(matches!(err, PeerError::IdentityMismatch));

    let delta = TrustDelta::rotate(old, new, &new_pk, Some("rollover".into()), None);
    apply_trust_delta(root, &issuer, &delta).unwrap();
    let t = TrustStore::load(root).unwrap();
    assert!(t.is_revoked(old));
    assert!(t.entries.iter().any(|e| e.identity_id == new));
    let audit = aira_object::TrustAuditLog::load(root).unwrap();
    assert!(audit.iter().any(|e| {
        e.action == aira_object::TrustAuditAction::Rotate
            && e.subject_id == old
            && e.new_id.as_deref() == Some(new)
            && e.source.as_deref() == Some("peer-delta")
            && e.issuer_id.as_deref() == Some(old)
    }));
}

#[test]
fn trust_delta_bad_schema_rejected() {
    let mut d = TrustDelta::revoke("aira:identity:x", None);
    d.schema = "nope".into();
    assert!(d.validate_shape().is_err());
    assert!(TrustDeltaOp::parse("nope").is_err());
}

#[test]
fn trust_delta_ops_require_issuer_subject_match() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (_id, _) = write_node_identity(root, "host-rk", [91u8; 32]);
    let peer = "aira:identity:peer-rk";
    let other = "aira:identity:other-rk";
    let peer_sk = SigningKey::from_bytes(&[93u8; 32]);
    let other_sk = SigningKey::from_bytes(&[95u8; 32]);
    let new_sk = SigningKey::from_bytes(&[97u8; 32]);
    let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());
    let other_pub = hex::encode(other_sk.verifying_key().to_bytes());
    let new_pk = hex::encode(new_sk.verifying_key().to_bytes());
    let mut t = TrustStore::load(root).unwrap();
    t.upsert(peer, &peer_pub).unwrap();
    t.upsert(other, &other_pub).unwrap();
    t.save(root).unwrap();
    let issuer = AiraRef::parse(peer).unwrap();

    for bad in [
        TrustDelta::revoke(other, Some("nope".into())),
        TrustDelta::unrevoke(other),
        TrustDelta::rotate(other, "aira:identity:new-rk", &new_pk, None, None),
        TrustDelta::rekey(other, &new_pk, None, None),
    ] {
        let err = apply_trust_delta(root, &issuer, &bad).unwrap_err();
        assert!(
            matches!(err, PeerError::IdentityMismatch),
            "op={:?} err={err:?}",
            bad.op
        );
    }

    // Issuer rekeys self → upsert.
    let ok = TrustDelta::rekey(peer, &new_pk, Some("rotated".into()), None);
    apply_trust_delta(root, &issuer, &ok).unwrap();
    let t = TrustStore::load(root).unwrap();
    let entry = t.entries.iter().find(|e| e.identity_id == peer).unwrap();
    assert_eq!(entry.public_key_hex, new_pk);
}

#[test]
fn make_trust_delta_envelope_rejects_foreign_subject() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (id, _) = write_node_identity(root, "send-gate", [99u8; 32]);
    let foreign = "aira:identity:foreign-sg";
    let err = make_trust_delta_envelope(root, &TrustDelta::revoke(foreign, None)).unwrap_err();
    assert!(matches!(err, PeerError::IdentityMismatch));
    let ok = make_trust_delta_envelope(root, &TrustDelta::revoke(id.as_str(), Some("self".into())))
        .unwrap();
    assert_eq!(ok.issuer_identity.as_str(), id.as_str());
}

#[tokio::test]
async fn notify_rekey_updates_peer_trust() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-nk", [101u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-nk", [103u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    // Announce upcoming key *before* rotate so hello still verifies.
    let new_sk = SigningKey::from_bytes(&[105u8; 32]);
    let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move {
        let mut peer = accept(&listener, &root_b2).await.unwrap();
        let env = peer.recv_envelope().await.unwrap();
        let delta = parse_trust_delta(&env).unwrap();
        apply_trust_delta(&root_b2, &env.issuer_identity, &delta).unwrap();
        delta
    });

    let results = notify_peers_of_rekey(root_a, &new_pub, None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].ok, "{:?}", results[0].error);

    let delta = accept_task.await.unwrap();
    assert_eq!(delta.op, TrustDeltaOp::Rekey);
    assert_eq!(delta.subject_id, id_a.as_str());
    assert_eq!(delta.new_pubkey_hex.as_deref(), Some(new_pub.as_str()));

    let tb = TrustStore::load(root_b).unwrap();
    let entry = tb
        .entries
        .iter()
        .find(|e| e.identity_id == id_a.as_str())
        .unwrap();
    assert_eq!(entry.public_key_hex, new_pub);
    assert!(entry.previous_public_key_hex.is_none());

    // Now rotate; subsequent dials use the new key against updated trust.
    aira_object::rotate_node_signing_secret(root_a, new_sk, false, None).unwrap();
}

#[tokio::test]
async fn notify_rekey_with_grace_keeps_old_pubkey() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "alice-grace", [121u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "bob-grace", [123u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let new_sk = SigningKey::from_bytes(&[125u8; 32]);
    let new_pub = hex::encode(new_sk.verifying_key().to_bytes());
    let until = "2099-12-01T00:00:00Z";

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let accept_task = tokio::spawn(async move {
        let mut peer = accept(&listener, &root_b2).await.unwrap();
        let env = peer.recv_envelope().await.unwrap();
        let delta = parse_trust_delta(&env).unwrap();
        apply_trust_delta(&root_b2, &env.issuer_identity, &delta).unwrap();
        delta
    });

    let results = notify_peers_of_rekey(root_a, &new_pub, Some(until))
        .await
        .unwrap();
    assert!(results[0].ok, "{:?}", results[0].error);
    let delta = accept_task.await.unwrap();
    assert_eq!(delta.grace_until.as_deref(), Some(until));

    let tb = TrustStore::load(root_b).unwrap();
    let entry = tb
        .entries
        .iter()
        .find(|e| e.identity_id == id_a.as_str())
        .unwrap();
    assert_eq!(entry.public_key_hex, new_pub);
    assert_eq!(
        entry.previous_public_key_hex.as_deref(),
        Some(pub_a.as_str())
    );
    let ring = tb.to_keyring_at("2026-07-28T18:00:00Z").unwrap();
    assert_eq!(ring.verifying_keys(id_a.as_str()).len(), 2);
    let old_sk = SigningKey::from_bytes(&[121u8; 32]);
    let msg = b"still-old";
    let old_sig = sign_with_key(id_a.clone(), &old_sk, msg);
    let new_sig = sign_with_key(id_a.clone(), &new_sk, msg);
    ring.verify(&old_sig, msg).unwrap();
    ring.verify(&new_sig, msg).unwrap();
}

/// Craft a signed trust-delta envelope **without** the send-side subject==local gate
/// (simulates legacy / hostile third-party CRL for Analyze-53 gossip filter).
fn craft_trust_delta_envelope_unchecked(
    root: &std::path::Path,
    delta: &TrustDelta,
) -> ProtocolEnvelope {
    use rand::rngs::OsRng;
    use rand::RngCore;
    delta.validate_shape().unwrap();
    let (local_id, ring) = Keyring::load_node_identity(root).unwrap();
    let json = String::from_utf8(delta.canonical_bytes().unwrap()).unwrap();
    let hash = ContentHash::sha256_bytes(json.as_bytes());
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id = AiraRef::parse(format!(
        "aira:message:trust-delta-hostile-{}",
        hex::encode(nonce)
    ))
    .unwrap();
    let created = aira_object::utc_now_rfc3339().unwrap();
    ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: TRUST_DELTA_MESSAGE_TYPE.into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id.clone(),
        target_scope: ScopeDescriptor::local("peer-trust-delta"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(json),
        created_at: Timestamp::parse(created).unwrap(),
        expires_at: None,
        signature: ProtocolEnvelope::placeholder_signature(&local_id),
    }
    .attach_canonical_signature_with_keyring(&ring, &local_id)
    .unwrap()
}

#[tokio::test]
async fn gossip_skips_non_self_sovereign_trust_delta() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let (id_a, _) = write_node_identity(root, "g-hostile", [141u8; 32]);
    let victim = "aira:identity:gossip-victim-53";
    // Address-book peer that must NOT be dialed for doomed deltas.
    let mut book = AddressBook::default();
    book.upsert("aira:identity:would-dial", "127.0.0.1:49157").unwrap();
    book.save(root).unwrap();

    let delta = TrustDelta::revoke(victim, Some("hostile-crl".into()));
    let env = craft_trust_delta_envelope_unchecked(root, &delta);
    assert_ne!(delta.subject_id, env.issuer_identity.as_str());

    let results = gossip_forward_trust_delta(root, &env, "aira:identity:upstream")
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].skipped);
    assert_eq!(
        results[0].error.as_deref(),
        Some("non-self-sovereign trust-delta")
    );
    // Seen marked so a retry is duplicate-skip (no dial).
    let again = gossip_forward_trust_delta(root, &env, "aira:identity:upstream")
        .await
        .unwrap();
    assert!(again[0].skipped);
    assert!(again[0].error.is_none());
    let _ = id_a;
}

#[tokio::test]
async fn gossip_trust_delta_a_to_b_to_c() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let dir_c = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    let root_c = dir_c.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    init_node(root_c).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "g-alice", [111u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "g-bob", [113u8; 32]);
    let (id_c, pub_c) = write_node_identity(root_c, "g-carol", [115u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);
    mutual_trust(root_b, id_b.as_str(), &pub_b, root_c, id_c.as_str(), &pub_c);
    // C must trust A (originator) to apply forwarded envelope.
    let mut tc = TrustStore::load(root_c).unwrap();
    tc.upsert(id_a.as_str(), &pub_a).unwrap();
    tc.save(root_c).unwrap();
    let mut ta = TrustStore::load(root_a).unwrap();
    ta.upsert(id_c.as_str(), &pub_c).unwrap();
    ta.save(root_a).unwrap();

    // C listens for gossip from B.
    let listener_c = listen_available_loopback().await.unwrap().0;
    let addr_c = listener_c.local_addr().unwrap();
    let mut book_b = AddressBook::default();
    book_b.upsert(id_c.as_str(), addr_c.to_string()).unwrap();
    book_b.save(root_b).unwrap();

    let root_c2 = root_c.to_path_buf();
    let c_task = tokio::spawn(async move {
        let mut peer = accept(&listener_c, &root_c2).await.unwrap();
        let env = peer
            .recv_envelope_allow_relayed_trust_delta()
            .await
            .unwrap();
        let delta = parse_trust_delta(&env).unwrap();
        apply_trust_delta(&root_c2, &env.issuer_identity, &delta).unwrap();
        let _ = PeerDiscoveryStore::record_and_save(
            &root_c2,
            env.issuer_identity.as_str(),
            None,
            Some(peer.peer_id.as_str().to_string()),
            DiscoverySource::Gossip,
        );
        (env.message_id.as_str().to_string(), delta)
    });

    // B listens for A, applies, then gossips to C.
    let listener_b = listen_available_loopback().await.unwrap().0;
    let addr_b = listener_b.local_addr().unwrap();
    let mut book_a = AddressBook::default();
    book_a.upsert(id_b.as_str(), addr_b.to_string()).unwrap();
    book_a.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let id_a_s = id_a.as_str().to_string();
    let b_task = tokio::spawn(async move {
        let mut peer = accept(&listener_b, &root_b2).await.unwrap();
        let env = peer.recv_envelope().await.unwrap();
        let delta = parse_trust_delta(&env).unwrap();
        apply_trust_delta(&root_b2, &env.issuer_identity, &delta).unwrap();
        let results = gossip_forward_trust_delta(&root_b2, &env, &id_a_s)
            .await
            .unwrap();
        (delta, results)
    });

    // Alice self-announces revoke; gossip fans out original signed envelope.
    let delta = TrustDelta::revoke(id_a.as_str(), Some("gossip-demo".into()));
    let env = make_trust_delta_envelope(root_a, &delta).unwrap();
    let mut client = dial(root_a, id_b.as_str()).await.unwrap();
    client.send_envelope(&env).await.unwrap();

    let (applied, results) = b_task.await.unwrap();
    assert_eq!(applied.op, TrustDeltaOp::Revoke);
    assert!(
        results.iter().any(|r| r.peer_id == id_c.as_str() && r.ok),
        "{results:?}"
    );

    let (msg_id, c_delta) = c_task.await.unwrap();
    assert_eq!(c_delta.subject_id, id_a.as_str());
    assert_eq!(msg_id, env.message_id.as_str());
    assert!(TrustStore::load(root_c).unwrap().is_revoked(id_a.as_str()));
    assert!(TrustStore::load(root_b).unwrap().is_revoked(id_a.as_str()));

    let disc = PeerDiscoveryStore::load(root_b).unwrap();
    assert!(disc.peers.iter().any(|e| e.identity_id == id_c.as_str()));

    // Second gossip of same message_id is skipped.
    let again = gossip_forward_trust_delta(root_b, &env, id_a.as_str())
        .await
        .unwrap();
    assert!(again.iter().any(|r| r.skipped));
}

#[tokio::test]
async fn relay_hub_delivers_trust_delta_a_to_c_via_r() {
    let dir_a = tempdir().unwrap();
    let dir_r = tempdir().unwrap();
    let dir_c = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_r = dir_r.path();
    let root_c = dir_c.path();
    init_node(root_a).unwrap();
    init_node(root_r).unwrap();
    init_node(root_c).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "r-alice", [121u8; 32]);
    let (id_r, pub_r) = write_node_identity(root_r, "r-relay", [123u8; 32]);
    let (id_c, pub_c) = write_node_identity(root_c, "r-carol", [125u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_r, id_r.as_str(), &pub_r);
    mutual_trust(root_c, id_c.as_str(), &pub_c, root_r, id_r.as_str(), &pub_r);
    // C must trust originator A to apply courier-delivered delta.
    let mut tc = TrustStore::load(root_c).unwrap();
    tc.upsert(id_a.as_str(), &pub_a).unwrap();
    tc.save(root_c).unwrap();

    let hub = RelayHub::new();
    let listener = listen_available_loopback().await.unwrap().0;
    let addr_r = listener.local_addr().unwrap();
    let hub_accept = hub.clone();
    let root_r2 = root_r.to_path_buf();
    let accept_task = tokio::spawn(async move {
        // Accept C then A (order not guaranteed — accept two).
        for _ in 0..2 {
            let peer = accept(&listener, &root_r2).await.unwrap();
            let hub_c = hub_accept.clone();
            let root_peer = root_r2.clone();
            tokio::spawn(async move {
                let _ = serve_relay_peer(hub_c, peer, &root_peer, None).await;
            });
        }
    });

    let mut book_c = AddressBook::default();
    book_c.upsert(id_r.as_str(), addr_r.to_string()).unwrap();
    book_c.save(root_c).unwrap();

    let mut book_a = AddressBook::default();
    book_a.upsert(id_r.as_str(), addr_r.to_string()).unwrap();
    // C is not dialable from A — courier via R only (dummy addr).
    book_a.upsert_via(
        id_c.as_str(),
        "127.0.0.1:65521",
        Some(id_r.as_str().to_string()),
    ).unwrap();
    book_a.save(root_a).unwrap();

    let root_c2 = root_c.to_path_buf();
    let id_r_s = id_r.as_str().to_string();
    let hold = tokio::spawn(async move {
        let mut peer = dial(&root_c2, &id_r_s).await.unwrap();
        let env = peer.recv_envelope_allow_relayed().await.unwrap();
        let delta = parse_trust_delta(&env).unwrap();
        apply_trust_delta(&root_c2, &env.issuer_identity, &delta).unwrap();
        delta
    });

    // Wait until C is registered on the hub.
    for _ in 0..50 {
        if hub.registered().iter().any(|id| id == id_c.as_str()) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(
        hub.registered().iter().any(|id| id == id_c.as_str()),
        "carol not registered: {:?}",
        hub.registered()
    );

    // Alice self-announces revoke via relay courier (Analyze-52).
    let delta = TrustDelta::revoke(id_a.as_str(), Some("via-relay".into()));
    let env = make_trust_delta_envelope(root_a, &delta).unwrap();
    send_envelope_to_peer(root_a, id_c.as_str(), &env)
        .await
        .unwrap();

    let applied = hold.await.unwrap();
    assert_eq!(applied.subject_id, id_a.as_str());
    assert!(TrustStore::load(root_c).unwrap().is_revoked(id_a.as_str()));
    let _ = accept_task.await;
}

#[tokio::test]
async fn dht_announce_a_to_b_then_find() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "dht-alice", [131u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "dht-bob", [133u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let listener = listen_available_loopback().await.unwrap().0;
    let addr_b = listener.local_addr().unwrap();
    let mut book_a = AddressBook::default();
    book_a.upsert(id_b.as_str(), addr_b.to_string()).unwrap();
    book_a.save(root_a).unwrap();

    let root_b2 = root_b.to_path_buf();
    let b_task = tokio::spawn(async move {
        let mut peer = accept(&listener, &root_b2).await.unwrap();
        let env = peer.recv_envelope().await.unwrap();
        let announce = parse_dht_announce(&env).unwrap();
        apply_dht_announce(&root_b2, &env.issuer_identity, &announce).unwrap();
        announce
    });

    let announce_addr = "127.0.0.1:49157";
    let results = dht_announce_to_peers(root_a, announce_addr).await.unwrap();
    assert!(
        results.iter().any(|(id, ok, _)| id == id_b.as_str() && *ok),
        "{results:?}"
    );

    let got = b_task.await.unwrap();
    assert_eq!(got.identity_id, id_a.as_str());
    assert_eq!(got.addr, announce_addr);

    let store_b = PeerDhtStore::load(root_b).unwrap();
    let found = store_b.get(id_a.as_str()).unwrap();
    assert_eq!(found.addr, announce_addr);
    let closest = store_b.closest(id_a.as_str(), 1);
    assert_eq!(closest[0].identity_id, id_a.as_str());

    let store_a = PeerDhtStore::load(root_a).unwrap();
    assert_eq!(store_a.get(id_a.as_str()).unwrap().addr, announce_addr);
}

#[tokio::test]
async fn dht_announce_apply_book_then_dial() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "dht-book-alice", [141u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "dht-book-bob", [143u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    // B listens for announce; A has B in book for fan-out.
    let listener = listen_available_loopback().await.unwrap().0;
    let addr_b = listener.local_addr().unwrap();
    let mut book_a = AddressBook::default();
    book_a.upsert(id_b.as_str(), addr_b.to_string()).unwrap();
    book_a.save(root_a).unwrap();

    // B already has a via entry for A that must survive promote.
    let mut book_b = AddressBook::default();
    book_b.upsert_via(
        id_a.as_str(),
        "127.0.0.1:65521",
        Some("aira:identity:relay-keep".into()),
    ).unwrap();
    book_b.save(root_b).unwrap();

    let root_b2 = root_b.to_path_buf();
    let b_task = tokio::spawn(async move {
        let mut peer = accept(&listener, &root_b2).await.unwrap();
        let env = peer.recv_envelope().await.unwrap();
        let announce = parse_dht_announce(&env).unwrap();
        apply_dht_announce_maybe_book(&root_b2, &env.issuer_identity, &announce, true).unwrap();
        announce
    });

    // Announce A's real listen addr so B can dial A after promote.
    let listener_a = listen_available_loopback().await.unwrap().0;
    let announce_addr = listener_a.local_addr().unwrap().to_string();
    let results = dht_announce_to_peers(root_a, &announce_addr).await.unwrap();
    assert!(
        results.iter().any(|(id, ok, _)| id == id_b.as_str() && *ok),
        "{results:?}"
    );
    let got = b_task.await.unwrap();
    assert_eq!(got.identity_id, id_a.as_str());
    assert_eq!(got.addr, announce_addr);

    let book_b = AddressBook::load(root_b).unwrap();
    let ep = book_b
        .peers
        .iter()
        .find(|p| p.identity_id == id_a.as_str())
        .unwrap();
    assert_eq!(ep.addr, announce_addr);
    assert_eq!(ep.via.as_deref(), Some("aira:identity:relay-keep"));

    let accept_a = {
        let root_a2 = root_a.to_path_buf();
        tokio::spawn(async move {
            let _peer = accept(&listener_a, &root_a2).await.unwrap();
        })
    };
    let session = dial(root_b, id_a.as_str()).await.unwrap();
    assert_eq!(session.peer_id.as_str(), id_a.as_str());
    let _ = accept_a.await;
}

#[test]
fn apply_book_exact_from_find_skips_closest_only() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    let mut store = PeerDhtStore::default();
    store
        .upsert("aira:identity:known", "127.0.0.1:49157", Some("local".into()))
        .unwrap();
    store.save(root).unwrap();
    // Production find --apply-book path: no exact → Ok(None), book untouched.
    let promoted = apply_book_exact_from_dht_find(root, "aira:identity:missing").unwrap();
    assert!(promoted.is_none());
    assert!(AddressBook::load(root).unwrap().peers.is_empty());
    // Exact hit does promote.
    let hit = apply_book_exact_from_dht_find(root, "aira:identity:known")
        .unwrap()
        .expect("exact");
    assert_eq!(hit.0, "aira:identity:known");
    assert_eq!(hit.1, "127.0.0.1:49157");
    assert_eq!(
        AddressBook::load(root)
            .unwrap()
            .resolve("aira:identity:known")
            .unwrap()
            .to_string(),
        "127.0.0.1:49157"
    );
}

#[test]
fn promote_without_prior_book_inserts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    init_node(root).unwrap();
    promote_dht_to_address_book(root, "aira:identity:x", "127.0.0.1:49157").unwrap();
    let book = AddressBook::load(root).unwrap();
    assert_eq!(book.peers.len(), 1);
    assert_eq!(book.peers[0].addr, "127.0.0.1:49157");
    assert!(book.peers[0].via.is_none());
}

/// Daemon-style accept: hung TCP (no hello) must not block a real dial (Analyze-59).
#[tokio::test]
async fn hung_tcp_does_not_block_accept_loop() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "a59-alice", [141u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "a59-bob", [143u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<AuthenticatedPeer, PeerError>>(4);
    let root_b2 = root_b.to_path_buf();
    let accept_loop = tokio::spawn(async move {
        loop {
            let stream = match accept_tcp(&listener).await {
                Ok(s) => s,
                Err(_) => break,
            };
            let root = root_b2.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(complete_accept(stream, &root).await).await;
            });
        }
    });

    // Hang without hello/Noise — would previously block composed `accept`.
    let _hung = tokio::net::TcpStream::connect(addr).await.unwrap();

    let started = std::time::Instant::now();
    let client = dial(root_a, id_b.as_str()).await.unwrap();
    assert!(
        started.elapsed() < std::time::Duration::from_secs(3),
        "dial blocked by hung handshake: {:?}",
        started.elapsed()
    );
    assert_eq!(client.peer_id, id_b);

    let mut got_ok = false;
    for _ in 0..20 {
        match tokio::time::timeout(std::time::Duration::from_millis(200), rx.recv()).await {
            Ok(Some(Ok(peer))) => {
                assert_eq!(peer.peer_id, id_a);
                got_ok = true;
                break;
            }
            Ok(Some(Err(_))) => continue, // hung handshake may time out later
            _ => continue,
        }
    }
    assert!(got_ok, "authenticated peer from real dial missing");
    accept_loop.abort();
}

/// Corrupt inbound bytes fail closed on that task; listener still accepts (Analyze-59).
#[tokio::test]
async fn broken_handshake_does_not_kill_listener() {
    use tokio::io::AsyncWriteExt;

    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "a59b-alice", [145u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "a59b-bob", [147u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<AuthenticatedPeer, PeerError>>(4);
    let root_b2 = root_b.to_path_buf();
    let accept_loop = tokio::spawn(async move {
        for _ in 0..2 {
            let stream = accept_tcp(&listener).await.unwrap();
            let root = root_b2.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(complete_accept(stream, &root).await).await;
            });
        }
    });

    let mut bad = tokio::net::TcpStream::connect(addr).await.unwrap();
    bad.write_all(b"not-a-peer-hello").await.unwrap();
    let _ = bad.shutdown().await;

    let mut saw_err = false;
    let mut saw_ok = false;
    // Real dial after garbage.
    let client = dial(root_a, id_b.as_str()).await.unwrap();
    assert_eq!(client.peer_id, id_b);
    drop(client);

    for _ in 0..20 {
        match tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv()).await {
            Ok(Some(Err(_))) => saw_err = true,
            Ok(Some(Ok(peer))) => {
                assert_eq!(peer.peer_id, id_a);
                saw_ok = true;
            }
            _ => {}
        }
        if saw_err && saw_ok {
            break;
        }
    }
    assert!(saw_err, "corrupt handshake should fail closed");
    assert!(saw_ok, "listener must accept after corrupt handshake");
    let _ = accept_loop.await;
}

/// ≥2 parallel authenticated sessions both recv (Analyze-59 Done when).
#[tokio::test]
async fn two_parallel_sessions_recv() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let dir_c = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_b = dir_b.path();
    let root_c = dir_c.path();
    init_node(root_a).unwrap();
    init_node(root_b).unwrap();
    init_node(root_c).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "a59p-alice", [149u8; 32]);
    let (id_b, pub_b) = write_node_identity(root_b, "a59p-bob", [151u8; 32]);
    let (id_c, pub_c) = write_node_identity(root_c, "a59p-carol", [153u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_b, id_b.as_str(), &pub_b);
    mutual_trust(root_c, id_c.as_str(), &pub_c, root_b, id_b.as_str(), &pub_b);

    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book_a = AddressBook::default();
    book_a.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book_a.save(root_a).unwrap();
    let mut book_c = AddressBook::default();
    book_c.upsert(id_b.as_str(), addr.to_string()).unwrap();
    book_c.save(root_c).unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(4);
    let root_b2 = root_b.to_path_buf();
    let accept_loop = tokio::spawn(async move {
        for _ in 0..2 {
            let stream = accept_tcp(&listener).await.unwrap();
            let root = root_b2.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut peer = complete_accept(stream, &root).await.unwrap();
                let env = peer.recv_envelope().await.unwrap();
                let _ = tx.send(env.issuer_identity.as_str().to_string()).await;
            });
        }
    });

    let root_a2 = root_a.to_path_buf();
    let id_b_a = id_b.as_str().to_string();
    let id_a_clone = id_a.clone();
    let send_a = tokio::spawn(async move {
        let mut peer = dial(&root_a2, &id_b_a).await.unwrap();
        let (_ida, ring) = Keyring::load_node_identity(&root_a2).unwrap();
        let env = make_envelope(&id_a_clone, &ring, "from-a");
        peer.send_envelope(&env).await.unwrap();
    });

    let root_c2 = root_c.to_path_buf();
    let id_b_c = id_b.as_str().to_string();
    let id_c_clone = id_c.clone();
    let send_c = tokio::spawn(async move {
        let mut peer = dial(&root_c2, &id_b_c).await.unwrap();
        let (_idc, ring) = Keyring::load_node_identity(&root_c2).unwrap();
        let env = make_envelope(&id_c_clone, &ring, "from-c");
        peer.send_envelope(&env).await.unwrap();
    });

    let mut issuers = Vec::new();
    for _ in 0..2 {
        let id = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv())
            .await
            .expect("recv timeout")
            .expect("channel closed");
        issuers.push(id);
    }
    issuers.sort();
    let mut expected = vec![id_a.as_str().to_string(), id_c.as_str().to_string()];
    expected.sort();
    assert_eq!(issuers, expected);
    send_a.await.unwrap();
    send_c.await.unwrap();
    let _ = accept_loop.await;
}

/// Relay-hub style: accept_tcp + spawn complete_accept still registers under hung peer.
#[tokio::test]
async fn relay_accept_tcp_spawn_survives_hung_peer() {
    let dir_a = tempdir().unwrap();
    let dir_r = tempdir().unwrap();
    let root_a = dir_a.path();
    let root_r = dir_r.path();
    init_node(root_a).unwrap();
    init_node(root_r).unwrap();
    let (id_a, pub_a) = write_node_identity(root_a, "a59r-alice", [155u8; 32]);
    let (id_r, pub_r) = write_node_identity(root_r, "a59r-relay", [157u8; 32]);
    mutual_trust(root_a, id_a.as_str(), &pub_a, root_r, id_r.as_str(), &pub_r);

    let hub = RelayHub::new();
    let (listener, addr) = listen_available_loopback().await.unwrap();
    let mut book = AddressBook::default();
    book.upsert(id_r.as_str(), addr.to_string()).unwrap();
    book.save(root_a).unwrap();

    let root_r2 = root_r.to_path_buf();
    let hub_c = hub.clone();
    let accept_loop = tokio::spawn(async move {
        for _ in 0..2 {
            let stream = accept_tcp(&listener).await.unwrap();
            let root = root_r2.clone();
            let hub = hub_c.clone();
            tokio::spawn(async move {
                if let Ok(peer) = complete_accept(stream, &root).await {
                    let _rx = hub.register(peer.peer_id.as_str());
                    // Hold the session briefly so the test can observe registration.
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    drop(peer);
                }
            });
        }
    });

    let _hung = tokio::net::TcpStream::connect(addr).await.unwrap();
    let started = std::time::Instant::now();
    let _client = dial(root_a, id_r.as_str()).await.unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(3));

    for _ in 0..50 {
        if hub.registered().iter().any(|id| id == id_a.as_str()) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(
        hub.registered().iter().any(|id| id == id_a.as_str()),
        "alice not registered after hung peer: {:?}",
        hub.registered()
    );
    accept_loop.abort();
}
