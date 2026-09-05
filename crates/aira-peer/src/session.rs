//! Authenticated peer session: dial / accept + Noise XX + envelope exchange.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use aira_object::{AiraRef, TrustStore, LOCAL_TEST_KEY_REF};
use aira_protocol::ProtocolEnvelope;
use snow::TransportState;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::handshake::{handshake_as_initiator, handshake_as_responder};
use crate::noise::{
    load_or_create_noise_static, noise_xx_initiator, noise_xx_responder, read_encrypted,
    write_encrypted,
};
use crate::trust_delta::TRUST_DELTA_MESSAGE_TYPE;

/// Default I/O / handshake deadline for peer link operations.
pub const DEFAULT_PEER_TIMEOUT: Duration = Duration::from_secs(15);

async fn with_timeout<T, F>(fut: F) -> Result<T, PeerError>
where
    F: std::future::Future<Output = Result<T, PeerError>>,
{
    timeout(DEFAULT_PEER_TIMEOUT, fut)
        .await
        .map_err(|_| PeerError::Io("peer operation timed out".into()))?
}

/// Authenticated TCP session bound to a remote peer identity (Noise transport).
#[derive(Debug)]
pub struct AuthenticatedPeer {
    pub(crate) stream: TcpStream,
    pub(crate) transport: TransportState,
    /// Local node root (for trust-backed envelope verify).
    local_root: PathBuf,
    /// Authenticated remote identity.
    pub peer_id: AiraRef,
    /// Local node identity.
    pub local_id: AiraRef,
}

impl AuthenticatedPeer {
    /// Send one signed protocol envelope (Noise-encrypted frame).
    ///
    /// Issuer must be the local node (direct send). For gossip relay use
    /// [`Self::send_relayed_trust_delta`].
    pub async fn send_envelope(&mut self, envelope: &ProtocolEnvelope) -> Result<(), PeerError> {
        if envelope.issuer_identity != self.local_id {
            return Err(PeerError::IdentityMismatch);
        }
        if envelope.signature.key_ref != self.local_id {
            return Err(PeerError::IdentityMismatch);
        }
        self.write_envelope_bytes(envelope).await
    }

    /// Forward an original signed envelope whose issuer is not local (courier).
    ///
    /// Used by gossip trust-delta and relay-hub deliver. Signature/`key_ref`
    /// must still bind to `issuer_identity`.
    pub async fn send_relayed_envelope(
        &mut self,
        envelope: &ProtocolEnvelope,
    ) -> Result<(), PeerError> {
        if envelope.signature.key_ref != envelope.issuer_identity {
            return Err(PeerError::IdentityMismatch);
        }
        self.write_envelope_bytes(envelope).await
    }

    /// Forward an original `peer.trust.delta` envelope whose issuer is not local.
    ///
    /// Prefer [`Self::send_relayed_envelope`] for new call sites.
    pub async fn send_relayed_trust_delta(
        &mut self,
        envelope: &ProtocolEnvelope,
    ) -> Result<(), PeerError> {
        if envelope.message_type != TRUST_DELTA_MESSAGE_TYPE {
            return Err(PeerError::Protocol(format!(
                "relay expects {TRUST_DELTA_MESSAGE_TYPE}, got {}",
                envelope.message_type
            )));
        }
        self.send_relayed_envelope(envelope).await
    }

    async fn write_envelope_bytes(&mut self, envelope: &ProtocolEnvelope) -> Result<(), PeerError> {
        let bytes = serde_json::to_vec(envelope)?;
        with_timeout(write_encrypted(
            &mut self.stream,
            &mut self.transport,
            &bytes,
        ))
        .await
    }

    /// Receive one envelope; fail closed unless issuer/sig bind to authenticated peer.
    pub async fn recv_envelope(&mut self) -> Result<ProtocolEnvelope, PeerError> {
        self.recv_envelope_inner(false).await
    }

    /// Like [`Self::recv_envelope`], but also accepts courier-delivered envelopes
    /// signed by a trusted originator (issuer ≠ TCP peer). Used for gossip apply
    /// and relay-hold receivers.
    pub async fn recv_envelope_allow_relayed_trust_delta(
        &mut self,
    ) -> Result<ProtocolEnvelope, PeerError> {
        self.recv_envelope_inner(true).await
    }

