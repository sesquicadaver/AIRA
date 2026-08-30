//! In-memory keyring + process signer (Analyze-82).

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock};

use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Deserialize;

use crate::types::{AiraRef, Signature};

use super::error::{
    node_grace_active, parse_public_hex, parse_secret_hex, CryptoError, LOCAL_TEST_KEY_REF,
    LOCAL_TEST_SEED,
};

/// In-memory verifying (+ optional signing) keys keyed by identity ref.
///
/// Multiple verifying keys per `key_ref` enable same-identity dual-key grace
/// after node secret rotate (Analyze-37). Signing remains a single current key.
#[derive(Debug, Default, Clone)]
pub struct Keyring {
    pub(super) verifying: HashMap<String, Vec<VerifyingKey>>,
    pub(super) signing: HashMap<String, SigningKey>,
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
        k.verifying.insert(id.clone(), vec![sk.verifying_key()]);
        k.signing.insert(id, sk);
        k
    }

    /// Replace verifying key(s) for `key_ref` with a single key.
    pub fn insert_verifying(&mut self, key_ref: AiraRef, verifying: VerifyingKey) {
        self.verifying
            .insert(key_ref.as_str().to_string(), vec![verifying]);
    }

    /// Empty ring with one verifying key from hex (detached verify; no local-test).
    pub fn with_verifying_hex(
        key_ref: &AiraRef,
        public_key_hex: &str,
    ) -> Result<Self, CryptoError> {
        let mut k = Self::new();
        k.insert_verifying(key_ref.clone(), parse_public_hex(public_key_hex)?);
        Ok(k)
    }

    /// Append a verifying key for `key_ref` if not already present (dual-key grace).
    pub fn add_verifying(&mut self, key_ref: AiraRef, verifying: VerifyingKey) {
        let id = key_ref.as_str().to_string();
        let slot = self.verifying.entry(id).or_default();
        if !slot.iter().any(|k| k.as_bytes() == verifying.as_bytes()) {
            slot.push(verifying);
        }
    }

    /// Register signing + primary verifying material for `key_ref`.
    ///
    /// The new verifying key becomes first; any prior verifying keys for the same
    /// ref (grace) are retained after it.
    pub fn insert_signing(&mut self, key_ref: AiraRef, signing: SigningKey) {
        let id = key_ref.as_str().to_string();
        let vk = signing.verifying_key();
        let mut rest = self
            .verifying
            .remove(&id)
            .unwrap_or_default()
            .into_iter()
            .filter(|k| k.as_bytes() != vk.as_bytes())
            .collect::<Vec<_>>();
        let mut keys = vec![vk];
        keys.append(&mut rest);
        self.verifying.insert(id.clone(), keys);
        self.signing.insert(id, signing);
    }

    /// Primary (current) verifying key for a ref, if any.
    pub fn verifying_key(&self, key_ref: &str) -> Option<&VerifyingKey> {
        self.verifying.get(key_ref).and_then(|v| v.first())
    }

    /// All verifying keys for a ref (current first when loaded via insert_signing).
    pub fn verifying_keys(&self, key_ref: &str) -> &[VerifyingKey] {
        self.verifying
            .get(key_ref)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Sign with a registered signing key.
    pub fn sign(&self, key_ref: &AiraRef, message: &[u8]) -> Result<Signature, CryptoError> {
        let sk = self
            .signing
            .get(key_ref.as_str())
            .ok_or_else(|| CryptoError::NoSigningKey(key_ref.as_str().to_string()))?;
        Ok(sign_with_key(key_ref.clone(), sk, message))
    }

    /// Verify using keys in this ring only (tries all keys for `signature.key_ref`).
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
        let keys = self.verifying_keys(signature.key_ref.as_str());
        if keys.is_empty() {
            return Err(CryptoError::UnknownKey(
                signature.key_ref.as_str().to_string(),
            ));
        }
        let bytes = hex::decode(raw).map_err(|_| CryptoError::InvalidEncoding)?;
        let sig_bytes: [u8; 64] = bytes.try_into().map_err(|_| CryptoError::InvalidEncoding)?;
        let dalek = DalekSignature::from_bytes(&sig_bytes);
        for vk in keys {
            if vk.verify(message, &dalek).is_ok() {
                return Ok(());
            }
        }
        Err(CryptoError::VerifyFailed)
    }

    /// Load identity descriptor + secret from a node root (`.aira`).
    ///
    /// Expects `identity/local.identity.json` and `identity/local.ed25519` as written by
    /// `aira identity create`. When `previous_public_key` + active `previous_grace_until`
    /// are present, the previous verifying key is also registered for the same `key_ref`.
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
        if let (Some(prev), Some(until)) = (
            desc.previous_public_key.as_ref(),
            desc.previous_grace_until.as_deref(),
        ) {
            if node_grace_active(until)? {
                let prev_vk = parse_public_hex(prev.key_hex.trim())?;
                ring.add_verifying(key_ref.clone(), prev_vk);
            }
        }
        Ok((key_ref, ring))
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct NodeIdentityFile {
    pub(super) identity_id: String,
    pub(super) public_key: NodePublicKey,
    #[serde(default)]
    previous_public_key: Option<NodePublicKey>,
    #[serde(default)]
    previous_grace_until: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NodePublicKey {
    pub(super) key_hex: String,
}

pub(super) fn process_keyring() -> &'static RwLock<Keyring> {
    static RING: OnceLock<RwLock<Keyring>> = OnceLock::new();
    RING.get_or_init(|| RwLock::new(Keyring::with_local_test()))
}

