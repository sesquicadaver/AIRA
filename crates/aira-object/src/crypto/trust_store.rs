//! Durable trust.json store (Analyze-82). CRL / rotate / rekey live here (EVO revocation).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::AiraRef;

use super::error::{
    normalize_rfc3339, parse_public_hex, parse_rfc3339, utc_now_rfc3339, CryptoError,
    LOCAL_TEST_KEY_REF,
};
use super::keyring::{
    process_keyring, register_keyring, register_node_identity, Keyring, NodeIdentityFile,
};

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
    /// Refuses [`LOCAL_TEST_KEY_REF`] — fixture identity must not enter runtime trust (SEC-1).
    pub fn upsert(&mut self, identity_id: &str, public_key_hex: &str) -> Result<(), CryptoError> {
        let _ = parse_public_hex(public_key_hex.trim())?;
        let id = identity_id.trim();
        if id == LOCAL_TEST_KEY_REF {
            return Err(CryptoError::ProtectedIdentity(LOCAL_TEST_KEY_REF.into()));
        }
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

    /// Strip legacy `local-test` entry from runtime trust (SEC-1 migration).
    ///
    /// Returns `true` when an entry was removed.
    pub fn strip_local_test(&mut self) -> bool {
        self.remove(LOCAL_TEST_KEY_REF)
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
            if let (Some(prev), Some(until)) = (
                e.previous_public_key_hex.as_deref(),
                e.previous_grace_until.as_deref(),
            ) {
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
            if let (Some(_), Some(until)) = (
                e.previous_public_key_hex.as_deref(),
                e.previous_grace_until.as_deref(),
            ) {
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
        let tenant_pubs: HashSet<String> =
            crate::tenant::tenant_publisher_ids().into_iter().collect();
        let ids: Vec<String> = guard.verifying.keys().cloned().collect();
        for id in ids {
            if id == LOCAL_TEST_KEY_REF
                || trusted.contains(&id)
                || grace.contains(&id)
                || tenant_pubs.contains(&id)
            {
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

/// Ensure node identity (when present) is in trust.json; strip legacy local-test (SEC-1).
pub fn ensure_trust_defaults(root: impl AsRef<Path>) -> Result<TrustStore, CryptoError> {
    let root = root.as_ref();
    let mut store = TrustStore::load(root)?;
    store.strip_local_test();
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
