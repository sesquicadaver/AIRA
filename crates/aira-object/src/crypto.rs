//! Ed25519 helpers + process keyring (Alpha.2 / Analyze-21).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
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
    #[error("csu tenant isolation: {0}")]
    TenantIsolation(String),
}

/// In-memory verifying (+ optional signing) keys keyed by identity ref.
///
/// Multiple verifying keys per `key_ref` enable same-identity dual-key grace
/// after node secret rotate (Analyze-37). Signing remains a single current key.
#[derive(Debug, Default, Clone)]
pub struct Keyring {
    verifying: HashMap<String, Vec<VerifyingKey>>,
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
        k.verifying.insert(id.clone(), vec![sk.verifying_key()]);
        k.signing.insert(id, sk);
        k
    }

    /// Replace verifying key(s) for `key_ref` with a single key.
    pub fn insert_verifying(&mut self, key_ref: AiraRef, verifying: VerifyingKey) {
        self.verifying
            .insert(key_ref.as_str().to_string(), vec![verifying]);
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
struct NodeIdentityFile {
    identity_id: String,
    public_key: NodePublicKey,
    #[serde(default)]
    previous_public_key: Option<NodePublicKey>,
    #[serde(default)]
    previous_grace_until: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NodePublicKey {
    key_hex: String,
}

/// True when `grace_until` is still active at process UTC now.
fn node_grace_active(grace_until: &str) -> Result<bool, CryptoError> {
    let until = parse_rfc3339(grace_until)?;
    let now = parse_rfc3339(&utc_now_rfc3339()?)?;
    Ok(now <= until)
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
    /// Previous Ed25519 pubkey during same-id rekey grace (Analyze-50).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_public_key_hex: Option<String>,
    /// End of same-id dual-key grace (RFC3339 UTC).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_grace_until: Option<String>,
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
        let json =
            serde_json::to_string_pretty(self).map_err(|e| CryptoError::Io(e.to_string()))?;
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
            // Upsert is immediate cutover — drop any same-id grace slot.
            e.previous_public_key_hex = None;
            e.previous_grace_until = None;
        } else {
            self.entries.push(TrustEntry {
                identity_id: id.to_string(),
                algorithm: "ed25519".into(),
                public_key_hex: public_key_hex.trim().to_string(),
                supersedes: None,
                previous_public_key_hex: None,
                previous_grace_until: None,
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
    pub fn revoke(&mut self, identity_id: &str, reason: Option<&str>) -> Result<(), CryptoError> {
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
                reason: reason
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
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
            e.previous_public_key_hex = None;
            e.previous_grace_until = None;
        } else {
            self.entries.push(TrustEntry {
                identity_id: new.to_string(),
                algorithm: "ed25519".into(),
                public_key_hex: new_pubkey_hex.trim().to_string(),
                supersedes: Some(old.to_string()),
                previous_public_key_hex: None,
                previous_grace_until: None,
            });
        }
        self.entries
            .sort_by(|a, b| a.identity_id.cmp(&b.identity_id));
        Ok(())
    }

    /// Same-identity pubkey rekey (Analyze-50).
    ///
    /// - Requires `identity_id` already in `entries` (not merely on CRL).
    /// - With `grace_until` (RFC3339 UTC): keeps the prior pubkey as
    ///   `previous_public_key_hex` until that instant (dual-key via
    ///   [`TrustStore::to_keyring_at`]).
    /// - Without grace: immediate cutover (clears any previous_*).
    ///
    /// Refuses [`LOCAL_TEST_KEY_REF`]. Idempotent when new pubkey equals current
    /// (still refreshes / clears grace according to `grace_until`).
    pub fn rekey(
        &mut self,
        identity_id: &str,
        new_pubkey_hex: &str,
        grace_until: Option<&str>,
    ) -> Result<(), CryptoError> {
        let id = identity_id.trim();
        if id == LOCAL_TEST_KEY_REF {
            return Err(CryptoError::ProtectedIdentity(LOCAL_TEST_KEY_REF.into()));
        }
        AiraRef::parse(id).map_err(|_| CryptoError::InvalidKey)?;
        let new_pk = new_pubkey_hex.trim();
        let _ = parse_public_hex(new_pk)?;
        if self.is_revoked(id) {
            return Err(CryptoError::RevokedKey(id.to_string()));
        }
        let grace_until = match grace_until {
            Some(s) => Some(normalize_rfc3339(s)?),
            None => None,
        };
        let e = self
            .entries
            .iter_mut()
            .find(|e| e.identity_id == id)
            .ok_or_else(|| CryptoError::NotTrusted(id.to_string()))?;
        let old_pk = e.public_key_hex.clone();
        if let Some(until) = grace_until {
            if old_pk != new_pk {
                e.previous_public_key_hex = Some(old_pk);
                e.previous_grace_until = Some(until);
            }
            // Same pubkey + grace: leave previous slot unchanged.
        } else {
            e.previous_public_key_hex = None;
            e.previous_grace_until = None;
        }
        e.public_key_hex = new_pk.to_string();
        e.algorithm = "ed25519".into();
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

    /// Build verifying keyring at `now` (RFC3339): active entries (incl. same-id
    /// previous_* grace) + CRL entries still in grace.
    pub fn to_keyring_at(&self, now_rfc3339: &str) -> Result<Keyring, CryptoError> {
        let now = parse_rfc3339(now_rfc3339)?;
        let mut ring = Keyring::new();
        for e in &self.entries {
            if e.algorithm != "ed25519" {
                return Err(CryptoError::UnsupportedAlgorithm(e.algorithm.clone()));
            }
            let id = AiraRef::parse(&e.identity_id).map_err(|_| CryptoError::InvalidKey)?;
            let vk = parse_public_hex(e.public_key_hex.trim())?;
            ring.insert_verifying(id.clone(), vk);
            if let (Some(prev), Some(until)) =
                (e.previous_public_key_hex.as_deref(), e.previous_grace_until.as_deref())
            {
                let until_dt = parse_rfc3339(until)?;
                if now <= until_dt {
                    let prev_vk = parse_public_hex(prev.trim())?;
                    ring.add_verifying(id, prev_vk);
                }
            }
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

    /// Identity ids with an active dual-key grace at `now` (CRL rotate grace **or**
    /// same-id `previous_grace_until` on an entry).
    pub fn grace_active_ids(&self, now_rfc3339: &str) -> Result<HashSet<String>, CryptoError> {
        let now = parse_rfc3339(now_rfc3339)?;
        let mut out = HashSet::new();
        for r in &self.revoked {
            if r.grace_active_at(now)? {
                out.insert(r.identity_id.clone());
            }
        }
        for e in &self.entries {
            if let (Some(_), Some(until)) =
                (e.previous_public_key_hex.as_deref(), e.previous_grace_until.as_deref())
            {
                let until_dt = parse_rfc3339(until)?;
                if now <= until_dt {
                    out.insert(e.identity_id.clone());
                }
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
/// - Active trust entries (incl. same-id previous_* grace) and CRL entries with
///   active `grace_until` stay verifiable.
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
                guard.verifying.insert(id, vec![sk.verifying_key()]);
                continue;
            }
            guard.verifying.remove(&id);
        }
    }

    let n = register_trust_store(root.as_ref())?;
    // Re-apply node signing + dual-key grace from identity JSON.
    let _ = register_node_identity(root.as_ref())?;
    Ok(n)
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

/// Relative path (under node `identity/`) for opt-in previous signing secret backup.
pub const NODE_SECRET_BACKUP_FILE: &str = "local.ed25519.prev";
/// Sidecar metadata for [`NODE_SECRET_BACKUP_FILE`] (pubkey + timestamp; never the secret).
pub const NODE_SECRET_BACKUP_META_FILE: &str = "local.ed25519.prev.meta.json";

const NODE_SECRET_BACKUP_TMP: &str = "local.ed25519.prev.tmp";
const NODE_SECRET_BACKUP_META_TMP: &str = "local.ed25519.prev.meta.json.tmp";

/// One listed node signing-secret backup (latest slot or archived timestamped slot).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSecretBackupInfo {
    /// `latest` or compact UTC stamp (`YYYYMMDDTHHMMSSZ`[+`-N`]).
    pub stamp: String,
    pub secret_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backed_up_at: Option<String>,
    /// True when this is the canonical [`NODE_SECRET_BACKUP_FILE`] slot.
    pub is_latest: bool,
}

fn compact_utc_stamp(rfc3339: &str) -> Result<String, CryptoError> {
    let dt = parse_rfc3339(rfc3339)?;
    Ok(format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        dt.year(),
        u8::from(dt.month()),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ))
}

fn backup_stamp_from_meta(meta_path: &Path) -> Result<String, CryptoError> {
    if meta_path.is_file() {
        let raw = fs::read_to_string(meta_path).map_err(|e| CryptoError::Io(e.to_string()))?;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(s) = v.get("backed_up_at").and_then(|x| x.as_str()) {
                if let Ok(stamp) = compact_utc_stamp(s) {
                    return Ok(stamp);
                }
            }
        }
    }
    compact_utc_stamp(&utc_now_rfc3339()?)
}

fn unique_archived_backup_stamp(identity_dir: &Path, base: &str) -> String {
    let mut stamp = base.to_string();
    let mut n = 2u32;
    loop {
        let candidate = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}"));
        if !candidate.exists() {
            return stamp;
        }
        stamp = format!("{base}-{n}");
        n += 1;
    }
}

/// Move the canonical `.prev` slot into a timestamped archive name (Analyze-41).
///
/// No-op when the latest slot is missing. On I/O failure returns `Err` without
/// deleting the latest slot.
fn archive_latest_prev_slot(identity_dir: &Path) -> Result<Option<PathBuf>, CryptoError> {
    let latest = identity_dir.join(NODE_SECRET_BACKUP_FILE);
    if !latest.is_file() {
        return Ok(None);
    }
    let meta = identity_dir.join(NODE_SECRET_BACKUP_META_FILE);
    let base = backup_stamp_from_meta(&meta)?;
    let stamp = unique_archived_backup_stamp(identity_dir, &base);
    let archived = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}"));
    let archived_meta = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}.meta.json"));

    fs::rename(&latest, &archived).map_err(|e| {
        CryptoError::Io(format!(
            "archive prev rename failed ({} → {}): {e}",
            latest.display(),
            archived.display()
        ))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&archived, fs::Permissions::from_mode(0o600));
    }

    if meta.is_file() {
        let raw = fs::read_to_string(&meta).map_err(|e| CryptoError::Io(e.to_string()))?;
        let mut v: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}));
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "secret_path".into(),
                serde_json::json!(format!("identity/{NODE_SECRET_BACKUP_FILE}.{stamp}")),
            );
            if let Ok(now) = utc_now_rfc3339() {
                obj.insert("archived_at".into(), serde_json::json!(now));
            }
            obj.insert("archive_stamp".into(), serde_json::json!(stamp));
        }
        let out = serde_json::to_string_pretty(&v).map_err(|e| CryptoError::Io(e.to_string()))?;
        fs::write(&archived_meta, format!("{out}\n")).map_err(|e| CryptoError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&archived_meta, fs::Permissions::from_mode(0o600));
        }
        let _ = fs::remove_file(&meta);
    }

    Ok(Some(archived))
}

