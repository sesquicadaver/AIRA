//! Ed25519 helpers + process keyring (Alpha.2 / Analyze-21).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock};

use ed25519_dalek::{Signature as DalekSignature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

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

/// One trusted verifying identity (public key only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustEntry {
    pub identity_id: String,
    #[serde(default = "default_ed25519")]
    pub algorithm: String,
    pub public_key_hex: String,
    /// Prior identity this entry replaced (set by [`TrustStore::rotate`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
}

/// Durable revocation record (CRL entry) — blocks re-trust via upsert.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevokedEntry {
    pub identity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Successor identity when revoked via [`TrustStore::rotate`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    /// Dual-key grace end (RFC3339 UTC). While `now <= grace_until`, the revoked
    /// pubkey remains verifiable alongside the successor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_until: Option<String>,
}

fn default_ed25519() -> String {
    "ed25519".into()
}

/// Persistent trust store under `.aira/identity/trust.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TrustStore {
    #[serde(default)]
    pub entries: Vec<TrustEntry>,
    /// Durable CRL — revoked identities cannot be re-added via [`TrustStore::upsert`].
    #[serde(default)]
    pub revoked: Vec<RevokedEntry>,
}

impl TrustStore {
    /// Path to trust.json under a node root.
    pub fn path(root: impl AsRef<Path>) -> std::path::PathBuf {
        root.as_ref().join("identity").join("trust.json")
    }

