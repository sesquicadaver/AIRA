//! AIRA peer link errors (Analyze-32).

use thiserror::Error;

/// Peer transport / admission failures.
#[derive(Debug, Error)]
pub enum PeerError {
    #[error("peer not trusted: {0}")]
    Untrusted(String),
    #[error("peer identity revoked: {0}")]
    Revoked(String),
    #[error("invalid peer signature")]
    InvalidSignature,
    #[error("issuer/signature does not match authenticated peer")]
    IdentityMismatch,
    #[error("frame too large ({0} bytes)")]
    FrameTooLarge(usize),
    #[error("truncated or empty frame")]
    TruncatedFrame,
    #[error("handshake failed: {0}")]
    Handshake(String),
    #[error("address book: {0}")]
    AddressBook(String),
    #[error("io: {0}")]
    Io(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("protocol: {0}")]
    Protocol(String),
}

impl From<std::io::Error> for PeerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<aira_object::CryptoError> for PeerError {
    fn from(e: aira_object::CryptoError) -> Self {
        Self::Crypto(e.to_string())
    }
}

impl From<serde_json::Error> for PeerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Protocol(e.to_string())
    }
}
