//! UDP discv5-style announce + iterative FIND_NODE (Analyze-67/68).
//!
//! AIRA-native signed datagrams (not Ethereum discv5/ENR). Announce upserts
//! [`PeerDhtStore`] with `source=udp`. FIND/NODES iterate XOR-closest over UDP.
//! No apply-book.

use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::path::Path;
use std::time::Duration;

use aira_object::{utc_now_rfc3339, AiraRef, Keyring, Signature, TrustStore, LOCAL_TEST_KEY_REF};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::dht::{PeerDhtStore, DHT_DEFAULT_K};
use crate::error::PeerError;

/// Domain tag for the signed canonical payload (distinct from DHT-lite schema).
pub const DISCV_ANNOUNCE_DOMAIN: &str = "aira:peer:discv:v1:announce";
/// JSON `schema` field on announce datagrams.
pub const DISCV_SCHEMA: &str = "aira:peer:discv:v1";
/// Domain + schema for FIND requests.
pub const DISCV_FIND_DOMAIN: &str = "aira:peer:discv:v1:find";
pub const DISCV_FIND_SCHEMA: &str = "aira:peer:discv:v1:find";
/// Domain + schema for NODES replies.
pub const DISCV_NODES_DOMAIN: &str = "aira:peer:discv:v1:nodes";
pub const DISCV_NODES_SCHEMA: &str = "aira:peer:discv:v1:nodes";

const MAX_DISCV_DATAGRAM: usize = 2048;
/// Parallel FIND queries per hop.
pub const DISCV_FIND_ALPHA: usize = 3;
/// Iteration cap (not unbounded Kademlia).
pub const DISCV_FIND_MAX_HOPS: usize = 8;
/// Per-query recv timeout.
pub const DISCV_FIND_TIMEOUT: Duration = Duration::from_millis(400);

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

/// Signed FIND request (requester → listener).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscvFind {
    pub schema: String,
    pub identity_id: String,
    pub target_id: String,
    pub k: usize,
    pub nonce_hex: String,
    pub created_at: String,
    pub signature: Signature,
}

/// One hint in a NODES reply (not individually signed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscvNodeHint {
    pub identity_id: String,
    pub addr: String,
}

/// Signed NODES reply (listener → requester).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscvNodes {
    pub schema: String,
    pub identity_id: String,
    pub target_id: String,
    pub nonce_hex: String,
    pub created_at: String,
    pub nodes: Vec<DiscvNodeHint>,
    pub signature: Signature,
}

/// Outcome of one inbound datagram on `discv listen`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscvHandleResult {
    StoredAnnounce(DiscvAnnounce),
    AnsweredFind {
        requester: String,
        target_id: String,
        n: usize,
    },
}

/// Result of [`iterative_discv_find`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscvFindReport {
    pub hops: usize,
    pub queried: usize,
    pub stored: usize,
    pub exact: Option<(String, String)>,
}

fn is_loopback_bind(bind: &str) -> bool {
    bind.starts_with("127.0.0.1:") || bind.starts_with("[::1]:") || bind.starts_with("localhost:")
}

fn announce_bytes(identity_id: &str, addr: &str, nonce_hex: &str, created_at: &str) -> Vec<u8> {
    format!("{DISCV_ANNOUNCE_DOMAIN}|{identity_id}|{addr}|{nonce_hex}|{created_at}").into_bytes()
}

fn find_bytes(
    identity_id: &str,
    target_id: &str,
    k: usize,
    nonce_hex: &str,
    created_at: &str,
) -> Vec<u8> {
    format!("{DISCV_FIND_DOMAIN}|{identity_id}|{target_id}|{k}|{nonce_hex}|{created_at}")
        .into_bytes()
}

fn nodes_canonical(nodes: &[DiscvNodeHint]) -> String {
    let mut parts: Vec<String> = nodes
        .iter()
        .map(|n| format!("{}={}", n.identity_id, n.addr))
        .collect();
    parts.sort();
    parts.join(",")
}