    /// Load trust store; missing file → empty store.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, CryptoError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| CryptoError::Io(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| CryptoError::Io(e.to_string()))
    }

    /// Persist trust store (creates identity dir as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), CryptoError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CryptoError::Io(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| CryptoError::Io(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| CryptoError::Io(e.to_string()))
    }

    /// True when `identity_id` is on the durable CRL.
    pub fn is_revoked(&self, identity_id: &str) -> bool {
        self.revoked.iter().any(|r| r.identity_id == identity_id)
    }

    /// Insert or replace an entry by identity_id.
    ///
    /// Fails with [`CryptoError::RevokedKey`] if the id is on the CRL.
    pub fn upsert(&mut self, identity_id: &str, public_key_hex: &str) -> Result<(), CryptoError> {
        let _ = parse_public_hex(public_key_hex.trim())?;
        let id = identity_id.trim();
        AiraRef::parse(id).map_err(|_| CryptoError::InvalidKey)?;
        if self.is_revoked(id) {
            return Err(CryptoError::RevokedKey(id.to_string()));
        }
        if let Some(e) = self.entries.iter_mut().find(|e| e.identity_id == id) {
            e.public_key_hex = public_key_hex.trim().to_string();
            e.algorithm = "ed25519".into();
        } else {
            self.entries.push(TrustEntry {
                identity_id: id.to_string(),
                algorithm: "ed25519".into(),
                public_key_hex: public_key_hex.trim().to_string(),
                supersedes: None,
            });
        }
        self.entries
            .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Remove entry; returns whether it existed. Does **not** add to CRL (use [`TrustStore::revoke`]).
    pub fn remove(&mut self, identity_id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.identity_id != identity_id);
        self.entries.len() != before
    }

    /// Durably revoke an identity: drop from entries, append CRL, block re-upsert.
    ///
    /// Refuses [`LOCAL_TEST_KEY_REF`]. Idempotent if already revoked.
    pub fn revoke(
        &mut self,
        identity_id: &str,
        reason: Option<&str>,
    ) -> Result<(), CryptoError> {
        let id = identity_id.trim();
        if id == LOCAL_TEST_KEY_REF {
            return Err(CryptoError::ProtectedIdentity(LOCAL_TEST_KEY_REF.into()));
        }
        AiraRef::parse(id).map_err(|_| CryptoError::InvalidKey)?;
        let pk = self
            .entries
            .iter()
            .find(|e| e.identity_id == id)
            .map(|e| e.public_key_hex.clone());
        let _ = self.remove(id);
        if !self.is_revoked(id) {
            self.revoked.push(RevokedEntry {
                identity_id: id.to_string(),
                public_key_hex: pk,
                reason: reason.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                superseded_by: None,
                grace_until: None,
            });
            self.revoked
                .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        }
        Ok(())
    }

    /// Atomically replace a trusted peer identity with a new key_ref + pubkey.
    ///
    /// - `old_ref` must currently be in `entries` (not merely on CRL).
    /// - `old_ref` is revoked with `superseded_by = new_ref`.
    /// - `new_ref` is upserted with `supersedes = old_ref` (replaces pubkey if already trusted).
    /// - If `grace_until` is set (RFC3339 UTC), old pubkey stays verifiable until that instant
    ///   via [`TrustStore::to_keyring_at`] / [`sync_trust_verifiers`]. Omit for immediate cutover.
    ///
    /// Refuses [`LOCAL_TEST_KEY_REF`] as either side; refuses identical refs.
    pub fn rotate(
        &mut self,
        old_ref: &str,
        new_ref: &str,
        new_pubkey_hex: &str,
        reason: Option<&str>,
        grace_until: Option<&str>,
    ) -> Result<(), CryptoError> {
        let old = old_ref.trim();
        let new = new_ref.trim();
        if old == LOCAL_TEST_KEY_REF || new == LOCAL_TEST_KEY_REF {
            return Err(CryptoError::ProtectedIdentity(LOCAL_TEST_KEY_REF.into()));
        }
        if old == new {
            return Err(CryptoError::SameIdentity);
        }
        AiraRef::parse(old).map_err(|_| CryptoError::InvalidKey)?;
        AiraRef::parse(new).map_err(|_| CryptoError::InvalidKey)?;
        let _ = parse_public_hex(new_pubkey_hex.trim())?;
        let grace_until = match grace_until {
            Some(s) => Some(normalize_rfc3339(s)?),
            None => None,
        };
        if self.is_revoked(new) {
            return Err(CryptoError::RevokedKey(new.to_string()));
        }
        let old_pk = self
            .entries
            .iter()
            .find(|e| e.identity_id == old)
            .map(|e| e.public_key_hex.clone())
            .ok_or_else(|| CryptoError::NotTrusted(old.to_string()))?;

        let _ = self.remove(old);
        if !self.is_revoked(old) {
            let reason = reason
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .or_else(|| Some(format!("rotated to {new}")));
            self.revoked.push(RevokedEntry {
                identity_id: old.to_string(),
                public_key_hex: Some(old_pk),
                reason,
                superseded_by: Some(new.to_string()),
                grace_until,
            });
            self.revoked
                .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        }

        if let Some(e) = self.entries.iter_mut().find(|e| e.identity_id == new) {
            e.public_key_hex = new_pubkey_hex.trim().to_string();
            e.algorithm = "ed25519".into();
            e.supersedes = Some(old.to_string());
        } else {
            self.entries.push(TrustEntry {
                identity_id: new.to_string(),
                algorithm: "ed25519".into(),
                public_key_hex: new_pubkey_hex.trim().to_string(),
                supersedes: Some(old.to_string()),
            });
        }
        self.entries
            .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Remove an identity from the durable CRL.
    ///
    /// Does **not** restore `entries` or register verifying keys — callers must
    /// [`TrustStore::upsert`] / `trust add` explicitly. Fails with
    /// [`CryptoError::NotRevoked`] if the id is not on the CRL.
    pub fn unrevoke(&mut self, identity_id: &str) -> Result<(), CryptoError> {
        let id = identity_id.trim();
        AiraRef::parse(id).map_err(|_| CryptoError::InvalidKey)?;
        let before = self.revoked.len();
        self.revoked.retain(|r| r.identity_id != id);
        if self.revoked.len() == before {
            return Err(CryptoError::NotRevoked(id.to_string()));
        }
        Ok(())
    }

    /// Ensure local-test public key is trusted.
    pub fn ensure_local_test(&mut self) -> Result<(), CryptoError> {
        self.upsert(LOCAL_TEST_KEY_REF, &local_test_public_key_hex())
    }

    /// Build a verifying-only keyring from active entries (revoked excluded; no grace).
    pub fn to_keyring(&self) -> Result<Keyring, CryptoError> {
        self.to_keyring_at(&utc_now_rfc3339()?)
    }

    /// Build verifying keyring at `now` (RFC3339): active entries + CRL entries still in grace.
    pub fn to_keyring_at(&self, now_rfc3339: &str) -> Result<Keyring, CryptoError> {
        let now = parse_rfc3339(now_rfc3339)?;
        let mut ring = Keyring::new();
        for e in &self.entries {
            if e.algorithm != "ed25519" {
                return Err(CryptoError::UnsupportedAlgorithm(e.algorithm.clone()));
            }
            let id = AiraRef::parse(&e.identity_id).map_err(|_| CryptoError::InvalidKey)?;
            let vk = parse_public_hex(e.public_key_hex.trim())?;
            ring.insert_verifying(id, vk);
        }
        for r in &self.revoked {
            if !r.grace_active_at(now)? {
                continue;
            }
            let Some(pk) = r.public_key_hex.as_deref() else {
                continue;
            };
            let id = AiraRef::parse(&r.identity_id).map_err(|_| CryptoError::InvalidKey)?;
            let vk = parse_public_hex(pk.trim())?;
            ring.insert_verifying(id, vk);
        }
        Ok(ring)
    }

    /// Identity ids on the CRL that still have an active dual-key grace at `now`.
    pub fn grace_active_ids(&self, now_rfc3339: &str) -> Result<HashSet<String>, CryptoError> {
        let now = parse_rfc3339(now_rfc3339)?;
        let mut out = HashSet::new();
        for r in &self.revoked {
            if r.grace_active_at(now)? {
                out.insert(r.identity_id.clone());
            }
        }
        Ok(out)
    }
}

