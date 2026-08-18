//! UDP discv5-style announce (Analyze-67 / QUEUE #32).
//!
//! AIRA-native signed datagram (not Ethereum discv5/ENR). One hop: verify +
//! upsert [`PeerDhtStore`] with `source=udp`. No FIND_NODE, no apply-book.

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::path::Path;
use std::time::Duration;

use aira_object::{utc_now_rfc3339, AiraRef, Keyring, Signature, TrustStore};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::dht::PeerDhtStore;
use crate::error::PeerError;

/// Domain tag for the signed canonical payload (distinct from DHT-lite schema).
pub const DISCV_ANNOUNCE_DOMAIN: &str = "aira:peer:discv:v1:announce";
/// JSON `schema` field on the datagram.
pub const DISCV_SCHEMA: &str = "aira:peer:discv:v1";

const MAX_DISCV_DATAGRAM: usize = 2048;

/// Signed UDP announce payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscvAnnounce {
    pub schema: String,
    pub identity_id: String,
    pub addr: String,
    pub nonce_hex: String,
    pub created_at: String,
    pub signature: Signature,
}

fn is_loopback_bind(bind: &str) -> bool {
    bind.starts_with("127.0.0.1:") || bind.starts_with("[::1]:") || bind.starts_with("localhost:")
}

fn announce_bytes(identity_id: &str, addr: &str, nonce_hex: &str, created_at: &str) -> Vec<u8> {
    format!("{DISCV_ANNOUNCE_DOMAIN}|{identity_id}|{addr}|{nonce_hex}|{created_at}").into_bytes()
}

fn random_nonce_hex() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

fn admit(trust: &TrustStore, identity_id: &str) -> Result<(), PeerError> {
    if trust.is_revoked(identity_id) {
        return Err(PeerError::Revoked(identity_id.into()));
    }
    if !trust.entries.iter().any(|e| e.identity_id == identity_id) {
        return Err(PeerError::Untrusted(identity_id.into()));
    }
    Ok(())
}

fn check_loopback_socket(sock: &UdpSocket, bind: &str) -> Result<(), PeerError> {
    let ip = sock.local_addr()?.ip();
    if !matches!(ip, IpAddr::V4(v4) if v4.is_loopback())
        && !matches!(ip, IpAddr::V6(v6) if v6.is_loopback())
    {
        return Err(PeerError::Discv(format!(
            "discv listen requires loopback bind, got {bind} resolved {ip} — pass --explicit"
        )));
    }
    Ok(())
}

/// Bind a **loopback** UDP listener for discv announce.
pub fn bind_udp(bind: &str) -> Result<UdpSocket, PeerError> {
    if !is_loopback_bind(bind) {
        return Err(PeerError::Discv(format!(
            "discv listen requires loopback bind, got {bind} — pass --explicit"
        )));
    }
    let sock = UdpSocket::bind(bind).map_err(|e| PeerError::Discv(e.to_string()))?;
    check_loopback_socket(&sock, bind)?;
    Ok(sock)
}

/// Bind UDP without loopback restriction (operator / advanced).
pub fn bind_udp_explicit(bind: &str) -> Result<UdpSocket, PeerError> {
    UdpSocket::bind(bind).map_err(|e| PeerError::Discv(e.to_string()))
}

/// Sign a discv announce for the local node identity.
pub fn sign_discv_announce(
    root: impl AsRef<Path>,
    advertised_addr: &str,
) -> Result<DiscvAnnounce, PeerError> {
    advertised_addr
        .parse::<SocketAddr>()
        .map_err(|e| PeerError::Discv(format!("bad advertised addr {advertised_addr}: {e}")))?;
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let nonce_hex = random_nonce_hex();
    let created_at = utc_now_rfc3339().map_err(|e| PeerError::Discv(e.to_string()))?;
    let bytes = announce_bytes(local_id.as_str(), advertised_addr, &nonce_hex, &created_at);
    let signature = ring
        .sign(&local_id, &bytes)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    Ok(DiscvAnnounce {
        schema: DISCV_SCHEMA.into(),
        identity_id: local_id.as_str().to_string(),
        addr: advertised_addr.to_string(),
        nonce_hex,
        created_at,
        signature,
    })
}

