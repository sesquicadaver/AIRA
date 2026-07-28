//! AIRA authenticated peer links (Analyze-32…43).
//!
//! Framed TCP + mutual Ed25519 hello v1 + Noise XX + signed [`ProtocolEnvelope`].
//! Admission is local [`TrustStore`] only — no controlling center, no DHT.
//! Trust-delta (`peer.trust.delta`) can propagate CRL ops and same-id rekey over the encrypted link.
//! Analyze-43: optional gossip fanout + durable `peers/discovery.json`.

mod address_book;
mod discovery;
mod envelope;
mod error;
mod frame;
mod gossip;
mod handshake;
mod noise;
mod notify;
mod session;
mod trust_delta;

pub use address_book::{AddressBook, PeerEndpoint};
pub use discovery::{DiscoveryEntry, DiscoverySource, PeerDiscoveryStore};
pub use envelope::make_peer_ping;
pub use error::PeerError;
pub use frame::{read_frame, write_frame, MAX_FRAME_BYTES};
pub use gossip::{
    gossip_forward_trust_delta, gossip_mark_seen, GossipForwardResult, GossipSeenLog, GOSSIP_SEEN_CAP,
};
pub use handshake::{HelloMessage, HelloResult, HELLO_DOMAIN};
pub use noise::{load_or_create_noise_static, x25519_public, NOISE_PATTERN};
pub use notify::{
    notify_peer_of_rekey, notify_peers_of_rekey, upcoming_rekey_delta, NotifyPeerResult,
};
pub use session::{accept, dial, listen, listen_explicit, AuthenticatedPeer, DEFAULT_PEER_TIMEOUT};
pub use trust_delta::{
    apply_trust_delta, local_rekey_delta, make_trust_delta_envelope, parse_trust_delta, TrustDelta,
    TrustDeltaOp, TRUST_DELTA_MESSAGE_TYPE, TRUST_DELTA_SCHEMA,
};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use aira_flow::{init_node, NodePaths};
    use aira_object::{
        ensure_trust_defaults, sign_with_key, AiraRef, ContentHash, Keyring, Timestamp, TrustStore,
    };
    use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    fn write_node_identity(
        root: &std::path::Path,
        name: &str,
        seed: [u8; 32],
    ) -> (AiraRef, String) {
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
        let sig = ring.sign(issuer, hash.as_str().as_bytes()).unwrap();
        ProtocolEnvelope {
            protocol_id: ProtocolId::Identity,
            protocol_version: "0.1".into(),
            message_type: "peer.ping".into(),
            message_id: AiraRef::parse("aira:message:peer-ping-1").unwrap(),
            correlation_id: None,
            causal_refs: vec![],
            issuer_identity: issuer.clone(),
            target_scope: ScopeDescriptor::local("peer-p0"),
            policy_refs: vec![],
            payload_hash: hash,
            payload_ref: None,
            created_at: Timestamp::parse("2026-07-16T12:00:00Z").unwrap(),
            expires_at: None,
            signature: sig,
        }
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

        let listener = listen("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut book = AddressBook::default();
        book.upsert(id_b.as_str(), addr.to_string());
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

        let listener = listen("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut book = AddressBook::default();
        book.upsert(id_b.as_str(), addr.to_string());
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

        let listener = listen("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut book = AddressBook::default();
        book.upsert(id_b.as_str(), addr.to_string());
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
        book.upsert(id_b.as_str(), "127.0.0.1:9");
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

        let listener = listen("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut book = AddressBook::default();
        book.upsert(id_b.as_str(), addr.to_string());
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
        let err = listen("0.0.0.0:0").await.unwrap_err();
        assert!(matches!(err, PeerError::Io(_)));
    }

    #[test]
    fn make_peer_ping_signs_payload_hash() {
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
        ring.verify(&env.signature, env.payload_hash.as_str().as_bytes())
            .unwrap();
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

        // Both trust carol; alice announces revoke of carol to bob.
        let carol = "aira:identity:carol-td";
        let carol_sk = SigningKey::from_bytes(&[71u8; 32]);
        let carol_pub = hex::encode(carol_sk.verifying_key().to_bytes());
        let mut ta = TrustStore::load(root_a).unwrap();
        ta.upsert(carol, &carol_pub).unwrap();
        ta.save(root_a).unwrap();
        let mut tb = TrustStore::load(root_b).unwrap();
        tb.upsert(carol, &carol_pub).unwrap();
        tb.save(root_b).unwrap();

        let listener = listen("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut book = AddressBook::default();
        book.upsert(id_b.as_str(), addr.to_string());
        book.save(root_a).unwrap();

        let root_b2 = root_b.to_path_buf();
        let accept_task = tokio::spawn(async move { accept(&listener, root_b2).await });

        let delta = TrustDelta::revoke(carol, Some("compromised".into()));
        let env = make_trust_delta_envelope(root_a, &delta).unwrap();
        let mut client = dial(root_a, id_b.as_str()).await.unwrap();
        client.send_envelope(&env).await.unwrap();
        let mut server = accept_task.await.unwrap().unwrap();
        let got = server.recv_envelope().await.unwrap();
        let parsed = parse_trust_delta(&got).unwrap();
        assert_eq!(parsed.op, TrustDeltaOp::Revoke);
        apply_trust_delta(root_b, &got.issuer_identity, &parsed).unwrap();

        let tb = TrustStore::load(root_b).unwrap();
        assert!(tb.is_revoked(carol));
        assert!(!tb.entries.iter().any(|e| e.identity_id == carol));
    }

    #[test]
    fn trust_delta_refuses_local_test_and_local_node() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        init_node(root).unwrap();
        let (id, _) = write_node_identity(root, "solo-td", [51u8; 32]);
        let peer = "aira:identity:peer-td";
        let peer_sk = SigningKey::from_bytes(&[73u8; 32]);
        let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());
        let mut t = TrustStore::load(root).unwrap();
        t.upsert(peer, &peer_pub).unwrap();
        t.save(root).unwrap();
        let issuer = AiraRef::parse(peer).unwrap();

        let bad = TrustDelta::revoke(aira_object::LOCAL_TEST_KEY_REF, None);
        let err = apply_trust_delta(root, &issuer, &bad).unwrap_err();
        assert!(matches!(err, PeerError::Protocol(_)));

        let bad_local = TrustDelta::revoke(id.as_str(), None);
        let err = apply_trust_delta(root, &issuer, &bad_local).unwrap_err();
        assert!(matches!(err, PeerError::Protocol(_)));
    }

    #[test]
    fn trust_delta_rotate_shape_and_apply() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        init_node(root).unwrap();
        let (_id, _) = write_node_identity(root, "rot-host", [53u8; 32]);
        let peer = "aira:identity:peer-rot";
        let old = "aira:identity:old-rot";
        let new = "aira:identity:new-rot";
        let peer_sk = SigningKey::from_bytes(&[77u8; 32]);
        let old_sk = SigningKey::from_bytes(&[79u8; 32]);
        let new_sk = SigningKey::from_bytes(&[81u8; 32]);
        let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let new_pk = hex::encode(new_sk.verifying_key().to_bytes());
        let mut t = TrustStore::load(root).unwrap();
        t.upsert(peer, &peer_pub).unwrap();
        t.upsert(old, &old_pub).unwrap();
        t.save(root).unwrap();
        let issuer = AiraRef::parse(peer).unwrap();

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
                && e.issuer_id.as_deref() == Some(peer)
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
    fn trust_delta_rekey_requires_issuer_subject_match() {
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

        // Wrong subject (not issuer) → IdentityMismatch.
        let bad = TrustDelta::rekey(other, &new_pk, None, None);
        let err = apply_trust_delta(root, &issuer, &bad).unwrap_err();
        assert!(matches!(err, PeerError::IdentityMismatch));

        // Issuer rekeys self → upsert.
        let ok = TrustDelta::rekey(peer, &new_pk, Some("rotated".into()), None);
        apply_trust_delta(root, &issuer, &ok).unwrap();
        let t = TrustStore::load(root).unwrap();
        let entry = t.entries.iter().find(|e| e.identity_id == peer).unwrap();
        assert_eq!(entry.public_key_hex, new_pk);
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

        let listener = listen("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut book = AddressBook::default();
        book.upsert(id_b.as_str(), addr.to_string());
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

        // Now rotate; subsequent dials use the new key against updated trust.
        aira_object::rotate_node_signing_secret(root_a, new_sk, false, None).unwrap();
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

        let carol = "aira:identity:gossip-victim";
        let carol_sk = SigningKey::from_bytes(&[117u8; 32]);
        let carol_pub = hex::encode(carol_sk.verifying_key().to_bytes());
        for root in [root_a, root_b, root_c] {
            let mut t = TrustStore::load(root).unwrap();
            t.upsert(carol, &carol_pub).unwrap();
            t.save(root).unwrap();
        }

        // C listens for gossip from B.
        let listener_c = listen("127.0.0.1:0").await.unwrap();
        let addr_c = listener_c.local_addr().unwrap();
        let mut book_b = AddressBook::default();
        book_b.upsert(id_c.as_str(), addr_c.to_string());
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
        let listener_b = listen("127.0.0.1:0").await.unwrap();
        let addr_b = listener_b.local_addr().unwrap();
        let mut book_a = AddressBook::default();
        book_a.upsert(id_b.as_str(), addr_b.to_string());
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

        let delta = TrustDelta::revoke(carol, Some("gossip-demo".into()));
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
        assert_eq!(c_delta.subject_id, carol);
        assert_eq!(msg_id, env.message_id.as_str());
        assert!(TrustStore::load(root_c).unwrap().is_revoked(carol));
        assert!(TrustStore::load(root_b).unwrap().is_revoked(carol));

        let disc = PeerDiscoveryStore::load(root_b).unwrap();
        assert!(disc.peers.iter().any(|e| e.identity_id == id_c.as_str()));

        // Second gossip of same message_id is skipped.
        let again = gossip_forward_trust_delta(root_b, &env, id_a.as_str())
            .await
            .unwrap();
        assert!(again.iter().any(|r| r.skipped));
    }
}