    /// Alias: accept any relayed signed envelope from a trusted issuer.
    pub async fn recv_envelope_allow_relayed(&mut self) -> Result<ProtocolEnvelope, PeerError> {
        self.recv_envelope_inner(true).await
    }

    async fn recv_envelope_inner(
        &mut self,
        allow_relayed: bool,
    ) -> Result<ProtocolEnvelope, PeerError> {
        let bytes = with_timeout(read_encrypted(&mut self.stream, &mut self.transport)).await?;
        let env: ProtocolEnvelope = serde_json::from_slice(&bytes)?;
        let trust = TrustStore::load(&self.local_root)?;
        if trust.is_revoked(self.peer_id.as_str()) {
            return Err(PeerError::Revoked(self.peer_id.as_str().into()));
        }

        let direct = env.issuer_identity == self.peer_id && env.signature.key_ref == self.peer_id;
        if direct {
            let ring = trust.to_keyring()?;
            env.validate_signature_with_keyring(&ring)
                .map_err(|_| PeerError::InvalidSignature)?;
            crate::replay::admit_received_envelope(&self.local_root, &env)?;
            return Ok(env);
        }

        if allow_relayed && env.signature.key_ref == env.issuer_identity {
            let issuer = env.issuer_identity.as_str();
            if trust.is_revoked(issuer) {
                return Err(PeerError::Revoked(issuer.into()));
            }
            if !trust.entries.iter().any(|e| e.identity_id == issuer) {
                return Err(PeerError::Untrusted(issuer.into()));
            }
            let ring = trust.to_keyring()?;
            env.validate_signature_with_keyring(&ring)
                .map_err(|_| PeerError::InvalidSignature)?;
            crate::replay::admit_received_envelope(&self.local_root, &env)?;
            return Ok(env);
        }

        Err(PeerError::IdentityMismatch)
    }
}

pub(crate) fn ensure_noise_static_bind(
    expected: &[u8; 32],
    actual: &[u8; 32],
) -> Result<(), PeerError> {
    if expected != actual {
        return Err(PeerError::Handshake(
            "Noise remote static does not match hello x25519_pub".into(),
        ));
    }
    Ok(())
}

async fn finish_initiator(
    stream: TcpStream,
    local_root: PathBuf,
    hello: crate::handshake::HelloResult,
) -> Result<AuthenticatedPeer, PeerError> {
    let static_priv = load_or_create_noise_static(&local_root)?;
    let mut stream = stream;
    let (transport, remote_static) = noise_xx_initiator(&mut stream, &static_priv).await?;
    ensure_noise_static_bind(&hello.peer_x25519_pub, &remote_static)?;
    let (local_id, _) = aira_object::Keyring::load_node_identity(&local_root)?;
    Ok(AuthenticatedPeer {
        stream,
        transport,
        local_root,
        peer_id: hello.peer_id,
        local_id,
    })
}

async fn finish_responder(
    stream: TcpStream,
    local_root: PathBuf,
    hello: crate::handshake::HelloResult,
) -> Result<AuthenticatedPeer, PeerError> {
    let static_priv = load_or_create_noise_static(&local_root)?;
    let mut stream = stream;
    let (transport, remote_static) = noise_xx_responder(&mut stream, &static_priv).await?;
    ensure_noise_static_bind(&hello.peer_x25519_pub, &remote_static)?;
    let (local_id, _) = aira_object::Keyring::load_node_identity(&local_root)?;
    Ok(AuthenticatedPeer {
        stream,
        transport,
        local_root,
        peer_id: hello.peer_id,
        local_id,
    })
}

