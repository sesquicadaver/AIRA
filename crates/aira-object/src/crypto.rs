//! Ed25519 helpers + process keyring (Alpha.2 / Analyze-21).

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock};

use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Deserialize;
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
    #[error("identity io: {0}")]
    Io(String),
    #[error("no signing key registered for: {0}")]
    NoSigningKey(String),
}

/// In-memory verifying (+ optional signing) keys keyed by identity ref.
#[derive(Debug, Default, Clone)]
pub struct Keyring {
    verifying: HashMap<String, VerifyingKey>,
    signing: HashMap<String, SigningKey>,
}

impl Keyring {
    /// Empty ring (callers usually want [`Keyring::with_local_test`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// Ring that always includes deterministic local-test keys.
    pub fn with_local_test() -> Self {
        let mut k = Self::new();
        let sk = local_test_signing_key();
        let id = LOCAL_TEST_KEY_REF.to_string();
        k.verifying.insert(id.clone(), sk.verifying_key());
        k.signing.insert(id, sk);
        k
    }

    /// Register a verifying key for `key_ref`.
    pub fn insert_verifying(&mut self, key_ref: AiraRef, verifying: VerifyingKey) {
        self.verifying
            .insert(key_ref.as_str().to_string(), verifying);
    }

    /// Register signing + verifying material for `key_ref`.
    pub fn insert_signing(&mut self, key_ref: AiraRef, signing: SigningKey) {
        let id = key_ref.as_str().to_string();
        self.verifying.insert(id.clone(), signing.verifying_key());
        self.signing.insert(id, signing);
    }

    /// Resolve verifying key for a ref.
    pub fn verifying_key(&self, key_ref: &str) -> Option<&VerifyingKey> {
        self.verifying.get(key_ref)
    }

    /// Sign with a registered signing key.
    pub fn sign(&self, key_ref: &AiraRef, message: &[u8]) -> Result<Signature, CryptoError> {
        let sk = self
            .signing
            .get(key_ref.as_str())
            .ok_or_else(|| CryptoError::NoSigningKey(key_ref.as_str().to_string()))?;
        Ok(sign_with_key(key_ref.clone(), sk, message))
    }

    /// Verify using keys in this ring only.
    pub fn verify(&self, signature: &Signature, message: &[u8]) -> Result<(), CryptoError> {
        if signature.algorithm != "ed25519" {
            return Err(CryptoError::UnsupportedAlgorithm(
                signature.algorithm.clone(),
            ));
        }
        let raw = signature.signature_value.trim();
        if raw.is_empty() || raw == "TESTSIG" {
            return Err(CryptoError::MissingOrLegacy);
        }
        let vk = self
            .verifying_key(signature.key_ref.as_str())
            .ok_or_else(|| CryptoError::UnknownKey(signature.key_ref.as_str().to_string()))?;
        let bytes = hex::decode(raw).map_err(|_| CryptoError::InvalidEncoding)?;
        let sig_bytes: [u8; 64] = bytes.try_into().map_err(|_| CryptoError::InvalidEncoding)?;
        let dalek = DalekSignature::from_bytes(&sig_bytes);
        vk.verify(message, &dalek)
            .map_err(|_| CryptoError::VerifyFailed)
    }

    /// Load identity descriptor + secret from a node root (`.aira`).
    ///
    /// Expects `identity/local.identity.json` and `identity/local.ed25519` as written by
    /// `aira identity create`.
    pub fn load_node_identity(root: impl AsRef<Path>) -> Result<(AiraRef, Self), CryptoError> {
        let root = root.as_ref();
        let json_path = root.join("identity").join("local.identity.json");
        let key_path = root.join("identity").join("local.ed25519");
        if !json_path.exists() {
            return Err(CryptoError::Io(format!("missing {}", json_path.display())));
        }
        let raw = fs::read_to_string(&json_path).map_err(|e| CryptoError::Io(e.to_string()))?;
        let desc: NodeIdentityFile =
            serde_json::from_str(&raw).map_err(|e| CryptoError::Io(e.to_string()))?;
        let key_ref = AiraRef::parse(&desc.identity_id).map_err(|_| CryptoError::InvalidKey)?;
        let mut ring = Self::with_local_test();
        if key_path.exists() {
            let secret_hex =
                fs::read_to_string(&key_path).map_err(|e| CryptoError::Io(e.to_string()))?;
            let secret = parse_secret_hex(secret_hex.trim())?;
            let sk = SigningKey::from_bytes(&secret);
            // Public key in JSON must match secret.
            let expected = hex::encode(sk.verifying_key().to_bytes());
            if expected != desc.public_key.key_hex.trim() {
                return Err(CryptoError::InvalidKey);
            }
            ring.insert_signing(key_ref.clone(), sk);
        } else {
            let pk = parse_public_hex(desc.public_key.key_hex.trim())?;
            ring.insert_verifying(key_ref.clone(), pk);
        }
        Ok((key_ref, ring))
    }
}

