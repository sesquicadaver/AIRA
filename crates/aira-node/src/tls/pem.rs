//! PEM load helpers (Analyze-85).

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::RootCertStore;

/// Load PEM certificates from a file into DER list.
pub(super) fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let raw = fs::read(path).with_context(|| format!("read cert {}", path.display()))?;
    let certs = rustls_pemfile::certs(&mut raw.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("parse cert PEM {}", path.display()))?;
    if certs.is_empty() {
        bail!("no certificates in {}", path.display());
    }
    Ok(certs)
}

/// Load a single private key from PEM.
pub(super) fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let raw = fs::read(path).with_context(|| format!("read key {}", path.display()))?;
    let mut keys: Vec<PrivateKeyDer<'static>> = rustls_pemfile::read_all(&mut raw.as_slice())
        .filter_map(|item| match item.ok()? {
            rustls_pemfile::Item::Pkcs8Key(k) => Some(PrivateKeyDer::Pkcs8(k)),
            rustls_pemfile::Item::Pkcs1Key(k) => Some(PrivateKeyDer::Pkcs1(k)),
            rustls_pemfile::Item::Sec1Key(k) => Some(PrivateKeyDer::Sec1(k)),
            _ => None,
        })
        .collect();
    if keys.len() != 1 {
        bail!(
            "expected exactly one private key in {} (got {})",
            path.display(),
            keys.len()
        );
    }
    Ok(keys.pop().unwrap())
}

/// Load CA PEM into a [`RootCertStore`] (fail closed if empty/invalid).
pub fn load_client_ca_roots(ca_path: &Path) -> Result<RootCertStore> {
    if !ca_path.exists() {
        bail!("--tls-client-ca not found: {}", ca_path.display());
    }
    let certs = load_certs(ca_path)?;
    let mut roots = RootCertStore::empty();
    let (added, _) = roots.add_parsable_certificates(certs);
    if added == 0 || roots.is_empty() {
        bail!(
            "--tls-client-ca contains no usable trust anchors: {}",
            ca_path.display()
        );
    }
    Ok(roots)
}
