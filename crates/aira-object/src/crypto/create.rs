//! Fail-closed node identity create / ensure (#316 / RFC-0201).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::types::AiraRef;

use super::error::{utc_now_rfc3339, CryptoError};
use super::keyring::{register_keyring, set_primary_signer, sign_with_key, Keyring};
use super::trust_store::ensure_trust_defaults;

const IDENTITY_DIR: &str = "identity";
const IDENTITY_KEY: &str = "local.ed25519";
const IDENTITY_JSON: &str = "local.identity.json";
const KEY_PATH_REL: &str = "identity/local.ed25519";

/// Policy for writing `identity/local.ed25519` + `local.identity.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdentityCreatePolicy {
    /// CLI `identity create`: any existing key or descriptor → error; never overwrite.
    CreateExclusive,
    /// Desktop ensure: complete pair → no-op Ok; empty → create; partial → error.
    Ensure,
}

/// Outcome of [`create_or_ensure_node_identity`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedNodeIdentity {
    pub identity_id: AiraRef,
    pub public_key_hex: String,
    pub identity_json_path: PathBuf,
    pub identity_key_path: PathBuf,
    /// `false` when [`NodeIdentityCreatePolicy::Ensure`] found a complete pair.
    pub wrote_new: bool,
}

/// Create a local node identity pair, or ensure one already exists (Desktop).
///
/// Authority (#316): validate identity ref + signed descriptor **before** any write.
/// Writes use `create_new` so concurrent create cannot silently overwrite.
pub fn create_or_ensure_node_identity(
    root: impl AsRef<Path>,
    identity_id: &str,
    display_name: &str,
    signing: SigningKey,
    policy: NodeIdentityCreatePolicy,
) -> Result<CreatedNodeIdentity, CryptoError> {
    let root = root.as_ref();
    let identity_dir = root.join(IDENTITY_DIR);
    let key_path = identity_dir.join(IDENTITY_KEY);
    let json_path = identity_dir.join(IDENTITY_JSON);
    let has_json = json_path.is_file();
    let has_key = key_path.is_file();

    match (has_json, has_key, policy) {
        (true, true, NodeIdentityCreatePolicy::Ensure) => {
            let id = load_existing_identity_id(&json_path)?;
            let public_key_hex = load_existing_public_hex(&json_path)?;
            return Ok(CreatedNodeIdentity {
                identity_id: id,
                public_key_hex,
                identity_json_path: json_path,
                identity_key_path: key_path,
                wrote_new: false,
            });
        }
        (true, true, NodeIdentityCreatePolicy::CreateExclusive) => {
            return Err(CryptoError::IdentityAlreadyExists);
        }
        (false, false, _) => {}
        (true, false, _) => {
            return Err(CryptoError::IdentityIncomplete(
                "local.identity.json present without local.ed25519 (#316)".into(),
            ));
        }
        (false, true, _) => {
            return Err(CryptoError::IdentityIncomplete(
                "local.ed25519 present without local.identity.json (#316)".into(),
            ));
        }
    }

    let id_ref = AiraRef::parse(identity_id).map_err(|e| CryptoError::Io(e.to_string()))?;
    let verifying: VerifyingKey = signing.verifying_key();
    let public_hex = hex::encode(verifying.to_bytes());
    let secret_hex = hex::encode(signing.to_bytes());
    let sig = sign_with_key(id_ref.clone(), &signing, identity_id.as_bytes());
    // Validate signature binds identity_id before touching disk (local ring; not yet registered).
    let mut probe = Keyring::with_local_test();
    probe.insert_signing(id_ref.clone(), signing.clone());
    probe.verify(&sig, identity_id.as_bytes())?;

    let created_at = utc_now_rfc3339().unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let desc = serde_json::json!({
        "identity_id": identity_id,
        "identity_type": "local",
        "display_name": display_name,
        "public_key": {
            "algorithm": "ed25519",
            "key_hex": public_hex
        },
        "created_at": created_at,
        "key_path": KEY_PATH_REL,
        "signature": sig
    });
    let desc_json =
        serde_json::to_string_pretty(&desc).map_err(|e| CryptoError::Io(e.to_string()))?;

    fs::create_dir_all(&identity_dir).map_err(|e| CryptoError::Io(e.to_string()))?;
    write_secret_create_new(&key_path, &format!("{secret_hex}\n"))?;
    if let Err(e) = write_json_create_new(&json_path, &desc_json) {
        // Best-effort rollback of secret so we do not leave an incomplete pair.
        let _ = fs::remove_file(&key_path);
        return Err(e);
    }

    let mut ring = Keyring::with_local_test();
    ring.insert_signing(id_ref.clone(), signing);
    register_keyring(&ring);
    set_primary_signer(id_ref.clone());
    let _ = ensure_trust_defaults(root);

    Ok(CreatedNodeIdentity {
        identity_id: id_ref,
        public_key_hex: public_hex,
        identity_json_path: json_path,
        identity_key_path: key_path,
        wrote_new: true,
    })
}

