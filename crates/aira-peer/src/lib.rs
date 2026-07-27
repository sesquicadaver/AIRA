//! AIRA authenticated peer links (Analyze-32 P0).
//!
//! Framed TCP + mutual Ed25519 hello + signed [`ProtocolEnvelope`] exchange.
//! Admission is local [`TrustStore`] only — no controlling center, no DHT.

mod address_book;
mod error;
mod frame;
mod handshake;
mod session;

pub use address_book::{AddressBook, PeerEndpoint};
pub use error::PeerError;
pub use frame::{read_frame, write_frame, MAX_FRAME_BYTES};
pub use handshake::{HelloMessage, HELLO_DOMAIN};
pub use session::{accept, dial, listen, listen_explicit, AuthenticatedPeer, DEFAULT_PEER_TIMEOUT};

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
        ensure_trust_defaults, sign_with_key, AiraRef, ContentHash, Keyring, Signature, Timestamp,
        TrustStore,
    };
    use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};
    use ed25519_dalek::SigningKey;
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

    fn mutual_trust(a_root: &std::path::Path, a_id: &str, a_pub: &str, b_root: &std::path::Path, b_id: &str, b_pub: &str) {
        let mut ta = TrustStore::load(a_root).unwrap();
        ta.upsert(b_id, b_pub).unwrap();
        ta.save(a_root).unwrap();
        let mut tb = TrustStore::load(b_root).unwrap();
        tb.upsert(a_id, a_pub).unwrap();
        tb.save(b_root).unwrap();
    }

    fn make_envelope(issuer: &AiraRef, ring: &Keyring, payload: &str) -> ProtocolEnvelope {
        let hash = ContentHash::sha256_bytes(payload.as_bytes());
        let sig = ring
            .sign(issuer, hash.as_str().as_bytes())
            .unwrap();
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
        mutual_trust(
            root_a,
            id_a.as_str(),
            &pub_a,
            root_b,
            id_b.as_str(),
            &pub_b,
        );

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
        mutual_trust(
            root_a,
            id_a.as_str(),
            &pub_a,
            root_b,
            id_b.as_str(),
            &pub_b,
        );
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
        mutual_trust(
            root_a,
            id_a.as_str(),
            &pub_a,
            root_b,
            id_b.as_str(),
            &pub_b,
        );

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

        // Direct frame with mismatched issuer after handshake.
        let (_ida2, ring_a2) = Keyring::load_node_identity(root_a).unwrap();
        let mut forged = make_envelope(&id_a, &ring_a2, "forged");
        forged.issuer_identity = id_b.clone();
        forged.signature = Signature {
            algorithm: "ed25519".into(),
            key_ref: id_b.clone(),
            signature_value: forged.signature.signature_value.clone(),
        };
        crate::frame::write_json(&mut client.stream, &forged)
            .await
            .unwrap();
        let err = server.recv_envelope().await.unwrap_err();
        assert!(matches!(
            err,
            PeerError::IdentityMismatch | PeerError::InvalidSignature
        ));
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
}