fn read_backup_meta_fields(meta_path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let Ok(raw) = fs::read_to_string(meta_path) else {
        return (None, None, None);
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (None, None, None);
    };
    (
        v.get("identity_id")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        v.get("old_public_key_hex")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        v.get("backed_up_at")
            .and_then(|x| x.as_str())
            .map(str::to_string),
    )
}

/// List durable node signing-secret backups (latest + archived timestamped slots).
///
/// Newest first. Does not read or return secret material.
pub fn list_node_secret_backups(
    root: impl AsRef<Path>,
) -> Result<Vec<NodeSecretBackupInfo>, CryptoError> {
    let identity_dir = root.as_ref().join("identity");
    if !identity_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let latest = identity_dir.join(NODE_SECRET_BACKUP_FILE);
    if latest.is_file() {
        let meta = identity_dir.join(NODE_SECRET_BACKUP_META_FILE);
        let (identity_id, old_public_key_hex, backed_up_at) = if meta.is_file() {
            read_backup_meta_fields(&meta)
        } else {
            (None, None, None)
        };
        out.push(NodeSecretBackupInfo {
            stamp: "latest".into(),
            secret_path: latest,
            meta_path: meta.is_file().then_some(meta),
            identity_id,
            old_public_key_hex,
            backed_up_at,
            is_latest: true,
        });
    }

    let prefix = format!("{NODE_SECRET_BACKUP_FILE}.");
    let rd = fs::read_dir(&identity_dir).map_err(|e| CryptoError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| CryptoError::Io(e.to_string()))?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(&prefix) {
            continue;
        }
        if name.ends_with(".meta.json") || name.ends_with(".tmp") || name == NODE_SECRET_BACKUP_META_FILE
        {
            continue;
        }
        // Archived secret: local.ed25519.prev.<stamp>
        let stamp = name[prefix.len()..].to_string();
        if stamp.is_empty() || stamp.contains('.') {
            continue;
        }
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        let meta = identity_dir.join(format!("{NODE_SECRET_BACKUP_FILE}.{stamp}.meta.json"));
        let (identity_id, old_public_key_hex, backed_up_at) = if meta.is_file() {
            read_backup_meta_fields(&meta)
        } else {
            (None, None, None)
        };
        out.push(NodeSecretBackupInfo {
            stamp,
            secret_path: path,
            meta_path: meta.is_file().then_some(meta),
            identity_id,
            old_public_key_hex,
            backed_up_at,
            is_latest: false,
        });
    }

    out.sort_by(|a, b| {
        // latest first, then stamp descending (lexicographic works for compact UTC).
        match (a.is_latest, b.is_latest) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.stamp.cmp(&a.stamp),
        }
    });
    Ok(out)
}

