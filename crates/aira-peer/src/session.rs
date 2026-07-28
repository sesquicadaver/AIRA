//! Authenticated peer session: dial / accept + envelope exchange.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use aira_object::{AiraRef, TrustStore};
use aira_protocol::ProtocolEnvelope;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

use crate::address_book::AddressBook;
use crate::error::PeerError;
use crate::frame::{read_json, write_json};
use crate::handshake::{handshake_as_initiator, handshake_as_responder};

/// Default I/O / handshake deadline for peer link operations.
pub const DEFAULT_PEER_TIMEOUT: Duration = Duration::from_secs(10);

async fn with_timeout<T, F>(fut: F) -> Result<T, PeerError>
where
    F: std::future::Future<Output = Result<T, PeerError>>,
{
    timeout(DEFAULT_PEER_TIMEOUT, fut)
        .await
        .map_err(|_| PeerError::Io("peer operation timed out".into()))?
}

/// Authenticated TCP session bound to a remote peer identity.
#[derive(Debug)]
pub struct AuthenticatedPeer {
    pub(crate) stream: TcpStream,
    /// Local node root (for trust-backed envelope verify).
    local_root: PathBuf,
    /// Authenticated remote identity.
    pub peer_id: AiraRef,
    /// Local node identity.
    pub local_id: AiraRef,
}

impl AuthenticatedPeer {
    /// Send one signed protocol envelope.
    pub async fn send_envelope(&mut self, envelope: &ProtocolEnvelope) -> Result<(), PeerError> {
        if envelope.issuer_identity != self.local_id {
            return Err(PeerError::IdentityMismatch);
        }
        if envelope.signature.key_ref != self.local_id {
            return Err(PeerError::IdentityMismatch);
        }
        with_timeout(write_json(&mut self.stream, envelope)).await
    }

    /// Receive one envelope; fail closed unless issuer/sig bind to authenticated peer.
    ///
    /// Verifies signature strictly over `payload_hash` (no local-test domain fallback on wire).
    pub async fn recv_envelope(&mut self) -> Result<ProtocolEnvelope, PeerError> {
        let env: ProtocolEnvelope = with_timeout(read_json(&mut self.stream)).await?;
        if env.issuer_identity != self.peer_id {
            return Err(PeerError::IdentityMismatch);
        }
        if env.signature.key_ref != self.peer_id {
            return Err(PeerError::IdentityMismatch);
        }
        let trust = TrustStore::load(&self.local_root)?;
        if trust.is_revoked(self.peer_id.as_str()) {
            return Err(PeerError::Revoked(self.peer_id.as_str().into()));
        }
        let ring = trust.to_keyring()?;
        ring.verify(&env.signature, env.payload_hash.as_str().as_bytes())
            .map_err(|_| PeerError::InvalidSignature)?;
        Ok(env)
    }
}

/// Dial a trusted peer from the local address book and complete hello.
pub async fn dial(
    local_root: impl AsRef<Path>,
    peer_identity_id: &str,
) -> Result<AuthenticatedPeer, PeerError> {
    let local_root = local_root.as_ref().to_path_buf();
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
    let mut stream =
        with_timeout(async { TcpStream::connect(addr).await.map_err(PeerError::from) }).await?;
    let peer_id = with_timeout(handshake_as_initiator(&mut stream, &local_root)).await?;
    if peer_id.as_str() != peer_identity_id {
        return Err(PeerError::IdentityMismatch);
    }
    let (local_id, _) = aira_object::Keyring::load_node_identity(&local_root)?;
    Ok(AuthenticatedPeer {
        stream,
        local_root,
        peer_id,
        local_id,
    })
}

/// Accept one inbound connection and complete hello.
pub async fn accept(
    listener: &TcpListener,
    local_root: impl AsRef<Path>,
) -> Result<AuthenticatedPeer, PeerError> {
    let local_root = local_root.as_ref().to_path_buf();
    let (mut stream, _addr) =
        with_timeout(async { listener.accept().await.map_err(PeerError::from) }).await?;
    let peer_id = with_timeout(handshake_as_responder(&mut stream, &local_root)).await?;
    let (local_id, _) = aira_object::Keyring::load_node_identity(&local_root)?;
    Ok(AuthenticatedPeer {
        stream,
        local_root,
        peer_id,
        local_id,
    })
}

fn is_loopback_bind(bind: &str) -> bool {
    bind.starts_with("127.0.0.1:") || bind.starts_with("[::1]:") || bind.starts_with("localhost:")
}

/// Bind a **loopback** listener for inbound peer links.
///
/// Non-loopback binds are rejected in P0 — use [`listen_explicit`] for overrides.
pub async fn listen(bind: &str) -> Result<TcpListener, PeerError> {
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
    Ok(TcpListener::bind(bind).await?)
}
