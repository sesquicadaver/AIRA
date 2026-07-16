//! Ed25519 helpers for Alpha.2 (deterministic local-test identity).

use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey};
use thiserror::Error;

use crate::types::{AiraRef, Signature};

/// Canonical key ref for the MVP local-test identity.
pub const LOCAL_TEST_KEY_REF: &str = "aira:identity:local-test";

/// Fixed 32-byte seed — deterministic fixtures/tests only (not a production secret).
const LOCAL_TEST_SEED: [u8; 32] = *b"aira-mvp-local-test-ed25519-key!";

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
}

/// Signing key for `aira:identity:local-test`.
pub fn local_test_signing_key() -> SigningKey {
    SigningKey::from_bytes(&LOCAL_TEST_SEED)
}

/// Verifying key for `aira:identity:local-test`.
pub fn local_test_verifying_key() -> VerifyingKey {
    local_test_signing_key().verifying_key()
}

/// Hex-encoded public key for local-test (stable across builds).
pub fn local_test_public_key_hex() -> String {
    hex::encode(local_test_verifying_key().to_bytes())
}

/// Sign `message` with local-test key → Signature envelope.
pub fn local_test_signature(message: &[u8]) -> Signature {
    sign_with_key(
        AiraRef::parse(LOCAL_TEST_KEY_REF).expect("local-test ref"),
        &local_test_signing_key(),
        message,
    )
}

/// Sign message with an Ed25519 signing key; value is lowercase hex of 64-byte signature.
pub fn sign_with_key(key_ref: AiraRef, signing: &SigningKey, message: &[u8]) -> Signature {
    let sig = signing.sign(message);
    Signature {
        algorithm: "ed25519".into(),
        key_ref,
        signature_value: hex::encode(sig.to_bytes()),
    }
}

/// Verify an Ed25519 signature over `message`.
///
/// Alpha.2 resolves only `aira:identity:local-test`. Rejects empty and `TESTSIG`.
pub fn verify_ed25519(signature: &Signature, message: &[u8]) -> Result<(), CryptoError> {
    if signature.algorithm != "ed25519" {
        return Err(CryptoError::UnsupportedAlgorithm(
            signature.algorithm.clone(),
        ));
    }
    let raw = signature.signature_value.trim();
    if raw.is_empty() || raw == "TESTSIG" {
        return Err(CryptoError::MissingOrLegacy);
    }
    if signature.key_ref.as_str() != LOCAL_TEST_KEY_REF {
        return Err(CryptoError::UnknownKey(
            signature.key_ref.as_str().to_string(),
        ));
    }
    let bytes = hex::decode(raw).map_err(|_| CryptoError::InvalidEncoding)?;
    let sig_bytes: [u8; 64] = bytes.try_into().map_err(|_| CryptoError::InvalidEncoding)?;
    let dalek = DalekSignature::from_bytes(&sig_bytes);
    local_test_verifying_key()
        .verify(message, &dalek)
        .map_err(|_| CryptoError::VerifyFailed)
}

/// True when signature material is non-empty and not the legacy TESTSIG placeholder.
pub fn is_cryptographic_signature(signature: &Signature) -> bool {
    let v = signature.signature_value.trim();
    !v.is_empty() && v != "TESTSIG"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_test_sign_verify_roundtrip() {
        let msg = b"aira:artifact:hash-demo";
        let sig = local_test_signature(msg);
        assert_ne!(sig.signature_value, "TESTSIG");
        assert_eq!(sig.signature_value.len(), 128);
        verify_ed25519(&sig, msg).unwrap();
    }

    #[test]
    fn rejects_testsig_and_tamper() {
        let msg = b"payload";
        let mut sig = local_test_signature(msg);
        assert!(verify_ed25519(
            &Signature {
                algorithm: "ed25519".into(),
                key_ref: sig.key_ref.clone(),
                signature_value: "TESTSIG".into(),
            },
            msg
        )
        .is_err());
        // Flip one hex nibble.
        let mut chars: Vec<char> = sig.signature_value.chars().collect();
        chars[0] = if chars[0] == '0' { '1' } else { '0' };
        sig.signature_value = chars.into_iter().collect();
        assert_eq!(verify_ed25519(&sig, msg), Err(CryptoError::VerifyFailed));
    }

    #[test]
    fn public_key_hex_is_stable() {
        let hex = local_test_public_key_hex();
        assert_eq!(hex.len(), 64);
        assert_eq!(hex, local_test_public_key_hex());
    }
}