/// Dial a trusted peer from the local address book and complete hello + Noise XX.
pub async fn dial(
    local_root: impl AsRef<Path>,
    peer_identity_id: &str,
) -> Result<AuthenticatedPeer, PeerError> {
    let local_root = local_root.as_ref().to_path_buf();
    if peer_identity_id == LOCAL_TEST_KEY_REF {
        return Err(PeerError::Untrusted(peer_identity_id.into()));
    }
    let trust = TrustStore::load(&local_root)?;
    if trust.is_revoked(peer_identity_id) {
        return Err(PeerError::Revoked(peer_identity_id.into()));
    }
    if !trust
        .entries
        .iter()
        .any(|e| e.identity_id == peer_identity_id)
    {
        return Err(PeerError::Untrusted(peer_identity_id.into()));
    }
    let book = AddressBook::load(&local_root)?;
    let addr = book.resolve(peer_identity_id)?;
    crate::prime_port::validate_aira_port(addr.port())?;
    let mut stream =
        with_timeout(async { TcpStream::connect(addr).await.map_err(PeerError::from) }).await?;
    let hello = with_timeout(handshake_as_initiator(&mut stream, &local_root)).await?;
    if hello.peer_id.as_str() != peer_identity_id {
        return Err(PeerError::IdentityMismatch);
    }
    with_timeout(finish_initiator(stream, local_root, hello)).await
}

/// Accept the next TCP connection only (no hello / Noise).
///
/// Waiting for the next TCP connection is **not** bounded by [`DEFAULT_PEER_TIMEOUT`].
/// Daemon listen loops should call this, then spawn [`complete_accept`] so a slow
/// handshake cannot block further TCP accepts (Analyze-59).
pub async fn accept_tcp(listener: &TcpListener) -> Result<TcpStream, PeerError> {
    let (stream, _addr) = listener.accept().await.map_err(PeerError::from)?;
    Ok(stream)
}

/// Complete hello + Noise XX on an already-accepted TCP stream (responder).
///
/// Handshake / Noise steps are bounded by [`DEFAULT_PEER_TIMEOUT`].
pub async fn complete_accept(
    stream: TcpStream,
    local_root: impl AsRef<Path>,
) -> Result<AuthenticatedPeer, PeerError> {
    let local_root = local_root.as_ref().to_path_buf();
    let mut stream = stream;
    let hello = with_timeout(handshake_as_responder(&mut stream, &local_root)).await?;
    with_timeout(finish_responder(stream, local_root, hello)).await
}

/// Accept one inbound connection and complete hello + Noise XX.
///
/// Composed helper: [`accept_tcp`] then [`complete_accept`]. Prefer the split
/// APIs in daemon accept loops so handshake work can run off the accept path.
pub async fn accept(
    listener: &TcpListener,
    local_root: impl AsRef<Path>,
) -> Result<AuthenticatedPeer, PeerError> {
    let stream = accept_tcp(listener).await?;
    complete_accept(stream, local_root).await
}

fn is_loopback_bind(bind: &str) -> bool {
    bind.starts_with("127.0.0.1:") || bind.starts_with("[::1]:") || bind.starts_with("localhost:")
}

/// Bind a **loopback** listener for inbound peer links.
pub async fn listen(bind: &str) -> Result<TcpListener, PeerError> {
    crate::prime_port::validate_aira_bind(bind)?;
    if !is_loopback_bind(bind) {
        return Err(PeerError::Io(format!(
            "P0 listen requires loopback bind, got {bind} — use listen_explicit for overrides"
        )));
    }
    let listener = TcpListener::bind(bind).await?;
    let ip = listener.local_addr()?.ip();
    if !matches!(ip, IpAddr::V4(v4) if v4.is_loopback())
        && !matches!(ip, IpAddr::V6(v6) if v6.is_loopback())
    {
        return Err(PeerError::Io(format!(
            "P0 listen resolved non-loopback {ip}"
        )));
    }
    Ok(listener)
}

/// Bind without loopback restriction (operator / advanced).
pub async fn listen_explicit(bind: &str) -> Result<TcpListener, PeerError> {
    crate::prime_port::validate_aira_bind(bind)?;
    Ok(TcpListener::bind(bind).await?)
}

/// Bind the first free loopback `P_AIRA` TCP port (tests / operators without identity hash).
pub async fn listen_available_loopback() -> Result<(TcpListener, SocketAddr), PeerError> {
    for &port in crate::prime_port::p_aira_ports() {
        let bind = format!("127.0.0.1:{port}");
        match listen(&bind).await {
            Ok(listener) => {
                let addr = listener.local_addr()?;
                return Ok((listener, addr));
            }
            Err(PeerError::InvalidPort(msg)) => {
                return Err(PeerError::InvalidPort(msg));
            }
            Err(_) => continue,
        }
    }
    Err(PeerError::InvalidPort(
        "no free AIRA prime TCP port available on 127.0.0.1".into(),
    ))
}