fn nodes_bytes(
    identity_id: &str,
    target_id: &str,
    nonce_hex: &str,
    created_at: &str,
    nodes: &[DiscvNodeHint],
) -> Vec<u8> {
    let body = nodes_canonical(nodes);
    format!("{DISCV_NODES_DOMAIN}|{identity_id}|{target_id}|{nonce_hex}|{created_at}|{body}")
        .into_bytes()
}

fn peek_schema(buf: &[u8]) -> Result<String, PeerError> {
    #[derive(Deserialize)]
    struct Peek {
        schema: String,
    }
    let p: Peek = serde_json::from_slice(buf).map_err(|e| PeerError::Discv(e.to_string()))?;
    Ok(p.schema)
}

fn send_json(sock: &UdpSocket, to: SocketAddr, json: &[u8]) -> Result<(), PeerError> {
    if json.len() > MAX_DISCV_DATAGRAM {
        return Err(PeerError::Discv(format!(
            "discv datagram {} exceeds cap {MAX_DISCV_DATAGRAM}",
            json.len()
        )));
    }
    sock.send_to(json, to)
        .map_err(|e| PeerError::Discv(e.to_string()))?;
    Ok(())
}

fn require_signer(identity_id: &str, sig: &Signature) -> Result<(), PeerError> {
    if identity_id != sig.key_ref.as_str() {
        return Err(PeerError::IdentityMismatch);
    }
    AiraRef::parse(identity_id).map_err(|e| PeerError::Discv(e.to_string()))?;
    Ok(())
}

fn random_nonce_hex() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

fn admit(trust: &TrustStore, identity_id: &str) -> Result<(), PeerError> {
    if identity_id == LOCAL_TEST_KEY_REF {
        return Err(PeerError::Untrusted(identity_id.into()));
    }
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
    let buf = bounded(buf)?;
    let announce: DiscvAnnounce =
        serde_json::from_slice(buf).map_err(|e| PeerError::Discv(e.to_string()))?;
    apply_discv_announce(root, &announce)?;
    Ok(announce)
}

fn bounded(buf: &[u8]) -> Result<&[u8], PeerError> {
    if buf.is_empty() || buf.len() > MAX_DISCV_DATAGRAM {
        return Err(PeerError::Discv(format!(
            "discv datagram size {} (max {MAX_DISCV_DATAGRAM})",
            buf.len()
        )));
    }
    Ok(buf)
}

fn sign_discv_find(
    root: impl AsRef<Path>,
    target_id: &str,
    k: usize,
) -> Result<DiscvFind, PeerError> {
    AiraRef::parse(target_id).map_err(|e| PeerError::Discv(e.to_string()))?;
    let k = k.max(1);
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let nonce_hex = random_nonce_hex();
    let created_at = utc_now_rfc3339().map_err(|e| PeerError::Discv(e.to_string()))?;
    let bytes = find_bytes(local_id.as_str(), target_id, k, &nonce_hex, &created_at);
    let signature = ring
        .sign(&local_id, &bytes)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    Ok(DiscvFind {
        schema: DISCV_FIND_SCHEMA.into(),
        identity_id: local_id.as_str().to_string(),
        target_id: target_id.to_string(),
        k,
        nonce_hex,
        created_at,
        signature,
    })
}

fn verify_find(root: impl AsRef<Path>, find: &DiscvFind) -> Result<(), PeerError> {
    if find.schema != DISCV_FIND_SCHEMA {
        return Err(PeerError::Discv(format!(
            "discv find schema mismatch: {}",
            find.schema
        )));
    }
    require_signer(&find.identity_id, &find.signature)?;
    AiraRef::parse(&find.target_id).map_err(|e| PeerError::Discv(e.to_string()))?;
    let trust = TrustStore::load(root)?;
    admit(&trust, &find.identity_id)?;
    let ring = trust.to_keyring()?;
    let bytes = find_bytes(
        &find.identity_id,
        &find.target_id,
        find.k,
        &find.nonce_hex,
        &find.created_at,
    );
    ring.verify(&find.signature, &bytes)
        .map_err(|_| PeerError::InvalidSignature)?;
    Ok(())
}

