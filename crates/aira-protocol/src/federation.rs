//! Local federation join prototype (Analyze-70 / QUEUE #35).
//!
//! Operator ceremony: self-signed descriptor → TrustStore pin + membership file.
//! Not Book II Join Request/Response. Other federation members stay Untrusted.

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{utc_now_rfc3339, AiraRef, Keyring, Signature, TrustStore, LOCAL_TEST_KEY_REF};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain tag and descriptor `schema` value.
pub const FEDERATION_DESCRIPTOR_DOMAIN: &str = "aira:federation:descriptor:v1";
/// On-disk membership schema.
pub const FEDERATION_MEMBERSHIP_SCHEMA: &str = "aira:federation:membership:v1";

/// Join / descriptor errors.
#[derive(Debug, Error)]
pub enum FederationError {
    #[error("{0}")]
    Failed(String),
    #[error(transparent)]
    Crypto(#[from] aira_object::CryptoError),
}

/// Self-signed federation descriptor (Book II §14.2 subset + pubkey for TOFU).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationDescriptor {
    pub schema: String,
    pub federation_id: String,
    pub federation_type: String,
    pub identity_ref: String,
    pub public_key_hex: String,
    pub signature: Signature,
}

/// Durable local membership (one per node until leave).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationMembership {
    pub schema: String,
    pub federation_id: String,
    pub federation_type: String,
    pub identity_ref: String,
    pub public_key_hex: String,
    pub joined_at: String,
}

/// Result of [`join_federation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinOutcome {
    pub membership: FederationMembership,
    pub already_member: bool,
}

/// Path to `.aira/federation/membership.json`.
pub fn membership_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("federation").join("membership.json")
}

/// Canonical bytes for Ed25519 (signature field excluded).
pub fn descriptor_canonical_bytes(d: &FederationDescriptor) -> Vec<u8> {
    format!(
        "{FEDERATION_DESCRIPTOR_DOMAIN}|{}|{}|{}|{}|{}",
        d.schema, d.federation_id, d.federation_type, d.identity_ref, d.public_key_hex
    )
    .into_bytes()
}

/// Verify self-signature against the embedded pubkey (not the process keyring).
pub fn verify_federation_descriptor(d: &FederationDescriptor) -> Result<(), FederationError> {
    if d.schema != FEDERATION_DESCRIPTOR_DOMAIN {
        return Err(FederationError::Failed(format!(
            "descriptor schema mismatch: {}",
            d.schema
        )));
    }
    if d.federation_type.trim().is_empty() {
        return Err(FederationError::Failed(
            "federation_type must be non-empty".into(),
        ));
    }
    let fed_id = AiraRef::parse(&d.federation_id)
        .map_err(|e| FederationError::Failed(format!("federation_id: {e}")))?;
    if !fed_id.as_str().starts_with("aira:federation:") {
        return Err(FederationError::Failed(format!(
            "federation_id must be aira:federation:…, got {fed_id}"
        )));
    }
    let id = AiraRef::parse(&d.identity_ref)
        .map_err(|e| FederationError::Failed(format!("identity_ref: {e}")))?;
    if id.as_str() != d.signature.key_ref.as_str() {
        return Err(FederationError::Failed(
            "signature.key_ref must equal identity_ref".into(),
        ));
    }
    if id.as_str() == LOCAL_TEST_KEY_REF {
        return Err(FederationError::Failed(
            "refusing aira:identity:local-test as federation identity".into(),
        ));
    }
    let ring = Keyring::with_verifying_hex(&id, &d.public_key_hex)?;
    ring.verify(&d.signature, &descriptor_canonical_bytes(d))?;
    Ok(())
}

fn load_membership(
    root: impl AsRef<Path>,
) -> Result<Option<FederationMembership>, FederationError> {
    let path = membership_path(&root);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| FederationError::Failed(e.to_string()))?;
    let m: FederationMembership =
        serde_json::from_str(&raw).map_err(|e| FederationError::Failed(e.to_string()))?;
    if m.schema != FEDERATION_MEMBERSHIP_SCHEMA {
        return Err(FederationError::Failed(format!(
            "membership schema mismatch: {}",
            m.schema
        )));
    }
    Ok(Some(m))
}