fn load_existing_identity_id(json_path: &Path) -> Result<AiraRef, CryptoError> {
    let text = fs::read_to_string(json_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let desc: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| CryptoError::Io(e.to_string()))?;
    let id = desc
        .get("identity_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CryptoError::Io("identity descriptor missing identity_id".into()))?;
    AiraRef::parse(id).map_err(|e| CryptoError::Io(e.to_string()))
}

fn load_existing_public_hex(json_path: &Path) -> Result<String, CryptoError> {
    let text = fs::read_to_string(json_path).map_err(|e| CryptoError::Io(e.to_string()))?;
    let desc: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| CryptoError::Io(e.to_string()))?;
    desc.get("public_key")
        .and_then(|pk| pk.get("key_hex"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| CryptoError::Io("identity descriptor missing public_key.key_hex".into()))
}

fn write_secret_create_new(path: &Path, contents: &str) -> Result<(), CryptoError> {
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts
        .open(path)
        .map_err(|e| CryptoError::Io(format!("create identity secret {}: {e}", path.display())))?;
    f.write_all(contents.as_bytes())
        .map_err(|e| CryptoError::Io(format!("write identity secret {}: {e}", path.display())))?;
    f.sync_all()
        .map_err(|e| CryptoError::Io(format!("sync identity secret {}: {e}", path.display())))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            CryptoError::Io(format!(
                "chmod 0600 identity secret {}: {e}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

fn write_json_create_new(path: &Path, contents: &str) -> Result<(), CryptoError> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            CryptoError::Io(format!(
                "create identity descriptor {}: {e}",
                path.display()
            ))
        })?;
    f.write_all(contents.as_bytes()).map_err(|e| {
        CryptoError::Io(format!("write identity descriptor {}: {e}", path.display()))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn mint(root: &Path, name: &str, policy: NodeIdentityCreatePolicy) -> CreatedNodeIdentity {
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        create_or_ensure_node_identity(
            root,
            &format!("aira:identity:{name}"),
            name,
            signing,
            policy,
        )
        .unwrap()
    }

    #[test]
    fn exclusive_create_then_second_create_fails_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let first = mint(root, "alice", NodeIdentityCreatePolicy::CreateExclusive);
        assert!(first.wrote_new);
        let key_before = fs::read(root.join("identity/local.ed25519")).unwrap();
        let json_before = fs::read(root.join("identity/local.identity.json")).unwrap();

        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let err = create_or_ensure_node_identity(
            root,
            "aira:identity:alice",
            "alice",
            signing,
            NodeIdentityCreatePolicy::CreateExclusive,
        )
        .unwrap_err();
        assert_eq!(err, CryptoError::IdentityAlreadyExists);
        assert_eq!(
            fs::read(root.join("identity/local.ed25519")).unwrap(),
            key_before
        );
        assert_eq!(
            fs::read(root.join("identity/local.identity.json")).unwrap(),
            json_before
        );
    }

    #[test]
    fn ensure_is_noop_on_complete_pair() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let first = mint(root, "desktop.one", NodeIdentityCreatePolicy::Ensure);
        assert!(first.wrote_new);
        let key_before = fs::read(root.join("identity/local.ed25519")).unwrap();
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let again = create_or_ensure_node_identity(
            root,
            "aira:identity:desktop.should-not-write",
            "desktop",
            signing,
            NodeIdentityCreatePolicy::Ensure,
        )
        .unwrap();
        assert!(!again.wrote_new);
        assert_eq!(again.identity_id, first.identity_id);
        assert_eq!(
            fs::read(root.join("identity/local.ed25519")).unwrap(),
            key_before
        );
    }

    #[test]
    fn incomplete_pair_rejected_without_mint() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("identity")).unwrap();
        fs::write(
            root.join("identity/local.identity.json"),
            r#"{"identity_id":"aira:identity:orphan"}"#,
        )
        .unwrap();
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let err = create_or_ensure_node_identity(
            root,
            "aira:identity:new",
            "new",
            signing,
            NodeIdentityCreatePolicy::CreateExclusive,
        )
        .unwrap_err();
        assert!(matches!(err, CryptoError::IdentityIncomplete(_)));
        assert!(!root.join("identity/local.ed25519").exists());
    }
}