impl RevokedEntry {
    fn grace_active_at(&self, now: OffsetDateTime) -> Result<bool, CryptoError> {
        let Some(until) = self.grace_until.as_deref() else {
            return Ok(false);
        };
        let until = parse_rfc3339(until)?;
        Ok(now <= until && self.public_key_hex.is_some())
    }
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

fn parse_rfc3339(s: &str) -> Result<OffsetDateTime, CryptoError> {
    OffsetDateTime::parse(s.trim(), &Rfc3339)
        .map_err(|e| CryptoError::InvalidTimestamp(format!("{} ({e})", s.trim())))
}

/// Load trust.json verifying keys (including active dual-key grace) into the process keyring.
pub fn register_trust_store(root: impl AsRef<Path>) -> Result<usize, CryptoError> {
    let store = TrustStore::load(&root)?;
    let now = utc_now_rfc3339()?;
    let ring = store.to_keyring_at(&now)?;
    register_keyring(&ring);
    Ok(store.entries.len())
}

/// Prune process verifying keys absent from trust/grace, then re-register.
///
/// - Never unloads [`LOCAL_TEST_KEY_REF`].
/// - Active trust entries and CRL entries with active `grace_until` stay verifiable.
/// - Identities with signing material keep verifying keys unless revoked **and** not in grace.
pub fn sync_trust_verifiers(root: impl AsRef<Path>) -> Result<usize, CryptoError> {
    let store = TrustStore::load(&root)?;
    let now = utc_now_rfc3339()?;
    let trusted: HashSet<String> = store
        .entries
        .iter()
        .map(|e| e.identity_id.clone())
        .collect();
    let grace = store.grace_active_ids(&now)?;
    let revoked: HashSet<String> = store
        .revoked
        .iter()
        .map(|e| e.identity_id.clone())
        .collect();

    {
        let mut guard = process_keyring().write().unwrap_or_else(|e| e.into_inner());
        let ids: Vec<String> = guard.verifying.keys().cloned().collect();
        for id in ids {
            if id == LOCAL_TEST_KEY_REF || trusted.contains(&id) || grace.contains(&id) {
                continue;
            }
            if revoked.contains(&id) {
                guard.verifying.remove(&id);
                continue;
            }
            if let Some(sk) = guard.signing.get(&id).cloned() {
                guard.verifying.insert(id, sk.verifying_key());
                continue;
            }
            guard.verifying.remove(&id);
        }
    }

    register_trust_store(root)
}

/// Ensure local-test (+ node identity if present) are in trust.json and registered.
pub fn ensure_trust_defaults(root: impl AsRef<Path>) -> Result<TrustStore, CryptoError> {
    let root = root.as_ref();
    let mut store = TrustStore::load(root)?;
    store.ensure_local_test()?;
    let id_path = root.join("identity").join("local.identity.json");
    if id_path.exists() {
        let raw = fs::read_to_string(&id_path).map_err(|e| CryptoError::Io(e.to_string()))?;
        let desc: NodeIdentityFile =
            serde_json::from_str(&raw).map_err(|e| CryptoError::Io(e.to_string()))?;
        store.upsert(&desc.identity_id, desc.public_key.key_hex.trim())?;
    }
    store.save(root)?;
    let _ = sync_trust_verifiers(root)?;
    Ok(store)
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
pub fn active_signature(message: &[u8]) -> Signature {
    let id = primary_signer();
    let ring = process_keyring_snapshot();
    match ring.sign(&id, message) {
        Ok(sig) => sig,
        Err(_) => local_test_signature(message),
    }
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

        set_primary_signer(loaded_id.clone());
        assert_eq!(active_identity().as_str(), id);
        let active = active_signature(msg);
        assert_eq!(active.key_ref.as_str(), id);
        verify_ed25519(&active, msg).unwrap();
        reset_primary_signer();
        assert_eq!(primary_signer().as_str(), LOCAL_TEST_KEY_REF);
    }

    #[test]
    fn trust_store_peer_verify_without_signing_key() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let peer_sk = SigningKey::from_bytes(&[13u8; 32]);
        let peer_id = "aira:identity:peer-alice";
        let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());

        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(peer_id, &peer_pub).unwrap();
        store.save(root).unwrap();

