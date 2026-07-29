//! Optional HTTPS helpers for `aira-node --http` (Analyze-45 + Analyze-51 mTLS).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{CertificateParams, KeyPair, SanType};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};

/// Ensure rustls crypto provider is installed (ring; no aws-lc).
pub fn install_crypto_provider() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("rustls crypto provider already installed or unavailable"))
}

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

    let mut params = CertificateParams::new(vec![
        "localhost".into(),
        "127.0.0.1".into(),
        "::1".into(),
    ])
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

/// Load PEM certificates from a file into DER list.
fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
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
fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
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

/// Build a rustls [`ServerConfig`] for HTTPS, optionally requiring client certs (Analyze-51).
pub fn build_server_config(
    cert: &Path,
    key: &Path,
    client_ca: Option<&Path>,
) -> Result<ServerConfig> {
    install_crypto_provider().ok();
    let certs = load_certs(cert)?;
    let key = load_private_key(key)?;

    let builder = ServerConfig::builder();
    let mut config = if let Some(ca) = client_ca {
        let roots = Arc::new(load_client_ca_roots(ca)?);
        let verifier = WebPkiClientVerifier::builder(roots)
            .build()
            .map_err(|e| anyhow::anyhow!("client cert verifier: {e}"))?;
        builder
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs, key)
            .map_err(|e| anyhow::anyhow!("server cert: {e}"))?
    } else {
        builder
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| anyhow::anyhow!("server cert: {e}"))?
    };
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