#[derive(Debug, Deserialize)]
struct NodeIdentityFile {
    identity_id: String,
    public_key: NodePublicKey,
}

#[derive(Debug, Deserialize)]
struct NodePublicKey {
    key_hex: String,
}

fn parse_secret_hex(s: &str) -> Result<[u8; 32], CryptoError> {
    let bytes = hex::decode(s).map_err(|_| CryptoError::InvalidKey)?;
    bytes.try_into().map_err(|_| CryptoError::InvalidKey)
}

fn parse_public_hex(s: &str) -> Result<VerifyingKey, CryptoError> {
    let bytes = hex::decode(s).map_err(|_| CryptoError::InvalidKey)?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::InvalidKey)?;
    VerifyingKey::from_bytes(&arr).map_err(|_| CryptoError::InvalidKey)
}

fn process_keyring() -> &'static RwLock<Keyring> {
    static RING: OnceLock<RwLock<Keyring>> = OnceLock::new();
    RING.get_or_init(|| RwLock::new(Keyring::with_local_test()))
}

/// Merge verifying/signing keys from `ring` into the process keyring (local-test preserved).
pub fn register_keyring(ring: &Keyring) {
    let mut guard = process_keyring().write().unwrap_or_else(|e| e.into_inner());
    for (id, vk) in &ring.verifying {
        guard.verifying.insert(id.clone(), *vk);
    }
    for (id, sk) in &ring.signing {
        guard.signing.insert(id.clone(), sk.clone());
    }
}

/// Load node identity from `root` and register into the process keyring.
pub fn register_node_identity(root: impl AsRef<Path>) -> Result<Option<AiraRef>, CryptoError> {
    let json_path = root.as_ref().join("identity").join("local.identity.json");
    if !json_path.exists() {
        return Ok(None);
    }
    let (id, ring) = Keyring::load_node_identity(root)?;
    register_keyring(&ring);
    Ok(Some(id))
}

/// Snapshot of the process keyring (for CLI sign).
pub fn process_keyring_snapshot() -> Keyring {
    process_keyring()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
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

/// Verify an Ed25519 signature over `message` using the process keyring.
///
/// The process keyring always includes `aira:identity:local-test`. Node identities
/// registered via [`register_node_identity`] are also resolved.
pub fn verify_ed25519(signature: &Signature, message: &[u8]) -> Result<(), CryptoError> {
    let ring = process_keyring().read().unwrap_or_else(|e| e.into_inner());
    ring.verify(signature, message)
}

/// True when signature material is non-empty and not the legacy TESTSIG placeholder.
pub fn is_cryptographic_signature(signature: &Signature) -> bool {
    let v = signature.signature_value.trim();
    !v.is_empty() && v != "TESTSIG"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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

    #[test]
    fn node_identity_keyring_sign_verify() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let id = "aira:identity:node-demo";
        let pub_hex = hex::encode(sk.verifying_key().to_bytes());
        fs::write(
            root.join("identity/local.ed25519"),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-demo",
                "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();

        let (loaded_id, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded_id.as_str(), id);
        let msg = b"node-bound-message";
        let sig = ring.sign(&AiraRef::parse(id).unwrap(), msg).unwrap();
        ring.verify(&sig, msg).unwrap();
        // local-test still present
        ring.verify(&local_test_signature(msg), msg).unwrap();

        register_keyring(&ring);
        verify_ed25519(&sig, msg).unwrap();
    }
}