/// Verify + upsert into `peers/dht.json` with `source=udp`. Does not touch address book.
pub fn apply_discv_announce(
    root: impl AsRef<Path>,
    announce: &DiscvAnnounce,
) -> Result<(), PeerError> {
    if announce.schema != DISCV_SCHEMA {
        return Err(PeerError::Discv(format!(
            "discv schema mismatch: {}",
            announce.schema
        )));
    }
    announce
        .addr
        .parse::<SocketAddr>()
        .map_err(|e| PeerError::Discv(format!("bad announced addr {}: {e}", announce.addr)))?;
    if announce.identity_id != announce.signature.key_ref.as_str() {
        return Err(PeerError::IdentityMismatch);
    }
    let root = root.as_ref();
    let trust = TrustStore::load(root)?;
    admit(&trust, &announce.identity_id)?;
    let ring = trust.to_keyring()?;
    let bytes = announce_bytes(
        &announce.identity_id,
        &announce.addr,
        &announce.nonce_hex,
        &announce.created_at,
    );
    ring.verify(&announce.signature, &bytes)
        .map_err(|_| PeerError::InvalidSignature)?;
    let issuer =
        AiraRef::parse(&announce.identity_id).map_err(|e| PeerError::Discv(e.to_string()))?;
    if announce.identity_id != issuer.as_str() {
        return Err(PeerError::IdentityMismatch);
    }
    let mut store = PeerDhtStore::load(root)?;
    store.upsert(
        announce.identity_id.clone(),
        announce.addr.clone(),
        Some("udp".into()),
    )?;
    store.save(root)
}

/// Decode datagram JSON then [`apply_discv_announce`].
pub fn apply_discv_datagram(
    root: impl AsRef<Path>,
    buf: &[u8],
) -> Result<DiscvAnnounce, PeerError> {
    if buf.is_empty() || buf.len() > MAX_DISCV_DATAGRAM {
        return Err(PeerError::Discv(format!(
            "discv datagram size {} (max {MAX_DISCV_DATAGRAM})",
            buf.len()
        )));
    }
    let announce: DiscvAnnounce =
        serde_json::from_slice(buf).map_err(|e| PeerError::Discv(e.to_string()))?;
    apply_discv_announce(root, &announce)?;
    Ok(announce)
}

/// Sign and send one UDP announce to `to`.
pub fn send_discv_announce(
    root: impl AsRef<Path>,
    advertised_addr: &str,
    to: SocketAddr,
) -> Result<(), PeerError> {
    let announce = sign_discv_announce(root, advertised_addr)?;
    let json = serde_json::to_vec(&announce).map_err(|e| PeerError::Discv(e.to_string()))?;
    if json.len() > MAX_DISCV_DATAGRAM {
        return Err(PeerError::Discv(
            "signed announce exceeds datagram cap".into(),
        ));
    }
    let sock = UdpSocket::bind("127.0.0.1:0").map_err(|e| PeerError::Discv(e.to_string()))?;
    sock.send_to(&json, to)
        .map_err(|e| PeerError::Discv(e.to_string()))?;
    Ok(())
}

/// Receive one datagram and apply (blocking).
pub fn recv_one_and_store(
    sock: &UdpSocket,
    root: impl AsRef<Path>,
) -> Result<DiscvAnnounce, PeerError> {
    let mut buf = [0u8; MAX_DISCV_DATAGRAM];
    let (n, _src) = sock
        .recv_from(&mut buf)
        .map_err(|e| PeerError::Discv(e.to_string()))?;
    apply_discv_datagram(root, &buf[..n])
}

