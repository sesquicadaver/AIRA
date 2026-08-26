//! Crypto errors, local-test constants, and parse helpers (Analyze-82).

use ed25519_dalek::VerifyingKey;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Canonical key ref for the MVP local-test identity.
pub const LOCAL_TEST_KEY_REF: &str = "aira:identity:local-test";

/// Fixed 32-byte seed — deterministic fixtures/tests only (not a production secret).
pub(super) const LOCAL_TEST_SEED: [u8; 32] = *b"aira-mvp-local-test-ed25519-key!";

/// Domain message when a helper has no content hash yet (presence signatures).
pub const LOCAL_TEST_DOMAIN_MSG: &[u8] = b"aira:domain:local-test:v0";

/// Crypto errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoError {
    #[error("unsupported signature algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("missing or legacy TESTSIG signature")]
    MissingOrLegacy,
    #[error("unknown key_ref for verify: {0}")]
    UnknownKey(String),
    #[error("invalid signature encoding")]
    InvalidEncoding,
    #[error("signature verification failed")]
    VerifyFailed,
    #[error("invalid key material")]
    InvalidKey,
    #[error("identity io: {0}")]
    Io(String),
    #[error("no signing key registered for: {0}")]
    NoSigningKey(String),
    #[error("identity is revoked and cannot be trusted: {0}")]
    RevokedKey(String),
    #[error("cannot revoke protected identity: {0}")]
    ProtectedIdentity(String),
    #[error("identity is not on the CRL: {0}")]
    NotRevoked(String),
    #[error("identity is not currently trusted: {0}")]
    NotTrusted(String),
    #[error("old and new identity refs must differ")]
    SameIdentity,
    #[error("invalid grace_until timestamp (need RFC3339 UTC): {0}")]
    InvalidTimestamp(String),
    #[error("csu tenant isolation: {0}")]
    TenantIsolation(String),
    #[error("signature.key_ref must equal producer_identity ({key_ref} != {producer})")]
    ProducerIdentityMismatch { producer: String, key_ref: String },
}
/// True when `grace_until` is still active at process UTC now.
pub(super) fn node_grace_active(grace_until: &str) -> Result<bool, CryptoError> {
    let until = parse_rfc3339(grace_until)?;
    let now = parse_rfc3339(&utc_now_rfc3339()?)?;
    Ok(now <= until)
}

pub(super) fn parse_secret_hex(s: &str) -> Result<[u8; 32], CryptoError> {
    let bytes = hex::decode(s).map_err(|_| CryptoError::InvalidKey)?;
    bytes.try_into().map_err(|_| CryptoError::InvalidKey)
}

pub(super) fn parse_public_hex(s: &str) -> Result<VerifyingKey, CryptoError> {
    let bytes = hex::decode(s).map_err(|_| CryptoError::InvalidKey)?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::InvalidKey)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| CryptoError::InvalidKey)
}

/// Parse and normalize an RFC3339 timestamp to a canonical UTC string.
pub fn normalize_rfc3339(s: &str) -> Result<String, CryptoError> {
    let dt = parse_rfc3339(s)?;
    dt.format(&Rfc3339)
        .map_err(|e| CryptoError::InvalidTimestamp(e.to_string()))
}

/// Current UTC time as RFC3339.
pub fn utc_now_rfc3339() -> Result<String, CryptoError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| CryptoError::InvalidTimestamp(e.to_string()))
}

pub(super) fn parse_rfc3339(s: &str) -> Result<OffsetDateTime, CryptoError> {
    OffsetDateTime::parse(s.trim(), &Rfc3339)
        .map_err(|e| CryptoError::InvalidTimestamp(format!("{} ({e})", s.trim())))
}