fn save_membership(
    root: impl AsRef<Path>,
    m: &FederationMembership,
) -> Result<(), FederationError> {
    let path = membership_path(&root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| FederationError::Failed(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(m).map_err(|e| FederationError::Failed(e.to_string()))?;
    fs::write(&path, format!("{json}\n")).map_err(|e| FederationError::Failed(e.to_string()))?;
    Ok(())
}

fn hex_eq(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

fn already_pinned(store: &TrustStore, identity_ref: &str, public_key_hex: &str) -> bool {
    store
        .entries
        .iter()
        .any(|e| e.identity_id == identity_ref && hex_eq(&e.public_key_hex, public_key_hex))
}

/// Persist TrustStore only when the identity is not already pinned with this key.
fn pin_identity(
    root: &Path,
    store: &mut TrustStore,
    desc: &FederationDescriptor,
) -> Result<(), FederationError> {
    if already_pinned(store, &desc.identity_ref, &desc.public_key_hex) {
        aira_object::register_trust_store(root)?;
        return Ok(());
    }
    store.upsert(&desc.identity_ref, &desc.public_key_hex)?;
    store.save(root)?;
    aira_object::register_trust_store(root)?;
    Ok(())
}

/// Load local federation membership if present.
pub fn load_federation_membership(
    root: impl AsRef<Path>,
) -> Result<Option<FederationMembership>, FederationError> {
    load_membership(root)
}

/// Join locally: verify descriptor, pin `identity_ref` in TrustStore, write membership.
pub fn join_federation(
    root: impl AsRef<Path>,
    desc: &FederationDescriptor,
) -> Result<JoinOutcome, FederationError> {
    verify_federation_descriptor(desc)?;
    let root = root.as_ref();
    let mut store = TrustStore::load(root)?;
    if store.is_revoked(&desc.identity_ref) {
        return Err(FederationError::Failed(format!(
            "federation identity is revoked: {}",
            desc.identity_ref
        )));
    }
    if let Some(e) = store
        .entries
        .iter()
        .find(|e| e.identity_id == desc.identity_ref)
    {
        if !hex_eq(&e.public_key_hex, &desc.public_key_hex) {
            return Err(FederationError::Failed(
                "TrustStore already has a different pubkey for identity_ref".into(),
            ));
        }
    }

    let existing = load_membership(root)?;
    if let Some(m) = existing {
        if m.federation_id != desc.federation_id {
            return Err(FederationError::Failed(format!(
                "already joined {} — different federation_id refused (leave is out of this slice)",
                m.federation_id
            )));
        }
        if m.identity_ref != desc.identity_ref || !hex_eq(&m.public_key_hex, &desc.public_key_hex) {
            return Err(FederationError::Failed(
                "same federation_id with a different key is refused".into(),
            ));
        }
        if !store
            .entries
            .iter()
            .any(|e| e.identity_id == desc.identity_ref)
        {
            pin_identity(root, &mut store, desc)?;
        }
        return Ok(JoinOutcome {
            membership: m,
            already_member: true,
        });
    }

    pin_identity(root, &mut store, desc)?;
    let membership = FederationMembership {
        schema: FEDERATION_MEMBERSHIP_SCHEMA.into(),
        federation_id: desc.federation_id.clone(),
        federation_type: desc.federation_type.clone(),
        identity_ref: desc.identity_ref.clone(),
        public_key_hex: desc.public_key_hex.trim().to_ascii_lowercase(),
        joined_at: utc_now_rfc3339()?,
    };
    save_membership(root, &membership)?;
    Ok(JoinOutcome {
        membership,
        already_member: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    fn keypair(seed: u8) -> (SigningKey, String) {
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let pk = hex::encode(sk.verifying_key().to_bytes());
        (sk, pk)
    }

    fn unsigned(id: &str, fed: &str, pk: &str) -> FederationDescriptor {
        FederationDescriptor {
            schema: FEDERATION_DESCRIPTOR_DOMAIN.into(),
            federation_id: fed.into(),
            federation_type: "private".into(),
            identity_ref: id.into(),
            public_key_hex: pk.into(),
            signature: Signature {
                algorithm: "ed25519".into(),
                key_ref: AiraRef::parse(id).unwrap(),
                signature_value: String::new(),
            },
        }
    }

    fn signed(id: &str, fed: &str, sk: &SigningKey, pk: &str) -> FederationDescriptor {
        let mut d = unsigned(id, fed, pk);
        d.signature = aira_object::sign_with_key(
            AiraRef::parse(id).unwrap(),
            sk,
            &descriptor_canonical_bytes(&d),
        );
        d
    }

    #[test]
    fn join_pins_trust_and_membership() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (sk, pk) = keypair(3);
        let desc = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        let out = join_federation(root, &desc).unwrap();
        assert!(!out.already_member);
        let store = TrustStore::load(root).unwrap();
        assert!(store
            .entries
            .iter()
            .any(|e| e.identity_id == "aira:identity:fed-home"));
        assert!(membership_path(root).exists());
        let again = join_federation(root, &desc).unwrap();
        assert!(again.already_member);
        assert_eq!(again.membership.joined_at, out.membership.joined_at);
    }

    #[test]
    fn join_does_not_clobber_existing_same_pubkey_grace() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (sk, pk) = keypair(13);
        let desc = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        let mut store = TrustStore::default();
        store.upsert(&desc.identity_ref, &pk).unwrap();
        {
            let e = store
                .entries
                .iter_mut()
                .find(|e| e.identity_id == desc.identity_ref)
                .unwrap();
            e.previous_public_key_hex = Some("aa".repeat(32));
            e.previous_grace_until = Some("2099-01-01T00:00:00Z".into());
        }
        store.save(root).unwrap();
        join_federation(root, &desc).unwrap();
        let loaded = TrustStore::load(root).unwrap();
        let e = loaded
            .entries
            .iter()
            .find(|e| e.identity_id == desc.identity_ref)
            .unwrap();
        assert_eq!(
            e.previous_grace_until.as_deref(),
            Some("2099-01-01T00:00:00Z")
        );
        assert!(membership_path(root).exists());
    }

    #[test]
    fn unsigned_leaves_trust_untouched() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (_sk, pk) = keypair(4);
        let desc = unsigned("aira:identity:fed-home", "aira:federation:home", &pk);
        assert!(join_federation(root, &desc).is_err());
        assert!(!membership_path(root).exists());
        let store = TrustStore::load(root).unwrap();
        assert!(store.entries.is_empty());
    }

    #[test]
    fn key_ref_mismatch_fails() {
        let dir = tempdir().unwrap();
        let (sk, pk) = keypair(5);
        let mut desc = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        desc.signature.key_ref = AiraRef::parse("aira:identity:other").unwrap();
        assert!(join_federation(dir.path(), &desc).is_err());
        assert!(!membership_path(dir.path()).exists());
    }

    #[test]
    fn revoked_identity_fails() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (sk, pk) = keypair(6);
        let desc = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        let mut store = TrustStore::default();
        store.upsert(&desc.identity_ref, &pk).unwrap();
        store.revoke(&desc.identity_ref, Some("test")).unwrap();
        store.save(root).unwrap();
        assert!(join_federation(root, &desc).is_err());
        assert!(!membership_path(root).exists());
    }

    #[test]
    fn other_federation_id_fail_closed() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (sk, pk) = keypair(7);
        let a = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        join_federation(root, &a).unwrap();
        let (sk2, pk2) = keypair(8);
        let b = signed(
            "aira:identity:fed-other",
            "aira:federation:other",
            &sk2,
            &pk2,
        );
        let err = join_federation(root, &b).unwrap_err().to_string();
        assert!(err.contains("already joined"), "{err}");
        let m = load_membership(root).unwrap().unwrap();
        assert_eq!(m.federation_id, "aira:federation:home");
    }

    #[test]
    fn same_federation_different_key_fails() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (sk, pk) = keypair(9);
        let a = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        join_federation(root, &a).unwrap();
        let (sk2, pk2) = keypair(10);
        let b = signed("aira:identity:fed-home", "aira:federation:home", &sk2, &pk2);
        assert!(join_federation(root, &b).is_err());
    }

    #[test]
    fn truststore_pubkey_mismatch_fails() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (sk, pk) = keypair(11);
        let desc = signed("aira:identity:fed-home", "aira:federation:home", &sk, &pk);
        let mut store = TrustStore::default();
        let other_pk = hex::encode(
            SigningKey::from_bytes(&[12u8; 32])
                .verifying_key()
                .to_bytes(),
        );
        store.upsert(&desc.identity_ref, &other_pk).unwrap();
        store.save(root).unwrap();
        let err = join_federation(root, &desc).unwrap_err().to_string();
        assert!(err.contains("different pubkey"), "{err}");
        assert!(!membership_path(root).exists());
    }

    #[test]
    fn refuses_local_test_identity() {
        let dir = tempdir().unwrap();
        let sk = aira_object::local_test_signing_key();
        let pk = aira_object::local_test_public_key_hex();
        let desc = signed(LOCAL_TEST_KEY_REF, "aira:federation:home", &sk, &pk);
        let err = join_federation(dir.path(), &desc).unwrap_err().to_string();
        assert!(err.contains("local-test"), "{err}");
    }

    #[test]
    fn invalid_pubkey_hex_fails() {
        let dir = tempdir().unwrap();
        let desc = unsigned("aira:identity:fed-home", "aira:federation:home", "zz");
        assert!(join_federation(dir.path(), &desc).is_err());
        assert!(!membership_path(dir.path()).exists());
    }
}
