//! Self-signed PEM paths and CLI TLS path resolution (Analyze-85).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rcgen::{CertificateParams, KeyPair, SanType};

/// Paths to PEM certificate and private key under node root.
pub fn self_signed_paths(root: impl AsRef<Path>) -> (PathBuf, PathBuf) {
    let dir = root.as_ref().join("http");
    (dir.join("cert.pem"), dir.join("key.pem"))
}

/// Generate a loopback self-signed cert+key if missing; return paths.
pub fn ensure_self_signed(root: impl AsRef<Path>) -> Result<(PathBuf, PathBuf)> {
    let (cert_path, key_path) = self_signed_paths(&root);
    if cert_path.exists() && key_path.exists() {
        return Ok((cert_path, key_path));
    }
    if let Some(parent) = cert_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }

    let mut params =
        CertificateParams::new(vec!["localhost".into(), "127.0.0.1".into(), "::1".into()])
            .context("certificate params")?;
    params
        .subject_alt_names
        .push(SanType::IpAddress(std::net::IpAddr::V4(
            std::net::Ipv4Addr::LOCALHOST,
        )));

    let key_pair = KeyPair::generate().context("generate key")?;
    let cert = params
        .self_signed(&key_pair)
        .context("self-sign certificate")?;

    fs::write(&cert_path, cert.pem()).with_context(|| format!("write {}", cert_path.display()))?;
    fs::write(&key_path, key_pair.serialize_pem())
        .with_context(|| format!("write {}", key_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
    }
    Ok((cert_path, key_path))
}

/// Resolve TLS PEM paths from CLI flags.
pub fn resolve_tls_paths(
    root: &Path,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
    tls_self_signed: bool,
) -> Result<Option<(PathBuf, PathBuf)>> {
    match (tls_cert, tls_key, tls_self_signed) {
        (None, None, false) => Ok(None),
        (Some(c), Some(k), false) => {
            if !c.exists() {
                bail!("--tls-cert not found: {}", c.display());
            }
            if !k.exists() {
                bail!("--tls-key not found: {}", k.display());
            }
            Ok(Some((c, k)))
        }
        (None, None, true) => Ok(Some(ensure_self_signed(root)?)),
        (Some(_), Some(_), true) => {
            bail!("--tls-self-signed is mutually exclusive with --tls-cert/--tls-key")
        }
        (Some(_), None, _) | (None, Some(_), _) => {
            bail!("--tls-cert and --tls-key must be provided together")
        }
    }
}