fn remove_path_quiet(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(path);
}

fn clear_staging_files(tmp: &Path, meta_tmp: &Path) {
    // Only regular leftover files from a crashed attempt — keep a directory trap so write fails.
    let _ = fs::remove_file(tmp);
    let _ = fs::remove_file(meta_tmp);
}

/// Rotate the node signing secret under fixed paths, keeping the same `identity_id`.
///
/// Rewrites `identity/local.ed25519` and updates `identity/local.identity.json` public key +
/// descriptor signature. Trust store gets an upsert (no CRL).
///
/// - Without `grace_until`: immediate cutover — previous verifying key for this id is dropped.
/// - With `grace_until` (RFC3339 UTC): persists `previous_public_key` + `previous_grace_until`
///   so old signatures under the same `key_ref` still verify until that instant (Analyze-37).
///
/// If trust upsert fails after the files were rewritten, previous secret + JSON are restored
/// so disk and trust stay consistent.
///
/// When `backup` is true, stages the previous secret under `*.tmp` (unix mode `0600`) **before**
/// overwrite and renames to `identity/local.ed25519.prev` (+ meta sidecar) only after a successful
/// rotate. If a prior `.prev` already exists, it is archived to
/// `local.ed25519.prev.<YYYYMMDDTHHMMSSZ>` (+ matching meta) before the new latest slot is committed
/// (Analyze-41). Staging/I/O failure aborts without changing the active secret; abort after staging
/// removes only tmp files (existing `.prev` / history slots are left intact).
///
/// Returns `(identity_id, new_public_key_hex, old_public_key_hex, backup_path)`.
pub fn rotate_node_signing_secret(
    root: impl AsRef<Path>,
    new_signing: SigningKey,
    backup: bool,
    grace_until: Option<&str>,
) -> Result<(AiraRef, String, String, Option<PathBuf>), CryptoError> {
    let root = root.as_ref();
    let identity_dir = root.join("identity");
    let json_path = identity_dir.join("local.identity.json");
    let key_path = identity_dir.join("local.ed25519");
    let backup_path = identity_dir.join(NODE_SECRET_BACKUP_FILE);
    let backup_meta_path = identity_dir.join(NODE_SECRET_BACKUP_META_FILE);
    let backup_tmp = identity_dir.join(NODE_SECRET_BACKUP_TMP);
    let backup_meta_tmp = identity_dir.join(NODE_SECRET_BACKUP_META_TMP);
    if !json_path.exists() {
        return Err(CryptoError::Io(format!(
            "missing {} — run `aira identity create` first",
            json_path.display()
        )));
    }
    // Fail closed if current material is inconsistent before overwrite.
    let (id, old_ring) = Keyring::load_node_identity(root)?;
    let old_json = fs::read_to_string(&json_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let old_secret = if key_path.exists() {
        Some(fs::read_to_string(&key_path).map_err(|e| CryptoError::Io(e.to_string()))?)
    } else {
        None
    };
    let mut desc: serde_json::Value =
        serde_json::from_str(&old_json).map_err(|e| CryptoError::Io(e.to_string()))?;
    let old_pub = desc
        .get("public_key")
        .and_then(|p| p.get("key_hex"))
        .and_then(|v| v.as_str())
        .ok_or(CryptoError::InvalidKey)?
        .trim()
        .to_string();

    let grace_until = match grace_until {
        Some(s) => Some(normalize_rfc3339(s)?),
        None => None,
    };

    let new_pub = hex::encode(new_signing.verifying_key().to_bytes());
    let secret_hex = hex::encode(new_signing.to_bytes());
    fs::create_dir_all(&identity_dir).map_err(|e| CryptoError::Io(e.to_string()))?;

    let cleanup_staging = || {
        remove_path_quiet(&backup_tmp);
        remove_path_quiet(&backup_meta_tmp);
    };

    let mut staged_backup = false;
    if backup {
        let secret = old_secret.as_ref().ok_or_else(|| {
            CryptoError::Io("cannot backup: missing identity/local.ed25519 before rotate".into())
        })?;
        // Drop leftover staging *files* from a previous crash (not directory traps).
        clear_staging_files(&backup_tmp, &backup_meta_tmp);
        if let Err(e) = fs::write(&backup_tmp, secret) {
            cleanup_staging();
            return Err(CryptoError::Io(format!(
                "backup stage failed ({}): {e} — rotate aborted",
                backup_tmp.display()
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(&backup_tmp, fs::Permissions::from_mode(0o600)) {
                cleanup_staging();
                return Err(CryptoError::Io(format!(
                    "backup stage chmod failed ({}): {e} — rotate aborted",
                    backup_tmp.display()
                )));
            }
        }
        let meta = serde_json::json!({
            "identity_id": id.as_str(),
            "old_public_key_hex": old_pub,
            "backed_up_at": match utc_now_rfc3339() {
                Ok(t) => t,
                Err(e) => {
                    cleanup_staging();
                    return Err(e);
                }
            },
            "secret_path": format!("identity/{NODE_SECRET_BACKUP_FILE}"),
        });
        let meta_out = match serde_json::to_string_pretty(&meta) {
            Ok(s) => s,
            Err(e) => {
                cleanup_staging();
                return Err(CryptoError::Io(e.to_string()));
            }
        };
        if let Err(e) = fs::write(&backup_meta_tmp, format!("{meta_out}\n")) {
            cleanup_staging();
            return Err(CryptoError::Io(format!(
                "backup meta stage failed ({}): {e} — rotate aborted",
                backup_meta_tmp.display()
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(&backup_meta_tmp, fs::Permissions::from_mode(0o600))
            {
                cleanup_staging();
                return Err(CryptoError::Io(format!(
                    "backup meta chmod failed ({}): {e} — rotate aborted",
                    backup_meta_tmp.display()
                )));
            }
        }
        staged_backup = true;
    }

    let restore_previous = || -> Result<(), CryptoError> {
        if let Some(secret) = &old_secret {
            fs::write(&key_path, secret).map_err(|e| CryptoError::Io(e.to_string()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
            }
        }
        fs::write(&json_path, &old_json).map_err(|e| CryptoError::Io(e.to_string()))?;
        register_keyring(&old_ring);
        set_primary_signer(id.clone());
        Ok(())
    };

    let abort_after_stage = |err: CryptoError| -> CryptoError {
        cleanup_staging();
        err
    };

    if let Err(e) = fs::write(&key_path, format!("{secret_hex}\n")) {
        return Err(abort_after_stage(CryptoError::Io(e.to_string())));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
    }

    let identity_id = id.as_str().to_string();
    let sig = sign_with_key(id.clone(), &new_signing, identity_id.as_bytes());
    if let Some(obj) = desc.as_object_mut() {
        obj.insert(
            "public_key".into(),
            serde_json::json!({
                "algorithm": "ed25519",
                "key_hex": new_pub
            }),
        );
        obj.insert(
            "signature".into(),
            serde_json::to_value(&sig).map_err(|e| {
                let _ = restore_previous();
                abort_after_stage(CryptoError::Io(e.to_string()))
            })?,
        );
        obj.insert(
            "key_path".into(),
            serde_json::json!("identity/local.ed25519"),
        );
        let rotated_at = match utc_now_rfc3339() {
            Ok(t) => t,
            Err(e) => {
                let _ = restore_previous();
                return Err(abort_after_stage(e));
            }
        };
        obj.insert("rotated_at".into(), serde_json::json!(rotated_at));
        if let Some(until) = grace_until.as_deref() {
            obj.insert(
                "previous_public_key".into(),
                serde_json::json!({
                    "algorithm": "ed25519",
                    "key_hex": old_pub
                }),
            );
            obj.insert("previous_grace_until".into(), serde_json::json!(until));
        } else {
            obj.remove("previous_public_key");
            obj.remove("previous_grace_until");
        }
    } else {
        let _ = restore_previous();
        return Err(abort_after_stage(CryptoError::InvalidKey));
    }
    let out = match serde_json::to_string_pretty(&desc) {
        Ok(s) => s,
        Err(e) => {
            let _ = restore_previous();
            return Err(abort_after_stage(CryptoError::Io(e.to_string())));
        }
    };
    if let Err(e) = fs::write(&json_path, format!("{out}\n")) {
        let _ = restore_previous();
        return Err(abort_after_stage(CryptoError::Io(e.to_string())));
    }

    if let Err(e) = ensure_trust_defaults(root) {
        let _ = restore_previous();
        return Err(abort_after_stage(e));
    }

    let mut wrote_backup: Option<PathBuf> = None;
    if staged_backup {
        // Destination must be a replaceable file path (not a directory trap).
        if backup_path.is_dir() {
            remove_path_quiet(&backup_path);
        }
        // Archive prior latest into timestamped history before committing the new latest.
        // On archive failure: leave staging tmp + existing `.prev` (never destroy history).
        let archive_ok = match archive_latest_prev_slot(&identity_dir) {
            Ok(_) => true,
            Err(_) => {
                wrote_backup = Some(backup_tmp.clone());
                false
            }
        };
        if archive_ok {
            match fs::rename(&backup_tmp, &backup_path) {
                Ok(()) => {
                    if backup_meta_path.is_dir() {
                        remove_path_quiet(&backup_meta_path);
                    }
                    if fs::rename(&backup_meta_tmp, &backup_meta_path).is_err() {
                        let _ = fs::copy(&backup_meta_tmp, &backup_meta_path);
                        remove_path_quiet(&backup_meta_tmp);
                    }
                    wrote_backup = Some(backup_path);
                }
                Err(_) => {
                    // Crypto + trust already committed — never restore_previous here.
                    // Leave staging tmp so the previous secret remains recoverable.
                    wrote_backup = Some(backup_tmp);
                }
            }
        }
    }

    // Reload so dual-key grace (if any) is registered for the same key_ref.
    let (id, ring) = Keyring::load_node_identity(root)?;
    register_keyring(&ring);
    set_primary_signer(id.clone());

    // Durable ceremony audit (pubkey only — never the secret).
    let audit = crate::audit::TrustAuditEntry::new(
        crate::audit::TrustAuditAction::NodeRotate,
        id.as_str(),
        Some("node-rotate"),
    )?
    .with_pubkey_hex(Some(new_pub.as_str()))
    .with_grace_until(grace_until.as_deref())
    .with_reason(Some("node signing secret rotated"));
    crate::audit::TrustAuditLog::append(root, &audit)?;

    Ok((id, new_pub, old_pub, wrote_backup))
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

    #[test]
    fn trust_rekey_grace_allows_old_same_id() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[41u8; 32]);
        let new_sk = SigningKey::from_bytes(&[43u8; 32]);
        let id = "aira:identity:peer-rekey-grace";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let new_pub = hex::encode(new_sk.verifying_key().to_bytes());

        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(id, &old_pub).unwrap();
        store.save(root).unwrap();

        let msg = b"same-id-grace";
        let old_sig = sign_with_key(AiraRef::parse(id).unwrap(), &old_sk, msg);
        let new_sig = sign_with_key(AiraRef::parse(id).unwrap(), &new_sk, msg);

        store
            .rekey(id, &new_pub, Some("2099-06-01T00:00:00Z"))
            .unwrap();
        store.save(root).unwrap();

        let entry = store
            .entries
            .iter()
            .find(|e| e.identity_id == id)
            .unwrap();
        assert_eq!(entry.public_key_hex, new_pub);
        assert_eq!(entry.previous_public_key_hex.as_deref(), Some(old_pub.as_str()));
        assert_eq!(
            entry.previous_grace_until.as_deref(),
            Some("2099-06-01T00:00:00Z")
        );

        let during = store.to_keyring_at("2026-07-28T12:00:00Z").unwrap();
        assert_eq!(during.verifying_keys(id).len(), 2);
        during.verify(&old_sig, msg).unwrap();
        during.verify(&new_sig, msg).unwrap();

        let after = store.to_keyring_at("2099-06-01T00:00:01Z").unwrap();
        assert_eq!(after.verifying_keys(id).len(), 1);
        assert!(after.verify(&old_sig, msg).is_err());
        after.verify(&new_sig, msg).unwrap();

        // Immediate cutover clears previous_*.
        store.rekey(id, &old_pub, None).unwrap();
        let e2 = store.entries.iter().find(|e| e.identity_id == id).unwrap();
        assert_eq!(e2.public_key_hex, old_pub);
        assert!(e2.previous_public_key_hex.is_none());
        assert!(e2.previous_grace_until.is_none());
    }

    #[test]
    fn node_signing_secret_rotate_cutover() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[31u8; 32]);
        let new_sk = SigningKey::from_bytes(&[33u8; 32]);
        let id = "aira:identity:node-rotate";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        fs::write(
            root.join("identity/local.ed25519"),
            format!("{}\n", hex::encode(old_sk.to_bytes())),
        )
        .unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-rotate",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();

        let msg = b"node-rotate-message";
        let old_sig = sign_with_key(AiraRef::parse(id).unwrap(), &old_sk, msg);
        let (loaded, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded.as_str(), id);
        ring.verify(&old_sig, msg).unwrap();

        let (rotated_id, new_pub, reported_old, backup_path) =
            rotate_node_signing_secret(root, new_sk.clone(), false, None).unwrap();
        assert_eq!(rotated_id.as_str(), id);
        assert_eq!(reported_old, old_pub);
        assert!(backup_path.is_none());
        assert!(!root.join("identity").join(NODE_SECRET_BACKUP_FILE).exists());
        assert_eq!(new_pub, hex::encode(new_sk.verifying_key().to_bytes()));
        // File-backed cutover (process keyring is shared across parallel tests).
        let (reloaded, new_ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(reloaded.as_str(), id);
        let new_sig = new_ring.sign(&reloaded, msg).unwrap();
        new_ring.verify(&new_sig, msg).unwrap();
        assert!(new_ring.verify(&old_sig, msg).is_err());

        let store = TrustStore::load(root).unwrap();
        let entry = store
            .entries
            .iter()
            .find(|e| e.identity_id == id)
            .expect("node trust entry");
        assert_eq!(entry.public_key_hex, new_pub);
        assert!(!store.is_revoked(id));
        let audit = crate::audit::TrustAuditLog::load(root).unwrap();
        assert!(audit.iter().any(|e| {
            e.action == crate::audit::TrustAuditAction::NodeRotate
                && e.subject_id == id
                && e.public_key_hex.as_deref() == Some(new_pub.as_str())
                && e.source.as_deref() == Some("node-rotate")
        }));

        let desc_raw = fs::read_to_string(root.join("identity/local.identity.json")).unwrap();
        let desc: serde_json::Value = serde_json::from_str(&desc_raw).unwrap();
        assert_eq!(desc["identity_id"], id);
        assert_eq!(desc["display_name"], "node-rotate");
        assert!(desc.get("rotated_at").is_some());

        reset_primary_signer();
    }

    #[test]
    fn node_rotate_rolls_back_when_node_revoked() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[37u8; 32]);
        let new_sk = SigningKey::from_bytes(&[39u8; 32]);
        let id = "aira:identity:node-rollback";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
        fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
        let old_json = serde_json::json!({
            "identity_id": id,
            "identity_type": "local",
            "display_name": "node-rollback",
            "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
            "created_at": "2026-07-16T00:00:00Z",
            "key_path": "identity/local.ed25519"
        })
        .to_string();
        fs::write(root.join("identity/local.identity.json"), &old_json).unwrap();
        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(id, &old_pub).unwrap();
        store.revoke(id, Some("block rotate")).unwrap();
        store.save(root).unwrap();

        let err = rotate_node_signing_secret(root, new_sk, false, None).unwrap_err();
        assert_eq!(err, CryptoError::RevokedKey(id.into()));
        assert_eq!(
            fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
            old_secret
        );
        let restored: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join("identity/local.identity.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(restored["public_key"]["key_hex"], old_pub);
        assert!(restored.get("rotated_at").is_none());
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_requires_existing_identity() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let err =
            rotate_node_signing_secret(root, SigningKey::from_bytes(&[35u8; 32]), false, None)
                .unwrap_err();
        assert!(matches!(err, CryptoError::Io(_)));
    }

    #[test]
    fn node_rotate_backup_writes_prev() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[41u8; 32]);
        let new_sk = SigningKey::from_bytes(&[43u8; 32]);
        let id = "aira:identity:node-backup";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
        fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-backup",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();

        let (rotated_id, new_pub, reported_old, backup_path) =
            rotate_node_signing_secret(root, new_sk.clone(), true, None).unwrap();
        assert_eq!(rotated_id.as_str(), id);
        assert_eq!(reported_old, old_pub);
        let backup = backup_path.expect("backup path");
        assert_eq!(backup, root.join("identity").join(NODE_SECRET_BACKUP_FILE));
        assert_eq!(fs::read_to_string(&backup).unwrap(), old_secret);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&backup).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        let meta: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_META_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(meta["old_public_key_hex"], old_pub);
        assert_eq!(meta["identity_id"], id);
        assert_ne!(
            fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
            old_secret
        );
        assert_eq!(new_pub, hex::encode(new_sk.verifying_key().to_bytes()));
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_backup_archives_prior_slot() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let sk1 = SigningKey::from_bytes(&[61u8; 32]);
        let sk2 = SigningKey::from_bytes(&[62u8; 32]);
        let sk3 = SigningKey::from_bytes(&[63u8; 32]);
        let id = "aira:identity:node-backup-hist";
        let pub1 = hex::encode(sk1.verifying_key().to_bytes());
        let secret1 = format!("{}\n", hex::encode(sk1.to_bytes()));
        fs::write(root.join("identity/local.ed25519"), &secret1).unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-backup-hist",
                "public_key": { "algorithm": "ed25519", "key_hex": pub1 },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();

        rotate_node_signing_secret(root, sk2.clone(), true, None).unwrap();
        let first_prev = fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_FILE))
            .unwrap();
        assert_eq!(first_prev, secret1);

        let secret2 = format!("{}\n", hex::encode(sk2.to_bytes()));
        rotate_node_signing_secret(root, sk3.clone(), true, None).unwrap();
        let latest = fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap();
        assert_eq!(latest, secret2);
        assert_ne!(latest, secret1);

        let list = list_node_secret_backups(root).unwrap();
        assert!(list.iter().any(|b| b.is_latest));
        assert!(
            list.iter().any(|b| !b.is_latest),
            "expected archived timestamped backup"
        );
        let archived = list.iter().find(|b| !b.is_latest).unwrap();
        assert_eq!(fs::read_to_string(&archived.secret_path).unwrap(), secret1);
        assert_eq!(
            archived.old_public_key_hex.as_deref(),
            Some(pub1.as_str())
        );
        // Both secrets still recoverable.
        let secrets: Vec<_> = list
            .iter()
            .map(|b| fs::read_to_string(&b.secret_path).unwrap())
            .collect();
        assert!(secrets.iter().any(|s| s == &secret1));
        assert!(secrets.iter().any(|s| s == &secret2));
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_backup_fail_closed() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[45u8; 32]);
        let new_sk = SigningKey::from_bytes(&[47u8; 32]);
        let id = "aira:identity:node-backup-fail";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
        fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-backup-fail",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        // Make backup staging path a directory so stage write fails.
        fs::create_dir_all(root.join("identity").join("local.ed25519.prev.tmp")).unwrap();

        let err = rotate_node_signing_secret(root, new_sk, true, None).unwrap_err();
        assert!(matches!(err, CryptoError::Io(_)));
        assert_eq!(
            fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
            old_secret
        );
        assert!(!root.join("identity").join(NODE_SECRET_BACKUP_FILE).exists());
        assert!(!root.join("identity/local.ed25519.prev.tmp").exists());
        let desc: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join("identity/local.identity.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(desc["public_key"]["key_hex"], old_pub);
        assert!(desc.get("rotated_at").is_none());
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_backup_preserves_prev_slot_on_trust_fail() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[49u8; 32]);
        let new_sk = SigningKey::from_bytes(&[51u8; 32]);
        let id = "aira:identity:node-backup-keep";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
        fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-backup-keep",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        let prior = b"prior-backup-secret\n";
        fs::write(root.join("identity").join(NODE_SECRET_BACKUP_FILE), prior).unwrap();
        let mut store = TrustStore::default();
        store.ensure_local_test().unwrap();
        store.upsert(id, &old_pub).unwrap();
        store.revoke(id, Some("block")).unwrap();
        store.save(root).unwrap();

        let err = rotate_node_signing_secret(root, new_sk, true, None).unwrap_err();
        assert_eq!(err, CryptoError::RevokedKey(id.into()));
        assert_eq!(
            fs::read_to_string(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap(),
            String::from_utf8_lossy(prior)
        );
        assert!(!root.join("identity/local.ed25519.prev.tmp").exists());
        assert_eq!(
            fs::read_to_string(root.join("identity/local.ed25519")).unwrap(),
            old_secret
        );
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_backup_commit_clears_prev_dir_trap() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[53u8; 32]);
        let new_sk = SigningKey::from_bytes(&[55u8; 32]);
        let id = "aira:identity:node-backup-dirtrap";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        let old_secret = format!("{}\n", hex::encode(old_sk.to_bytes()));
        fs::write(root.join("identity/local.ed25519"), &old_secret).unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-backup-dirtrap",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        fs::create_dir_all(root.join("identity").join(NODE_SECRET_BACKUP_FILE)).unwrap();

        let (rotated_id, new_pub, _, backup_path) =
            rotate_node_signing_secret(root, new_sk.clone(), true, None).unwrap();
        assert_eq!(rotated_id.as_str(), id);
        assert_eq!(new_pub, hex::encode(new_sk.verifying_key().to_bytes()));
        let backup = backup_path.expect("backup path");
        assert_eq!(backup, root.join("identity").join(NODE_SECRET_BACKUP_FILE));
        assert!(backup.is_file());
        assert_eq!(fs::read_to_string(&backup).unwrap(), old_secret);
        let store = TrustStore::load(root).unwrap();
        let entry = store.entries.iter().find(|e| e.identity_id == id).unwrap();
        assert_eq!(entry.public_key_hex, new_pub);
        let (loaded, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded.as_str(), id);
        let msg = b"dirtrap-ok";
        let sig = ring.sign(&loaded, msg).unwrap();
        ring.verify(&sig, msg).unwrap();
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_grace_allows_old_until() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[57u8; 32]);
        let new_sk = SigningKey::from_bytes(&[59u8; 32]);
        let id = "aira:identity:node-grace";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        fs::write(
            root.join("identity/local.ed25519"),
            format!("{}\n", hex::encode(old_sk.to_bytes())),
        )
        .unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-grace",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        let _ = ensure_trust_defaults(root).unwrap();

        let msg = b"node-grace-message";
        let old_sig = sign_with_key(AiraRef::parse(id).unwrap(), &old_sk, msg);
        let until = "2099-01-01T00:00:00Z";
        let (rotated_id, _new_pub, _, _) =
            rotate_node_signing_secret(root, new_sk.clone(), false, Some(until)).unwrap();
        assert_eq!(rotated_id.as_str(), id);

        let (reloaded, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(ring.verifying_keys(id).len(), 2);
        ring.verify(&old_sig, msg).unwrap();
        let new_sig = ring.sign(&reloaded, msg).unwrap();
        ring.verify(&new_sig, msg).unwrap();

        let desc_raw = fs::read_to_string(root.join("identity/local.identity.json")).unwrap();
        let desc: serde_json::Value = serde_json::from_str(&desc_raw).unwrap();
        assert_eq!(desc["previous_public_key"]["key_hex"], old_pub);
        assert_eq!(desc["previous_grace_until"], until);

        // Expired grace: rewrite until to the past and reload.
        let mut desc_obj = desc.as_object().unwrap().clone();
        desc_obj.insert(
            "previous_grace_until".into(),
            serde_json::json!("2020-01-01T00:00:00Z"),
        );
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::to_string_pretty(&desc_obj).unwrap(),
        )
        .unwrap();
        let (_, expired) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(expired.verifying_keys(id).len(), 1);
        assert!(expired.verify(&old_sig, msg).is_err());
        reset_primary_signer();
    }

    #[test]
    fn node_rotate_rejects_bad_grace_until() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        let old_sk = SigningKey::from_bytes(&[61u8; 32]);
        let new_sk = SigningKey::from_bytes(&[63u8; 32]);
        let id = "aira:identity:node-bad-grace";
        let old_pub = hex::encode(old_sk.verifying_key().to_bytes());
        fs::write(
            root.join("identity/local.ed25519"),
            format!("{}\n", hex::encode(old_sk.to_bytes())),
        )
        .unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "node-bad-grace",
                "public_key": { "algorithm": "ed25519", "key_hex": old_pub },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        let err = rotate_node_signing_secret(root, new_sk, false, Some("not-a-time")).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidTimestamp(_)));
        reset_primary_signer();
    }
}