fn sign_discv_nodes(
    root: impl AsRef<Path>,
    target_id: &str,
    nodes: Vec<DiscvNodeHint>,
) -> Result<DiscvNodes, PeerError> {
    let root = root.as_ref();
    let (local_id, ring) = Keyring::load_node_identity(root)?;
    let nonce_hex = random_nonce_hex();
    let created_at = utc_now_rfc3339().map_err(|e| PeerError::Discv(e.to_string()))?;
    let bytes = nodes_bytes(
        local_id.as_str(),
        target_id,
        &nonce_hex,
        &created_at,
        &nodes,
    );
    let signature = ring
        .sign(&local_id, &bytes)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    Ok(DiscvNodes {
        schema: DISCV_NODES_SCHEMA.into(),
        identity_id: local_id.as_str().to_string(),
        target_id: target_id.to_string(),
        nonce_hex,
        created_at,
        nodes,
        signature,
    })
}

fn verify_nodes(root: impl AsRef<Path>, nodes: &DiscvNodes) -> Result<(), PeerError> {
    if nodes.schema != DISCV_NODES_SCHEMA {
        return Err(PeerError::Discv(format!(
            "discv nodes schema mismatch: {}",
            nodes.schema
        )));
    }
    require_signer(&nodes.identity_id, &nodes.signature)?;
    let trust = TrustStore::load(root)?;
    admit(&trust, &nodes.identity_id)?;
    let ring = trust.to_keyring()?;
    let bytes = nodes_bytes(
        &nodes.identity_id,
        &nodes.target_id,
        &nodes.nonce_hex,
        &nodes.created_at,
        &nodes.nodes,
    );
    ring.verify(&nodes.signature, &bytes)
        .map_err(|_| PeerError::InvalidSignature)?;
    Ok(())
}

fn answer_find(
    sock: &UdpSocket,
    src: SocketAddr,
    root: impl AsRef<Path>,
    find: &DiscvFind,
) -> Result<DiscvHandleResult, PeerError> {
    let root = root.as_ref();
    verify_find(root, find)?;
    let k = find.k.clamp(1, DHT_DEFAULT_K);
    let store = PeerDhtStore::load(root)?;
    let hints: Vec<DiscvNodeHint> = store
        .closest(&find.target_id, k)
        .into_iter()
        .map(|r| DiscvNodeHint {
            identity_id: r.identity_id.clone(),
            addr: r.addr.clone(),
        })
        .collect();
    let n = hints.len();
    let reply = sign_discv_nodes(root, &find.target_id, hints)?;
    let json = serde_json::to_vec(&reply).map_err(|e| PeerError::Discv(e.to_string()))?;
    send_json(sock, src, &json)?;
    Ok(DiscvHandleResult::AnsweredFind {
        requester: find.identity_id.clone(),
        target_id: find.target_id.clone(),
        n,
    })
}

fn merge_nodes_into_store(root: impl AsRef<Path>, nodes: &DiscvNodes) -> Result<usize, PeerError> {
    verify_nodes(&root, nodes)?;
    let root = root.as_ref();
    let trust = TrustStore::load(root)?;
    let source = format!("udp:nodes:{}", nodes.identity_id);
    let mut store = PeerDhtStore::load(root)?;
    let mut stored = 0usize;
    for hint in &nodes.nodes {
        if admit(&trust, &hint.identity_id).is_err() {
            continue;
        }
        if hint.addr.parse::<SocketAddr>().is_err() {
            continue;
        }
        store.upsert(
            hint.identity_id.clone(),
            hint.addr.clone(),
            Some(source.clone()),
        )?;
        stored += 1;
    }
    if stored > 0 {
        store.save(root)?;
    }
    Ok(stored)
}