/// Merge verifying/signing keys from `ring` into the process keyring (local-test preserved).
///
/// For each `key_ref` present in `ring.verifying`, the verifying list is **replaced**
/// (supports dual-key grace cutover and trust upserts). Signing keys are upserted.
pub fn register_keyring(ring: &Keyring) {
    let mut guard = process_keyring().write().unwrap_or_else(|e| e.into_inner());
    for (id, vks) in &ring.verifying {
        guard.verifying.insert(id.clone(), vks.clone());
    }
    for (id, sk) in &ring.signing {
        guard.signing.insert(id.clone(), sk.clone());
    }
}

/// Load node identity from `root` and register into the process keyring.
///
/// When a node identity is loaded, it becomes the [`primary_signer`].
pub fn register_node_identity(root: impl AsRef<Path>) -> Result<Option<AiraRef>, CryptoError> {
    let json_path = root.as_ref().join("identity").join("local.identity.json");
    if !json_path.exists() {
        return Ok(None);
    }
    let (id, ring) = Keyring::load_node_identity(root)?;
    register_keyring(&ring);
    set_primary_signer(id.clone());
    Ok(Some(id))
}

/// Snapshot of the process keyring (for CLI sign).
pub fn process_keyring_snapshot() -> Keyring {
    process_keyring()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn primary_slot() -> &'static RwLock<AiraRef> {
    static PRIMARY: OnceLock<RwLock<AiraRef>> = OnceLock::new();
    PRIMARY.get_or_init(|| RwLock::new(AiraRef::parse(LOCAL_TEST_KEY_REF).expect("local-test ref")))
}

/// Set the identity used by [`active_identity`] / [`active_signature`].
pub fn set_primary_signer(key_ref: AiraRef) {
    let mut g = primary_slot().write().unwrap_or_else(|e| e.into_inner());
    *g = key_ref;
}

/// Reset primary signer to local-test (tests).
pub fn reset_primary_signer() {
    set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).expect("local-test ref"));
}

/// Remove verifying material for `key_ref` from the process keyring.
///
/// Returns `false` (no-op) for [`LOCAL_TEST_KEY_REF`] and the current [`primary_signer`].
/// Never removes signing material.
pub fn unregister_verifying(key_ref: &AiraRef) -> bool {
    let id = key_ref.as_str();
    if id == LOCAL_TEST_KEY_REF || id == primary_signer().as_str() {
        return false;
    }
    let mut guard = process_keyring().write().unwrap_or_else(|e| e.into_inner());
    guard.verifying.remove(id).is_some()
}

/// Current primary producer identity (node identity when registered, else local-test).
pub fn primary_signer() -> AiraRef {
    primary_slot()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Alias for [`primary_signer`].
pub fn active_identity() -> AiraRef {
    primary_signer()
}

/// Sign `message` with the primary identity's registered signing key.
///
/// No silent fallback to [`local_test_signature`]. Demo/test use the default
/// primary `aira:identity:local-test` (which has a process signing key). A
/// non-local-test primary without a registered signing key returns [`CryptoError::NoSigningKey`].
pub fn active_signature(message: &[u8]) -> Result<Signature, CryptoError> {
    let id = primary_signer();
    process_keyring_snapshot().sign(&id, message)
}

/// Sign `message` with an explicit identity — no local-test fallback.
///
/// Used for per-CSU `publisher_identity` emits (Analyze-29).
pub fn signature_for(key_ref: &AiraRef, message: &[u8]) -> Result<Signature, CryptoError> {
    process_keyring_snapshot().sign(key_ref, message)
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
