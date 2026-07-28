//! Mutual Ed25519 hello v1 + Noise XX static binding (Analyze-35).

use aira_object::{AiraRef, Keyring, Signature, TrustStore};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::error::PeerError;
use crate::frame::{read_json, write_json};
use crate::noise::{load_or_create_noise_static, x25519_public};

/// Domain tag for peer hello signatures (v1 includes Noise static pub).
pub const HELLO_DOMAIN: &str = "aira:peer:hello:v1";

/// Wire message for hello exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloMessage {
    pub role: String,
    pub identity_id: String,
    pub nonce_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_nonce_hex: Option<String>,
    /// Hex-encoded X25519 static public key (32 bytes), bound by Ed25519 signature.
    pub x25519_pub_hex: String,
    pub signature: Signature,
}

fn hello_bytes(
    role: &str,
    identity_id: &str,
    nonce_hex: &str,
    peer_nonce_hex: Option<&str>,
    x25519_pub_hex: &str,
) -> Vec<u8> {
    match peer_nonce_hex {
        Some(pn) => {
            format!("{HELLO_DOMAIN}|{role}|{identity_id}|{nonce_hex}|{pn}|{x25519_pub_hex}")
                .into_bytes()
        }
        None => {
            format!("{HELLO_DOMAIN}|{role}|{identity_id}|{nonce_hex}|{x25519_pub_hex}").into_bytes()
        }
    }
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

fn parse_x25519_pub(hex_str: &str) -> Result<[u8; 32], PeerError> {
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| PeerError::Handshake(format!("x25519_pub_hex: {e}")))?;
    if bytes.len() != 32 {
        return Err(PeerError::Handshake(format!(
            "x25519_pub_hex must be 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn verify_hello(trust: &TrustStore, msg: &HelloMessage) -> Result<[u8; 32], PeerError> {
    admit(trust, &msg.identity_id)?;
    let x25519_pub = parse_x25519_pub(&msg.x25519_pub_hex)?;
    let ring = trust.to_keyring()?;
    let bytes = hello_bytes(
        &msg.role,
        &msg.identity_id,
        &msg.nonce_hex,
        msg.peer_nonce_hex.as_deref(),
        &msg.x25519_pub_hex,
    );
    if msg.signature.key_ref.as_str() != msg.identity_id {
        return Err(PeerError::IdentityMismatch);
    }
    ring.verify(&msg.signature, &bytes)
        .map_err(|_| PeerError::InvalidSignature)?;
    Ok(x25519_pub)
}

fn sign_hello(
    local_id: &AiraRef,
    ring: &Keyring,
    role: &str,
    nonce_hex: &str,
    peer_nonce_hex: Option<&str>,
    x25519_pub_hex: &str,
) -> Result<HelloMessage, PeerError> {
    let bytes = hello_bytes(
        role,
        local_id.as_str(),
        nonce_hex,
        peer_nonce_hex,
        x25519_pub_hex,
    );
    let signature = ring
        .sign(local_id, &bytes)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    Ok(HelloMessage {
        role: role.into(),
        identity_id: local_id.as_str().to_string(),
        nonce_hex: nonce_hex.into(),
        peer_nonce_hex: peer_nonce_hex.map(|s| s.to_string()),
        x25519_pub_hex: x25519_pub_hex.into(),
        signature,
    })
}

/// Result of Ed25519 hello: peer identity + peer's Noise static public key.
#[derive(Debug, Clone)]
pub struct HelloResult {
    pub peer_id: AiraRef,
    pub peer_x25519_pub: [u8; 32],
}

/// Run initiator (client) side of mutual hello. Returns peer id + Noise static pub.
pub async fn handshake_as_initiator(
    stream: &mut TcpStream,
    local_root: impl AsRef<std::path::Path>,
) -> Result<HelloResult, PeerError> {
    let local_root = local_root.as_ref();
    let (local_id, local_ring) = Keyring::load_node_identity(local_root)?;
    let trust = TrustStore::load(local_root)?;
    let static_priv = load_or_create_noise_static(local_root)?;
    let x25519_pub_hex = hex::encode(x25519_public(&static_priv));
    let nonce_c = random_nonce_hex();
    let hello = sign_hello(
        &local_id,
        &local_ring,
        "client",
        &nonce_c,
        None,
        &x25519_pub_hex,
    )?;
    write_json(stream, &hello).await?;

    let reply: HelloMessage = read_json(stream).await?;
    if reply.role != "server" {
        return Err(PeerError::Handshake("expected server hello".into()));
    }
    if reply.peer_nonce_hex.as_deref() != Some(nonce_c.as_str()) {
        return Err(PeerError::Handshake(
            "server did not echo client nonce".into(),
        ));
    }
    let peer_x25519_pub = verify_hello(&trust, &reply)?;

    let ack = sign_hello(
        &local_id,
        &local_ring,
        "client-ack",
        &nonce_c,
        Some(reply.nonce_hex.as_str()),
        &x25519_pub_hex,
    )?;
    write_json(stream, &ack).await?;
    let peer_id =
        AiraRef::parse(&reply.identity_id).map_err(|e| PeerError::Handshake(e.to_string()))?;
    Ok(HelloResult {
        peer_id,
        peer_x25519_pub,
    })
}

/// Run responder (server) side of mutual hello. Returns peer id + Noise static pub.
pub async fn handshake_as_responder(
    stream: &mut TcpStream,
    local_root: impl AsRef<std::path::Path>,
) -> Result<HelloResult, PeerError> {
    let local_root = local_root.as_ref();
    let (local_id, local_ring) = Keyring::load_node_identity(local_root)?;
    let trust = TrustStore::load(local_root)?;
    let static_priv = load_or_create_noise_static(local_root)?;
    let x25519_pub_hex = hex::encode(x25519_public(&static_priv));

    let hello: HelloMessage = read_json(stream).await?;
    if hello.role != "client" {
        return Err(PeerError::Handshake("expected client hello".into()));
    }
    let peer_x25519_pub = verify_hello(&trust, &hello)?;

    let nonce_s = random_nonce_hex();
    let reply = sign_hello(
        &local_id,
        &local_ring,
        "server",
        &nonce_s,
        Some(hello.nonce_hex.as_str()),
        &x25519_pub_hex,
    )?;
    write_json(stream, &reply).await?;

    let ack: HelloMessage = read_json(stream).await?;
    if ack.role != "client-ack" {
        return Err(PeerError::Handshake("expected client-ack".into()));
    }
    if ack.identity_id != hello.identity_id {
        return Err(PeerError::IdentityMismatch);
    }
    if ack.nonce_hex != hello.nonce_hex || ack.peer_nonce_hex.as_deref() != Some(nonce_s.as_str()) {
        return Err(PeerError::Handshake("client-ack nonce mismatch".into()));
    }
    let ack_pub = verify_hello(&trust, &ack)?;
    if ack_pub != peer_x25519_pub {
        return Err(PeerError::Handshake(
            "client-ack x25519_pub mismatch".into(),
        ));
    }
    let peer_id =
        AiraRef::parse(&hello.identity_id).map_err(|e| PeerError::Handshake(e.to_string()))?;
    Ok(HelloResult {
        peer_id,
        peer_x25519_pub,
    })
}