/// Dispatch one inbound datagram: announce store or FIND → NODES.
pub fn handle_discv_datagram(
    sock: &UdpSocket,
    root: impl AsRef<Path>,
    buf: &[u8],
    src: SocketAddr,
) -> Result<DiscvHandleResult, PeerError> {
    let buf = bounded(buf)?;
    let schema = peek_schema(buf)?;
    match schema.as_str() {
        DISCV_SCHEMA => {
            let announce: DiscvAnnounce =
                serde_json::from_slice(buf).map_err(|e| PeerError::Discv(e.to_string()))?;
            apply_discv_announce(root, &announce)?;
            Ok(DiscvHandleResult::StoredAnnounce(announce))
        }
        DISCV_FIND_SCHEMA => {
            let find: DiscvFind =
                serde_json::from_slice(buf).map_err(|e| PeerError::Discv(e.to_string()))?;
            answer_find(sock, src, root, &find)
        }
        DISCV_NODES_SCHEMA => Err(PeerError::Discv(
            "unexpected NODES on listen socket (client-only)".into(),
        )),
        other => Err(PeerError::Discv(format!("unknown discv schema {other}"))),
    }
}

/// Receive one datagram and dispatch (blocking).
pub fn recv_one_and_handle(
    sock: &UdpSocket,
    root: impl AsRef<Path>,
) -> Result<DiscvHandleResult, PeerError> {
    let mut buf = [0u8; MAX_DISCV_DATAGRAM];
    let (n, src) = sock
        .recv_from(&mut buf)
        .map_err(|e| PeerError::Discv(e.to_string()))?;
    handle_discv_datagram(sock, root, &buf[..n], src)
}

fn query_find_one(
    root: impl AsRef<Path>,
    target_id: &str,
    k: usize,
    to: SocketAddr,
    timeout: Duration,
) -> Result<DiscvNodes, PeerError> {
    let find = sign_discv_find(root.as_ref(), target_id, k)?;
    let json = serde_json::to_vec(&find).map_err(|e| PeerError::Discv(e.to_string()))?;
    let sock = UdpSocket::bind("127.0.0.1:0").map_err(|e| PeerError::Discv(e.to_string()))?;
    set_udp_timeout(&sock, timeout)?;
    send_json(&sock, to, &json)?;
    let mut buf = [0u8; MAX_DISCV_DATAGRAM];
    let (n, _src) = sock
        .recv_from(&mut buf)
        .map_err(|e| PeerError::Discv(e.to_string()))?;
    let buf = bounded(&buf[..n])?;
    let nodes: DiscvNodes =
        serde_json::from_slice(buf).map_err(|e| PeerError::Discv(e.to_string()))?;
    if nodes.target_id != target_id {
        return Err(PeerError::Discv("NODES target_id mismatch".into()));
    }
    Ok(nodes)
}

