//! First-run init + identity + HTTP token material.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use aira_flow::{init_node, node_config_present, NodePaths};

use crate::paths::DesktopPaths;
use crate::settings::{resolve_token_path, write_settings, DesktopSettings, HttpAuthMode};

/// Ensure node layout, local identity, and bearer token (when required).
pub fn ensure_bootstrap(paths: &DesktopPaths, settings: &mut DesktopSettings) -> Result<()> {
    paths.ensure_dirs().context("desktop dirs")?;
    if !node_config_present(&paths.data_root) {
        let _ = init_node(&paths.data_root).map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    ensure_local_identity(&paths.data_root)?;
    if settings.http_auth_mode == HttpAuthMode::BearerToken {
        let token_path = resolve_token_path(paths, settings)?;
        ensure_http_token(&token_path)?;
        let abs = token_path
            .canonicalize()
            .unwrap_or(token_path)
            .display()
            .to_string();
        if settings.http_token_ref.as_deref() != Some(abs.as_str()) {
            settings.http_token_ref = Some(abs);
            write_settings(paths, settings)?;
        }
    }
    Ok(())
}

/// Legacy fixed Desktop identity id (pre-#276). Existing installs keep this value on disk.
pub const LEGACY_DESKTOP_IDENTITY_ID: &str = "aira:identity:desktop";

/// Allocate a new install-scoped Desktop identity id (`aira:identity:desktop.<uuid>`).
pub fn new_desktop_identity_id() -> String {
    format!("aira:identity:desktop.{}", uuid::Uuid::now_v7().simple())
}

/// Read `identity_id` from an existing local identity descriptor (if present).
pub fn read_local_identity_id(root: &Path) -> Result<Option<String>> {
    let np = NodePaths::new(root);
    if !np.identity_json().is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(np.identity_json())
        .with_context(|| format!("read {}", np.identity_json().display()))?;
    let desc: serde_json::Value =
        serde_json::from_str(&text).context("parse local.identity.json")?;
    Ok(desc
        .get("identity_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

fn ensure_local_identity(root: &Path) -> Result<()> {
    let np = NodePaths::new(root);
    let has_json = np.identity_json().is_file();
    let has_key = np.identity_key().is_file();
    match (has_json, has_key) {
        (true, true) => return Ok(()),
        (false, false) => {}
        (true, false) => anyhow::bail!(
            "identity incomplete: local.identity.json present without local.ed25519 (#298)"
        ),
        (false, true) => anyhow::bail!(
            "identity incomplete: local.ed25519 present without local.identity.json (#298)"
        ),
    }

    let mut rng = OsRng;
    let signing = SigningKey::generate(&mut rng);
    let verifying: VerifyingKey = signing.verifying_key();
    let secret_hex = hex::encode(signing.to_bytes());
    let public_hex = hex::encode(verifying.to_bytes());
    fs::create_dir_all(np.identity_dir())?;

    // Mint only on empty pair: refuse overwrite / concurrent create (#298).
    write_secret_create_new(&np.identity_key(), &format!("{secret_hex}\n"))?;

    // Install-scoped unique ID; display_name stays "desktop" (#276).
    // Existing roots that already have identity files keep their ID (no silent migration).
    let identity_id = new_desktop_identity_id();
    let id_ref = aira_object::AiraRef::parse(&identity_id).map_err(|e| anyhow::anyhow!("{e}"))?;
    let sig = aira_object::sign_with_key(id_ref.clone(), &signing, identity_id.as_bytes());
    let created_at =
        aira_object::utc_now_rfc3339().unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let desc = serde_json::json!({
        "identity_id": identity_id,
        "identity_type": "local",
        "display_name": "desktop",
        "public_key": {
            "algorithm": "ed25519",
            "key_hex": public_hex
        },
        "created_at": created_at,
        "key_path": "identity/local.ed25519",
        "signature": sig
    });
    write_json_create_new(&np.identity_json(), &serde_json::to_string_pretty(&desc)?)?;

    let mut ring = aira_object::Keyring::with_local_test();
    ring.insert_signing(id_ref.clone(), signing);
    aira_object::register_keyring(&ring);
    aira_object::set_primary_signer(id_ref);
    let _ = aira_object::ensure_trust_defaults(root);
    Ok(())
}

/// Create secret file only if absent; Unix mode 0o600 is required (fail-closed).
fn write_secret_create_new(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts
        .open(path)
        .with_context(|| format!("create identity secret {}", path.display()))?;
    f.write_all(contents.as_bytes())
        .with_context(|| format!("write identity secret {}", path.display()))?;
    f.sync_all()
        .with_context(|| format!("sync identity secret {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 0600 identity secret {}", path.display()))?;
    }
    Ok(())
}

fn write_json_create_new(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create identity descriptor {}", path.display()))?;
    f.write_all(contents.as_bytes())
        .with_context(|| format!("write identity descriptor {}", path.display()))?;
    Ok(())
}

fn ensure_http_token(path: &Path) -> Result<()> {
    if path.is_file() {
        let meta = fs::metadata(path)?;
        if meta.len() > 0 {
            return Ok(());
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut bytes);
    let token = hex::encode(bytes);
    fs::write(path, format!("{token}\n")).with_context(|| format!("write {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Read bearer token from disk (trimmed).
pub fn read_http_token(path: &Path) -> Result<String> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read token {}", path.display()))?;
    let t = text.trim();
    if t.is_empty() {
        anyhow::bail!("empty http token at {}", path.display());
    }
    Ok(t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::DesktopPaths;
    use crate::settings::load_or_create_settings;

    #[test]
    fn new_desktop_identity_id_is_unique_and_parseable() {
        let a = new_desktop_identity_id();
        let b = new_desktop_identity_id();
        assert_ne!(a, b);
        assert!(a.starts_with("aira:identity:desktop."));
        assert_ne!(a, LEGACY_DESKTOP_IDENTITY_ID);
        aira_object::AiraRef::parse(&a).unwrap();
        aira_object::AiraRef::parse(&b).unwrap();
    }

    #[test]
    fn two_roots_get_distinct_identity_ids_via_ensure_bootstrap() {
        let tmp = tempfile::tempdir().unwrap();
        let a_paths = DesktopPaths::for_data_root(tmp.path().join("a"));
        let b_paths = DesktopPaths::for_data_root(tmp.path().join("b"));
        a_paths.ensure_dirs().unwrap();
        b_paths.ensure_dirs().unwrap();
        let mut a_settings = load_or_create_settings(&a_paths).unwrap();
        let mut b_settings = load_or_create_settings(&b_paths).unwrap();
        ensure_bootstrap(&a_paths, &mut a_settings).unwrap();
        ensure_bootstrap(&b_paths, &mut b_settings).unwrap();
        let id_a = read_local_identity_id(&a_paths.data_root).unwrap().unwrap();
        let id_b = read_local_identity_id(&b_paths.data_root).unwrap().unwrap();
        assert_ne!(id_a, id_b, "two installs must not share identity_id");
        assert_ne!(id_a, LEGACY_DESKTOP_IDENTITY_ID);
        assert_ne!(id_b, LEGACY_DESKTOP_IDENTITY_ID);
        ensure_bootstrap(&a_paths, &mut a_settings).unwrap();
        assert_eq!(
            read_local_identity_id(&a_paths.data_root).unwrap().unwrap(),
            id_a
        );
    }

    #[test]
    fn legacy_identity_file_is_not_silently_migrated() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path().join("legacy"));
        paths.ensure_dirs().unwrap();
        let np = NodePaths::new(&paths.data_root);
        fs::create_dir_all(np.identity_dir()).unwrap();
        fs::write(
            np.identity_key(),
            format!(
                "{}
",
                "00".repeat(32)
            ),
        )
        .unwrap();
        let desc = serde_json::json!({
            "identity_id": LEGACY_DESKTOP_IDENTITY_ID,
            "identity_type": "local",
            "display_name": "desktop",
            "public_key": {"algorithm": "ed25519", "key_hex": "11".repeat(32)},
            "created_at": "2026-01-01T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": {
                "algorithm": "ed25519",
                "key_ref": LEGACY_DESKTOP_IDENTITY_ID,
                "sig_hex": "22".repeat(64)
            }
        });
        fs::write(
            np.identity_json(),
            serde_json::to_string_pretty(&desc).unwrap(),
        )
        .unwrap();
        let mut settings = load_or_create_settings(&paths).unwrap();
        ensure_bootstrap(&paths, &mut settings).unwrap();
        assert_eq!(
            read_local_identity_id(&paths.data_root).unwrap().as_deref(),
            Some(LEGACY_DESKTOP_IDENTITY_ID)
        );
    }

    #[test]
    fn incomplete_identity_pair_is_fail_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path().join("partial"));
        paths.ensure_dirs().unwrap();
        let np = NodePaths::new(&paths.data_root);
        fs::create_dir_all(np.identity_dir()).unwrap();
        let mut settings = load_or_create_settings(&paths).unwrap();

        // Descriptor without secret → reject; must not mint a new pair.
        let orphan_desc = serde_json::json!({
            "identity_id": "aira:identity:desktop.partial-json",
            "identity_type": "local",
            "display_name": "desktop",
            "public_key": {"algorithm": "ed25519", "key_hex": "11".repeat(32)},
            "created_at": "2026-01-01T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": {
                "algorithm": "ed25519",
                "key_ref": "aira:identity:desktop.partial-json",
                "signature_value": "22".repeat(64)
            }
        });
        fs::write(
            np.identity_json(),
            serde_json::to_string_pretty(&orphan_desc).unwrap(),
        )
        .unwrap();
        let err = ensure_bootstrap(&paths, &mut settings).unwrap_err();
        assert!(
            err.to_string().contains("#298") && err.to_string().contains("incomplete"),
            "{err}"
        );
        assert!(
            !np.identity_key().is_file(),
            "must not mint secret over orphan json"
        );
        assert_eq!(
            read_local_identity_id(&paths.data_root).unwrap().as_deref(),
            Some("aira:identity:desktop.partial-json")
        );

        // Secret without descriptor → reject; must not mint a new descriptor/id.
        fs::remove_file(np.identity_json()).unwrap();
        fs::write(np.identity_key(), format!("{}\n", "33".repeat(32))).unwrap();
        let err = ensure_bootstrap(&paths, &mut settings).unwrap_err();
        assert!(
            err.to_string().contains("#298") && err.to_string().contains("incomplete"),
            "{err}"
        );
        assert!(
            !np.identity_json().is_file(),
            "must not mint descriptor over orphan secret"
        );
        let key_before = fs::read_to_string(np.identity_key()).unwrap();
        assert_eq!(key_before.trim(), "33".repeat(32));
    }

    #[test]
    #[cfg(unix)]
    fn minted_identity_secret_is_owner_rw_only() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path().join("perms"));
        paths.ensure_dirs().unwrap();
        let mut settings = load_or_create_settings(&paths).unwrap();
        ensure_bootstrap(&paths, &mut settings).unwrap();
        let np = NodePaths::new(&paths.data_root);
        let mode = fs::metadata(np.identity_key())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "identity secret must be 0600, got {mode:o}");
    }
}
