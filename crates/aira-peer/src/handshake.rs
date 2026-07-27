//! Mutual Ed25519 hello (Analyze-32 P0 — Noise XX deferred).

use aira_object::{AiraRef, Keyring, Signature, TrustStore};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use crate::error::PeerError;
use crate::frame::{read_json, write_json};

/// Domain tag for peer hello signatures.
pub const HELLO_DOMAIN: &str = "aira:peer:hello:v0";

/// Wire message for hello exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloMessage {
    pub role: String,
    pub identity_id: String,
    pub nonce_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_nonce_hex: Option<String>,
    pub signature: Signature,
}

fn hello_bytes(role: &str, identity_id: &str, nonce_hex: &str, peer_nonce_hex: Option<&str>) -> Vec<u8> {
    match peer_nonce_hex {
        Some(pn) => format!("{HELLO_DOMAIN}|{role}|{identity_id}|{nonce_hex}|{pn}").into_bytes(),
        None => format!("{HELLO_DOMAIN}|{role}|{identity_id}|{nonce_hex}").into_bytes(),
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

fn verify_hello(trust: &TrustStore, msg: &HelloMessage) -> Result<(), PeerError> {
    admit(trust, &msg.identity_id)?;
    let ring = trust.to_keyring()?;
    let bytes = hello_bytes(
        &msg.role,
        &msg.identity_id,
        &msg.nonce_hex,
        msg.peer_nonce_hex.as_deref(),
    );
    if msg.signature.key_ref.as_str() != msg.identity_id {
        return Err(PeerError::IdentityMismatch);
    }
    ring.verify(&msg.signature, &bytes)
        .map_err(|_| PeerError::InvalidSignature)?;
    Ok(())
}

fn sign_hello(
    local_id: &AiraRef,
    ring: &Keyring,
    role: &str,
    nonce_hex: &str,
    peer_nonce_hex: Option<&str>,
) -> Result<HelloMessage, PeerError> {
    let bytes = hello_bytes(role, local_id.as_str(), nonce_hex, peer_nonce_hex);
    let signature = ring
        .sign(local_id, &bytes)
        .map_err(|e| PeerError::Crypto(e.to_string()))?;
    Ok(HelloMessage {
        role: role.into(),
        identity_id: local_id.as_str().to_string(),
        nonce_hex: nonce_hex.into(),
        peer_nonce_hex: peer_nonce_hex.map(|s| s.to_string()),
        signature,
    })
}

/// Run initiator (client) side of mutual hello. Returns authenticated peer identity.
pub async fn handshake_as_initiator(
    stream: &mut TcpStream,
    local_root: impl AsRef<std::path::Path>,
) -> Result<AiraRef, PeerError> {
    let local_root = local_root.as_ref();
    let (local_id, local_ring) = Keyring::load_node_identity(local_root)?;
    let trust = TrustStore::load(local_root)?;
    let nonce_c = random_nonce_hex();
    let hello = sign_hello(&local_id, &local_ring, "client", &nonce_c, None)?;
    write_json(stream, &hello).await?;

    let reply: HelloMessage = read_json(stream).await?;
    if reply.role != "server" {
        return Err(PeerError::Handshake("expected server hello".into()));
    }
    if reply.peer_nonce_hex.as_deref() != Some(nonce_c.as_str()) {
        return Err(PeerError::Handshake("server did not echo client nonce".into()));
    }
    verify_hello(&trust, &reply)?;

    let ack = sign_hello(
        &local_id,
        &local_ring,
        "client-ack",
        &nonce_c,
        Some(reply.nonce_hex.as_str()),
    )?;
    write_json(stream, &ack).await?;
    AiraRef::parse(&reply.identity_id).map_err(|e| PeerError::Handshake(e.to_string()))
}

/// Run responder (server) side of mutual hello. Returns authenticated peer identity.
pub async fn handshake_as_responder(
    stream: &mut TcpStream,
    local_root: impl AsRef<std::path::Path>,
) -> Result<AiraRef, PeerError> {
    let local_root = local_root.as_ref();
    let (local_id, local_ring) = Keyring::load_node_identity(local_root)?;
    let trust = TrustStore::load(local_root)?;

    let hello: HelloMessage = read_json(stream).await?;
    if hello.role != "client" {
        return Err(PeerError::Handshake("expected client hello".into()));
    }
    verify_hello(&trust, &hello)?;

    let nonce_s = random_nonce_hex();
    let reply = sign_hello(
        &local_id,
        &local_ring,
        "server",
        &nonce_s,
        Some(hello.nonce_hex.as_str()),
    )?;
    write_json(stream, &reply).await?;

    let ack: HelloMessage = read_json(stream).await?;
    if ack.role != "client-ack" {
        return Err(PeerError::Handshake("expected client-ack".into()));
    }
    if ack.identity_id != hello.identity_id {
        return Err(PeerError::IdentityMismatch);
    }
    if ack.nonce_hex != hello.nonce_hex || ack.peer_nonce_hex.as_deref() != Some(nonce_s.as_str())
    {
        return Err(PeerError::Handshake("client-ack nonce mismatch".into()));
    }
    verify_hello(&trust, &ack)?;
    AiraRef::parse(&hello.identity_id).map_err(|e| PeerError::Handshake(e.to_string()))
}