        let loaded = TrustStore::load(root).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        let _ = register_trust_store(root).unwrap();

        let msg = b"peer-message";
        let sig = sign_with_key(AiraRef::parse(peer_id).unwrap(), &peer_sk, msg);
        // File-backed ring (process keyring is shared across parallel tests).
        let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
        ring.verify(&sig, msg).unwrap();

        store.remove(peer_id);
        store.save(root).unwrap();
        let _ = sync_trust_verifiers(root).unwrap();
        let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
        assert!(ring.verifying_key(peer_id).is_none());
        assert!(ring.verifying_key(LOCAL_TEST_KEY_REF).is_some());
        assert!(ring.verify(&sig, msg).is_err());
        ring.verify(&local_test_signature(msg), msg).unwrap();
    }

    #[test]
    fn trust_crl_revoke_blocks_readd_and_verify() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let peer_sk = SigningKey::from_bytes(&[17u8; 32]);
        let peer_id = "aira:identity:peer-bob";
        let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());

        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(peer_id, &peer_pub).unwrap();
        store.save(root).unwrap();
        let _ = register_trust_store(root).unwrap();

        let msg = b"crl-message";
        let sig = sign_with_key(AiraRef::parse(peer_id).unwrap(), &peer_sk, msg);
        TrustStore::load(root)
            .unwrap()
            .to_keyring()
            .unwrap()
            .verify(&sig, msg)
            .unwrap();

        store.revoke(peer_id, Some("compromised")).unwrap();
        store.save(root).unwrap();
        let _ = sync_trust_verifiers(root).unwrap();
        assert!(store.is_revoked(peer_id));
        let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
        assert!(ring.verifying_key(peer_id).is_none());
        assert!(ring.verify(&sig, msg).is_err());
        assert_eq!(
            store.upsert(peer_id, &peer_pub),
            Err(CryptoError::RevokedKey(peer_id.into()))
        );
        assert!(TrustStore::default()
            .revoke(LOCAL_TEST_KEY_REF, None)
            .is_err());
        ring.verify(&local_test_signature(msg), msg).unwrap();
    }

    #[test]
    fn trust_crl_unrevoke_allows_explicit_readd() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let peer_sk = SigningKey::from_bytes(&[19u8; 32]);
        let peer_id = "aira:identity:peer-carol";
        let peer_pub = hex::encode(peer_sk.verifying_key().to_bytes());

        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(peer_id, &peer_pub).unwrap();
        store.save(root).unwrap();
        let _ = register_trust_store(root).unwrap();

        let msg = b"unrevoke-message";
        let sig = sign_with_key(AiraRef::parse(peer_id).unwrap(), &peer_sk, msg);
        TrustStore::load(root)
            .unwrap()
            .to_keyring()
            .unwrap()
            .verify(&sig, msg)
            .unwrap();

        store.revoke(peer_id, Some("temp")).unwrap();
        store.save(root).unwrap();
        let _ = sync_trust_verifiers(root).unwrap();
        assert_eq!(
            store.upsert(peer_id, &peer_pub),
            Err(CryptoError::RevokedKey(peer_id.into()))
        );

        store.unrevoke(peer_id).unwrap();
        assert!(!store.is_revoked(peer_id));
        assert_eq!(
            store.unrevoke(peer_id),
            Err(CryptoError::NotRevoked(peer_id.into()))
        );
        // Unrevoke alone must not restore entries / verifying key.
        assert!(!store.entries.iter().any(|e| e.identity_id == peer_id));
        assert!(TrustStore::load(root)
            .unwrap()
            .to_keyring()
            .unwrap()
            .verifying_key(peer_id)
            .is_none());

        store.upsert(peer_id, &peer_pub).unwrap();
        store.save(root).unwrap();
        let _ = sync_trust_verifiers(root).unwrap();
        TrustStore::load(root)
            .unwrap()
            .to_keyring()
            .unwrap()
            .verify(&sig, msg)
            .unwrap();
    }

    #[test]
    fn trust_rotate_revokes_old_trusts_new() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[21u8; 32]);
        let new_sk = SigningKey::from_bytes(&[23u8; 32]);
        let old_id = "aira:identity:peer-old";
        let new_id = "aira:identity:peer-new";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(old_id, &old_pub).unwrap();
        store.save(root).unwrap();
        let _ = register_trust_store(root).unwrap();

        let msg = b"rotate-message";
        let old_sig = sign_with_key(AiraRef::parse(old_id).unwrap(), &old_sk, msg);
        let new_sig = sign_with_key(AiraRef::parse(new_id).unwrap(), &new_sk, msg);
        TrustStore::load(root)
            .unwrap()
            .to_keyring()
            .unwrap()
            .verify(&old_sig, msg)
            .unwrap();

        store
            .rotate(old_id, new_id, &new_pub, Some("key rollover"), None)
            .unwrap();
        store.save(root).unwrap();
        let _ = sync_trust_verifiers(root).unwrap();

        assert!(store.is_revoked(old_id));
        let revoked = store
            .revoked
            .iter()
            .find(|r| r.identity_id == old_id)
            .unwrap();
        assert_eq!(revoked.superseded_by.as_deref(), Some(new_id));
        let entry = store
            .entries
            .iter()
            .find(|e| e.identity_id == new_id)
            .unwrap();
        assert_eq!(entry.supersedes.as_deref(), Some(old_id));

        let ring = TrustStore::load(root).unwrap().to_keyring().unwrap();
        assert!(ring.verifying_key(old_id).is_none());
        assert!(ring.verifying_key(new_id).is_some());
        assert!(ring.verify(&old_sig, msg).is_err());
        ring.verify(&new_sig, msg).unwrap();
        assert_eq!(
            store.upsert(old_id, &old_pub),
            Err(CryptoError::RevokedKey(old_id.into()))
        );
        assert_eq!(
            store.rotate(old_id, new_id, &new_pub, None, None),
            Err(CryptoError::NotTrusted(old_id.into()))
        );
        assert_eq!(
            store.rotate(new_id, new_id, &new_pub, None, None),
            Err(CryptoError::SameIdentity)
        );
        assert!(store
            .rotate(LOCAL_TEST_KEY_REF, new_id, &new_pub, None, None)
            .is_err());
    }

    #[test]
    fn trust_rotate_grace_allows_old_until() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[25u8; 32]);
        let new_sk = SigningKey::from_bytes(&[27u8; 32]);
        let old_id = "aira:identity:peer-grace-old";
        let new_id = "aira:identity:peer-grace-new";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(old_id, &old_pub).unwrap();
        store.save(root).unwrap();

        let msg = b"grace-message";
        let old_sig = sign_with_key(AiraRef::parse(old_id).unwrap(), &old_sk, msg);
        let new_sig = sign_with_key(AiraRef::parse(new_id).unwrap(), &new_sk, msg);

        store
            .rotate(
                old_id,
                new_id,
                &new_pub,
                Some("grace rollover"),
                Some("2099-01-01T00:00:00Z"),
            )
            .unwrap();
        store.save(root).unwrap();

        let during = store.to_keyring_at("2026-07-16T12:00:00Z").unwrap();
        during.verify(&old_sig, msg).unwrap();
        during.verify(&new_sig, msg).unwrap();
        assert_eq!(
            store.upsert(old_id, &old_pub),
            Err(CryptoError::RevokedKey(old_id.into()))
        );

        let after = store.to_keyring_at("2099-01-01T00:00:01Z").unwrap();
        assert!(after.verifying_key(old_id).is_none());
        assert!(after.verify(&old_sig, msg).is_err());
        after.verify(&new_sig, msg).unwrap();

        assert!(store
            .rotate(old_id, new_id, &new_pub, None, Some("not-a-timestamp"))
            .is_err());
    }
}
