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

fn ensure_local_identity(root: &Path) -> Result<()> {
    let np = NodePaths::new(root);
    if np.identity_json().is_file() && np.identity_key().is_file() {
        return Ok(());
    }
    let mut rng = OsRng;
    let signing = SigningKey::generate(&mut rng);
    let verifying: VerifyingKey = signing.verifying_key();
    let secret_hex = hex::encode(signing.to_bytes());
    let public_hex = hex::encode(verifying.to_bytes());
    fs::create_dir_all(np.identity_dir())?;
    fs::write(np.identity_key(), format!("{secret_hex}\n"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(np.identity_key(), fs::Permissions::from_mode(0o600));
    }
    let identity_id = "aira:identity:desktop";
    let id_ref = aira_object::AiraRef::parse(identity_id).map_err(|e| anyhow::anyhow!("{e}"))?;
    let sig = aira_object::sign_with_key(id_ref.clone(), &signing, identity_id.as_bytes());
    let desc = serde_json::json!({
        "identity_id": identity_id,
        "identity_type": "local",
        "display_name": "desktop",
        "public_key": {
            "algorithm": "ed25519",
            "key_hex": public_hex
        },
        "created_at": "2026-08-20T00:00:00Z",
        "key_path": "identity/local.ed25519",
        "signature": sig
    });
    fs::write(np.identity_json(), serde_json::to_string_pretty(&desc)?)?;
    let mut ring = aira_object::Keyring::with_local_test();
    ring.insert_signing(id_ref.clone(), signing);
    aira_object::register_keyring(&ring);
    aira_object::set_primary_signer(id_ref);
    let _ = aira_object::ensure_trust_defaults(root);
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