/// Serve the axum router over HTTPS (server-only or mTLS).
pub async fn serve_https(
    addr: std::net::SocketAddr,
    app: Router,
    cert: &Path,
    key: &Path,
    client_ca: Option<&Path>,
) -> Result<()> {
    let server_config = build_server_config(cert, key, client_ca)?;
    let config = RustlsConfig::from_config(Arc::new(server_config));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .context("https serve")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::{BasicConstraints, IsCa, KeyUsagePurpose};
    use rustls::ClientConfig;
    use rustls::pki_types::ServerName;
    use tempfile::tempdir;

    fn write_pair(dir: &Path, name: &str, cert_pem: &str, key_pem: &str) -> (PathBuf, PathBuf) {
        let cert = dir.join(format!("{name}.crt.pem"));
        let key = dir.join(format!("{name}.key.pem"));
        fs::write(&cert, cert_pem).unwrap();
        fs::write(&key, key_pem).unwrap();
        (cert, key)
    }

    /// CA + server EE + two clients (good + wrong-CA).
    struct Fixture {
        _dir: tempfile::TempDir,
        server_cert: PathBuf,
        server_key: PathBuf,
        ca_path: PathBuf,
        client_cert_pem: String,
        client_key_pem: String,
        wrong_client_cert_pem: String,
        wrong_client_key_pem: String,
    }

    fn fixture() -> Fixture {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let mut ca_params = CertificateParams::new(vec!["AIRA Test CA".into()]).unwrap();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let ca_key = KeyPair::generate().unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();

        let mut srv_params = CertificateParams::new(vec!["localhost".into()]).unwrap();
        srv_params
            .subject_alt_names
            .push(SanType::IpAddress(std::net::IpAddr::V4(
                std::net::Ipv4Addr::LOCALHOST,
            )));
        let srv_key = KeyPair::generate().unwrap();
        let srv_cert = srv_params
            .signed_by(&srv_key, &ca_cert, &ca_key)
            .unwrap();

        let mut client_params = CertificateParams::new(vec!["client-good".into()]).unwrap();
        client_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        let client_key = KeyPair::generate().unwrap();
        let client_cert = client_params
            .signed_by(&client_key, &ca_cert, &ca_key)
            .unwrap();

        // Separate CA for wrong-client.
        let mut ca2_params = CertificateParams::new(vec!["Other CA".into()]).unwrap();
        ca2_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca2_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let ca2_key = KeyPair::generate().unwrap();
        let ca2_cert = ca2_params.self_signed(&ca2_key).unwrap();
        let mut bad_params = CertificateParams::new(vec!["client-bad".into()]).unwrap();
        bad_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        let bad_key = KeyPair::generate().unwrap();
        let bad_cert = bad_params.signed_by(&bad_key, &ca2_cert, &ca2_key).unwrap();

        let (server_cert, server_key) =
            write_pair(root, "server", &srv_cert.pem(), &srv_key.serialize_pem());
        let ca_path = root.join("ca.pem");
        fs::write(&ca_path, ca_cert.pem()).unwrap();

        Fixture {
            _dir: dir,
            server_cert,
            server_key,
            ca_path,
            client_cert_pem: client_cert.pem(),
            client_key_pem: client_key.serialize_pem(),
            wrong_client_cert_pem: bad_cert.pem(),
            wrong_client_key_pem: bad_key.serialize_pem(),
        }
    }

    /// In-memory rustls handshake (no sockets — avoids deadlock).
    fn handshake_mem(
        server_cfg: Arc<ServerConfig>,
        client_cfg: Arc<ClientConfig>,
    ) -> Result<(), String> {
        let name = ServerName::try_from("localhost").unwrap();
        let mut client = rustls::ClientConnection::new(client_cfg, name).unwrap();
        let mut server = rustls::ServerConnection::new(server_cfg).unwrap();
        let mut rounds = 0usize;
        while (client.is_handshaking() || server.is_handshaking()) && rounds < 128 {
            rounds += 1;
            while client.wants_write() {
                let mut buf = Vec::new();
                client.write_tls(&mut buf).map_err(|e| e.to_string())?;
                if buf.is_empty() {
                    break;
                }
                server
                    .read_tls(&mut buf.as_slice())
                    .map_err(|e| e.to_string())?;
                server.process_new_packets().map_err(|e| e.to_string())?;
            }
            while server.wants_write() {
                let mut buf = Vec::new();
                server.write_tls(&mut buf).map_err(|e| e.to_string())?;
                if buf.is_empty() {
                    break;
                }
                client
                    .read_tls(&mut buf.as_slice())
                    .map_err(|e| e.to_string())?;
                client.process_new_packets().map_err(|e| e.to_string())?;
            }
        }
        if client.is_handshaking() || server.is_handshaking() {
            return Err("handshake incomplete".into());
        }
        Ok(())
    }

    fn client_config(
        trust_server_ca: &Path,
        client_cert_pem: Option<(&str, &str)>,
    ) -> Arc<ClientConfig> {
        install_crypto_provider().ok();
        let mut roots = RootCertStore::empty();
        let certs = load_certs(trust_server_ca).unwrap();
        let (n, _) = roots.add_parsable_certificates(certs);
        assert!(n > 0);
        let builder = ClientConfig::builder().with_root_certificates(roots);
        let cfg = if let Some((cert_pem, key_pem)) = client_cert_pem {
            let certs: Vec<_> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let mut keys: Vec<PrivateKeyDer<'static>> =
                rustls_pemfile::read_all(&mut key_pem.as_bytes())
                    .filter_map(|item| match item.ok()? {
                        rustls_pemfile::Item::Pkcs8Key(k) => Some(PrivateKeyDer::Pkcs8(k)),
                        rustls_pemfile::Item::Pkcs1Key(k) => Some(PrivateKeyDer::Pkcs1(k)),
                        rustls_pemfile::Item::Sec1Key(k) => Some(PrivateKeyDer::Sec1(k)),
                        _ => None,
                    })
                    .collect();
            builder
                .with_client_auth_cert(certs, keys.pop().unwrap())
                .unwrap()
        } else {
            builder.with_no_client_auth()
        };
        Arc::new(cfg)
    }

    #[tokio::test]
    async fn self_signed_loads_into_rustls_config() {
        let dir = tempdir().unwrap();
        let (cert, key) = ensure_self_signed(dir.path()).unwrap();
        assert!(cert.exists());
        assert!(key.exists());
        let cfg = build_server_config(&cert, &key, None).unwrap();
        assert!(cfg.alpn_protocols.iter().any(|p| p == b"http/1.1"));
        let _ = RustlsConfig::from_config(Arc::new(cfg));
    }

    #[test]
    fn resolve_requires_pair() {
        let dir = tempdir().unwrap();
        let err = resolve_tls_paths(dir.path(), Some(dir.path().join("c.pem")), None, false)
            .unwrap_err();
        assert!(err.to_string().contains("together"));
    }

    #[test]
    fn client_ca_empty_fails_closed() {
        let dir = tempdir().unwrap();
        let empty = dir.path().join("empty.pem");
        fs::write(&empty, "").unwrap();
        let err = load_client_ca_roots(&empty).unwrap_err();
        assert!(
            err.to_string().contains("no usable") || err.to_string().contains("no certificates"),
            "{err}"
        );
    }

    #[test]
    fn mtls_accepts_valid_client_cert() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(&f.server_cert, &f.server_key, Some(&f.ca_path)).unwrap(),
        );
        let client = client_config(
            &f.ca_path,
            Some((&f.client_cert_pem, &f.client_key_pem)),
        );
        handshake_mem(server, client).expect("valid mTLS handshake");
    }

    #[test]
    fn mtls_rejects_missing_client_cert() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(&f.server_cert, &f.server_key, Some(&f.ca_path)).unwrap(),
        );
        let client = client_config(&f.ca_path, None);
        let err = handshake_mem(server, client).expect_err("must reject anonymous");
        assert!(!err.is_empty(), "{err}");
    }

    #[test]
    fn mtls_rejects_wrong_ca_client_cert() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(&f.server_cert, &f.server_key, Some(&f.ca_path)).unwrap(),
        );
        let client = client_config(
            &f.ca_path,
            Some((&f.wrong_client_cert_pem, &f.wrong_client_key_pem)),
        );
        let err = handshake_mem(server, client).expect_err("must reject wrong CA");
        assert!(!err.is_empty(), "{err}");
    }

    #[test]
    fn without_client_ca_alpn_still_set() {
        let f = fixture();
        let cfg = build_server_config(&f.server_cert, &f.server_key, None).unwrap();
        assert!(cfg.alpn_protocols.iter().any(|p| p == b"h2"));
        assert!(cfg.alpn_protocols.iter().any(|p| p == b"http/1.1"));
    }
}