/// Helper for tests: set a short read timeout.
pub fn set_udp_timeout(sock: &UdpSocket, timeout: Duration) -> Result<(), PeerError> {
    sock.set_read_timeout(Some(timeout))
        .map_err(|e| PeerError::Discv(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    use aira_flow::NodePaths;
    use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, TrustStore};
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    use crate::address_book::AddressBook;

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

    #[test]
    fn udp_announce_roundtrip_stores_dht_not_book() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let (id_a, pub_a) = write_node_identity(&root_a, "alice-discv", [21u8; 32]);
        let (id_b, pub_b) = write_node_identity(&root_b, "bob-discv", [22u8; 32]);
        mutual_trust(
            &root_a,
            id_a.as_str(),
            &pub_a,
            &root_b,
            id_b.as_str(),
            &pub_b,
        );

        let sock = bind_udp("127.0.0.1:0").unwrap();
        set_udp_timeout(&sock, Duration::from_secs(2)).unwrap();
        let to = sock.local_addr().unwrap();
        let advertised = "127.0.0.1:7900";
        let root_a2 = root_a.clone();
        let h = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            send_discv_announce(&root_a2, advertised, to).unwrap();
        });
        let got = recv_one_and_store(&sock, &root_b).unwrap();
        h.join().unwrap();
        assert_eq!(got.identity_id, id_a.as_str());
        assert_eq!(got.addr, advertised);
        let store = PeerDhtStore::load(&root_b).unwrap();
        let rec = store.get(id_a.as_str()).expect("stored");
        assert_eq!(rec.addr, advertised);
        assert_eq!(rec.source.as_deref(), Some("udp"));
        let book = AddressBook::load(&root_b).unwrap();
        assert!(book.peers.is_empty(), "UDP must not apply-book");
    }

    #[test]
    fn untrusted_udp_announce_rejected() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let (_id_a, _) = write_node_identity(&root_a, "alice-untrust", [23u8; 32]);
        let _ = write_node_identity(&root_b, "bob-untrust", [24u8; 32]);
        let announce = sign_discv_announce(&root_a, "127.0.0.1:1").unwrap();
        let err = apply_discv_announce(&root_b, &announce).unwrap_err();
        assert!(matches!(err, PeerError::Untrusted(_)), "{err}");
        assert!(PeerDhtStore::load(&root_b).unwrap().records.is_empty());
    }

    #[test]
    fn revoked_udp_announce_rejected() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let (id_a, pub_a) = write_node_identity(&root_a, "alice-rev", [25u8; 32]);
        let (id_b, pub_b) = write_node_identity(&root_b, "bob-rev", [26u8; 32]);
        mutual_trust(
            &root_a,
            id_a.as_str(),
            &pub_a,
            &root_b,
            id_b.as_str(),
            &pub_b,
        );
        let mut tb = TrustStore::load(&root_b).unwrap();
        tb.revoke(id_a.as_str(), Some("test")).unwrap();
        tb.save(&root_b).unwrap();
        let announce = sign_discv_announce(&root_a, "127.0.0.1:2").unwrap();
        let err = apply_discv_announce(&root_b, &announce).unwrap_err();
        assert!(matches!(err, PeerError::Revoked(_)), "{err}");
    }

    #[test]
    fn bind_udp_rejects_non_loopback_without_explicit() {
        let err = bind_udp("0.0.0.0:0").unwrap_err();
        assert!(err.to_string().contains("loopback"), "{err}");
        let sock = bind_udp_explicit("127.0.0.1:0").unwrap();
        assert!(sock.local_addr().unwrap().ip().is_loopback());
    }

    #[test]
    fn identity_mismatch_on_key_ref() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let (id_a, pub_a) = write_node_identity(&root_a, "alice-mm", [27u8; 32]);
        let (id_b, pub_b) = write_node_identity(&root_b, "bob-mm", [28u8; 32]);
        mutual_trust(
            &root_a,
            id_a.as_str(),
            &pub_a,
            &root_b,
            id_b.as_str(),
            &pub_b,
        );
        let mut announce = sign_discv_announce(&root_a, "127.0.0.1:3").unwrap();
        announce.signature.key_ref = id_b.clone();
        let err = apply_discv_announce(&root_b, &announce).unwrap_err();
        assert!(matches!(err, PeerError::IdentityMismatch), "{err}");
    }
}