/// Iterative XOR FIND over UDP discv listeners (Analyze-68).
pub fn iterative_discv_find(
    root: impl AsRef<Path>,
    target_id: &str,
    extra_seeds: &[SocketAddr],
    k: usize,
) -> Result<DiscvFindReport, PeerError> {
    AiraRef::parse(target_id).map_err(|e| PeerError::Discv(e.to_string()))?;
    let k = k.max(1);
    let root = root.as_ref();
    let local_id = Keyring::load_node_identity(root)?.0;
    let mut queried: HashSet<SocketAddr> = HashSet::new();
    let mut stored_total = 0usize;
    let mut hops = 0usize;
    let mut queried_n = 0usize;

    for hop in 0..DISCV_FIND_MAX_HOPS {
        hops = hop + 1;
        let store = PeerDhtStore::load(root)?;
        if let Some(exact) = store.get(target_id) {
            if queried_n > 0 {
                return Ok(DiscvFindReport {
                    hops,
                    queried: queried_n,
                    stored: stored_total,
                    exact: Some((exact.identity_id.clone(), exact.addr.clone())),
                });
            }
        }
        let mut candidates: Vec<SocketAddr> = Vec::new();
        if hop == 0 {
            for s in extra_seeds {
                if queried.insert(*s) {
                    candidates.push(*s);
                }
            }
        }
        for rec in store.closest(target_id, k.max(DISCV_FIND_ALPHA)) {
            if rec.identity_id == local_id.as_str() {
                continue;
            }
            if let Ok(addr) = rec.addr.parse::<SocketAddr>() {
                if queried.insert(addr) {
                    candidates.push(addr);
                }
            }
            if candidates.len() >= DISCV_FIND_ALPHA {
                break;
            }
        }
        if candidates.is_empty() {
            break;
        }
        let mut progress = false;
        for to in candidates.into_iter().take(DISCV_FIND_ALPHA) {
            queried_n += 1;
            match query_find_one(root, target_id, k, to, DISCV_FIND_TIMEOUT) {
                Ok(nodes) => {
                    let n = merge_nodes_into_store(root, &nodes)?;
                    if n > 0 {
                        stored_total += n;
                        progress = true;
                    }
                }
                Err(_) => continue,
            }
        }
        if let Some(exact) = PeerDhtStore::load(root)?.get(target_id) {
            return Ok(DiscvFindReport {
                hops,
                queried: queried_n,
                stored: stored_total,
                exact: Some((exact.identity_id.clone(), exact.addr.clone())),
            });
        }
        if !progress && hop > 0 {
            break;
        }
    }
    let exact = PeerDhtStore::load(root)?
        .get(target_id)
        .map(|r| (r.identity_id.clone(), r.addr.clone()));
    Ok(DiscvFindReport {
        hops,
        queried: queried_n,
        stored: stored_total,
        exact,
    })
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
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

    fn listen_loop(sock: UdpSocket, root: std::path::PathBuf, stop: Arc<AtomicBool>) {
        set_udp_timeout(&sock, Duration::from_millis(150)).unwrap();
        while !stop.load(Ordering::SeqCst) {
            let _ = recv_one_and_handle(&sock, &root);
        }
    }

    #[test]
    fn iterative_find_a_via_b_stores_c() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let root_c = dir.path().join("c");
        let (id_a, pub_a) = write_node_identity(&root_a, "alice-fn", [31u8; 32]);
        let (id_b, pub_b) = write_node_identity(&root_b, "bob-fn", [32u8; 32]);
        let (id_c, pub_c) = write_node_identity(&root_c, "carol-fn", [33u8; 32]);
        mutual_trust(
            &root_a,
            id_a.as_str(),
            &pub_a,
            &root_b,
            id_b.as_str(),
            &pub_b,
        );
        mutual_trust(
            &root_b,
            id_b.as_str(),
            &pub_b,
            &root_c,
            id_c.as_str(),
            &pub_c,
        );
        mutual_trust(
            &root_a,
            id_a.as_str(),
            &pub_a,
            &root_c,
            id_c.as_str(),
            &pub_c,
        );

        let sock_b = bind_udp("127.0.0.1:0").unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        let sock_c = bind_udp("127.0.0.1:0").unwrap();
        let addr_c = sock_c.local_addr().unwrap();
        apply_discv_announce(
            &root_b,
            &sign_discv_announce(&root_c, &addr_c.to_string()).unwrap(),
        )
        .unwrap();
        apply_discv_announce(
            &root_a,
            &sign_discv_announce(&root_b, &addr_b.to_string()).unwrap(),
        )
        .unwrap();

        let stop = Arc::new(AtomicBool::new(false));
        let listen_b = sock_b.try_clone().unwrap();
        let root_b2 = root_b.clone();
        let stop_b = stop.clone();
        let hb = thread::spawn(move || listen_loop(listen_b, root_b2, stop_b));
        let listen_c = sock_c.try_clone().unwrap();
        let root_c2 = root_c.clone();
        let stop_c = stop.clone();
        let hc = thread::spawn(move || listen_loop(listen_c, root_c2, stop_c));
        thread::sleep(Duration::from_millis(30));

        let report = iterative_discv_find(&root_a, id_c.as_str(), &[], DHT_DEFAULT_K).unwrap();
        stop.store(true, Ordering::SeqCst);
        let _ = hb.join();
        let _ = hc.join();

        assert!(report.queried >= 1, "{report:?}");
        let exact = report.exact.expect("exact C");
        assert_eq!(exact.0, id_c.as_str());
        assert_eq!(exact.1, addr_c.to_string());
        let rec = PeerDhtStore::load(&root_a)
            .unwrap()
            .get(id_c.as_str())
            .unwrap()
            .clone();
        assert!(
            rec.source.as_ref().unwrap().starts_with("udp:nodes:"),
            "{:?}",
            rec.source
        );
        assert!(AddressBook::load(&root_a).unwrap().peers.is_empty());
    }

    #[test]
    fn find_untrusted_requester_times_out() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let (id_a, _) = write_node_identity(&root_a, "alice-fnu", [34u8; 32]);
        let _ = write_node_identity(&root_b, "bob-fnu", [35u8; 32]);
        let sock_b = bind_udp("127.0.0.1:0").unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let listen_b = sock_b.try_clone().unwrap();
        let root_b2 = root_b.clone();
        let stop_b = stop.clone();
        let hb = thread::spawn(move || listen_loop(listen_b, root_b2, stop_b));
        thread::sleep(Duration::from_millis(20));
        let err = query_find_one(
            &root_a,
            id_a.as_str(),
            8,
            addr_b,
            Duration::from_millis(250),
        )
        .unwrap_err();
        stop.store(true, Ordering::SeqCst);
        let _ = hb.join();
        assert!(matches!(err, PeerError::Discv(_)), "{err}");
        assert!(PeerDhtStore::load(&root_a).unwrap().records.is_empty());
    }

    #[test]
    fn nodes_skip_untrusted_hint() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let root_c = dir.path().join("c");
        let (id_a, pub_a) = write_node_identity(&root_a, "alice-skip", [36u8; 32]);
        let (id_b, pub_b) = write_node_identity(&root_b, "bob-skip", [37u8; 32]);
        let (id_c, pub_c) = write_node_identity(&root_c, "carol-skip", [38u8; 32]);
        mutual_trust(
            &root_a,
            id_a.as_str(),
            &pub_a,
            &root_b,
            id_b.as_str(),
            &pub_b,
        );
        mutual_trust(
            &root_b,
            id_b.as_str(),
            &pub_b,
            &root_c,
            id_c.as_str(),
            &pub_c,
        );
        let sock_c = bind_udp("127.0.0.1:0").unwrap();
        let addr_c = sock_c.local_addr().unwrap();
        apply_discv_announce(
            &root_b,
            &sign_discv_announce(&root_c, &addr_c.to_string()).unwrap(),
        )
        .unwrap();
        let sock_b = bind_udp("127.0.0.1:0").unwrap();
        let addr_b = sock_b.local_addr().unwrap();
        apply_discv_announce(
            &root_a,
            &sign_discv_announce(&root_b, &addr_b.to_string()).unwrap(),
        )
        .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let listen_b = sock_b.try_clone().unwrap();
        let root_b2 = root_b.clone();
        let stop_b = stop.clone();
        let hb = thread::spawn(move || listen_loop(listen_b, root_b2, stop_b));
        thread::sleep(Duration::from_millis(20));
        let report = iterative_discv_find(&root_a, id_c.as_str(), &[], 8).unwrap();
        stop.store(true, Ordering::SeqCst);
        let _ = hb.join();
        assert!(
            PeerDhtStore::load(&root_a)
                .unwrap()
                .get(id_c.as_str())
                .is_none(),
            "untrusted C must not be stored: {report:?}"
        );
    }

    #[test]
    fn handle_still_stores_announce() {
        let dir = tempdir().unwrap();
        let root_a = dir.path().join("a");
        let root_b = dir.path().join("b");
        let (id_a, pub_a) = write_node_identity(&root_a, "alice-h", [39u8; 32]);
        let (id_b, pub_b) = write_node_identity(&root_b, "bob-h", [40u8; 32]);
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
        let advertised = "127.0.0.1:7901";
        let root_a2 = root_a.clone();
        let h = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            send_discv_announce(&root_a2, advertised, to).unwrap();
        });
        let got = recv_one_and_handle(&sock, &root_b).unwrap();
        h.join().unwrap();
        match got {
            DiscvHandleResult::StoredAnnounce(a) => {
                assert_eq!(a.identity_id, id_a.as_str());
                assert_eq!(a.addr, advertised);
            }
            other => panic!("expected announce, got {other:?}"),
        }
    }
}
