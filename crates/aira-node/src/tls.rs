//! Optional HTTPS helpers for `aira-node --http` (Analyze-45/51/55 mTLS + CN→TrustStore).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aira_object::{AiraRef, TrustStore};
use anyhow::{bail, Context, Result};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{CertificateParams, KeyPair, SanType};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::WebPkiClientVerifier;
use rustls::{
    CertificateError, DigitallySignedStruct, DistinguishedName, Error as RustlsError,
    RootCertStore, ServerConfig, SignatureScheme,
};
use x509_parser::prelude::*;

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

/// Extract subject Common Name from an EE certificate DER (Analyze-55).
pub fn client_cert_common_name(end_entity: &CertificateDer<'_>) -> Result<String, String> {
    let (_, cert) = X509Certificate::from_der(end_entity.as_ref())
        .map_err(|e| format!("client cert parse: {e}"))?;
    let cn = cert
        .subject()
        .iter_common_name()
        .next()
        .ok_or_else(|| "client cert missing Common Name".to_string())?;
    let cn = cn
        .as_str()
        .map_err(|_| "client cert CN is not valid UTF-8".to_string())?
        .trim();
    if cn.is_empty() {
        return Err("client cert CN empty".into());
    }
    Ok(cn.to_string())
}

/// Fail-closed: CN must be a trusted, non-revoked AiraRef in local TrustStore.
pub fn assert_cn_in_trust_store(node_root: &Path, cn: &str) -> Result<(), String> {
    let id = cn.trim();
    AiraRef::parse(id).map_err(|e| format!("client cert CN is not an AiraRef: {e}"))?;
    let store = TrustStore::load(node_root).map_err(|e| format!("trust store: {e}"))?;
    if store.is_revoked(id) {
        return Err(format!("client cert CN revoked: {id}"));
    }
    if !store.entries.iter().any(|e| e.identity_id == id) {
        return Err(format!("client cert CN not in TrustStore: {id}"));
    }
    Ok(())
}

/// WebPki CA check + TrustStore CN map (Analyze-55).
#[derive(Debug)]
struct TrustMappedClientVerifier {
    inner: Arc<dyn ClientCertVerifier>,
    node_root: PathBuf,
}

impl ClientCertVerifier for TrustMappedClientVerifier {
    fn offer_client_auth(&self) -> bool {
        self.inner.offer_client_auth()
    }

    fn client_auth_mandatory(&self) -> bool {
        self.inner.client_auth_mandatory()
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        self.inner.root_hint_subjects()
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, RustlsError> {
        let verified = self
            .inner
            .verify_client_cert(end_entity, intermediates, now)?;
        let cn = client_cert_common_name(end_entity).map_err(|_e| {
            RustlsError::InvalidCertificate(CertificateError::ApplicationVerificationFailure)
        })?;
        assert_cn_in_trust_store(&self.node_root, &cn).map_err(|_e| {
            RustlsError::InvalidCertificate(CertificateError::ApplicationVerificationFailure)
        })?;
        Ok(verified)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// Build a rustls [`ServerConfig`] for HTTPS, optionally requiring client certs + TrustStore CN.
///
/// When `client_ca` is set, `node_root` must be provided (Analyze-55 CN→TrustStore).
pub fn build_server_config(
    cert: &Path,
    key: &Path,
    client_ca: Option<&Path>,
    node_root: Option<&Path>,
) -> Result<ServerConfig> {
    install_crypto_provider().ok();
    let certs = load_certs(cert)?;
    let key = load_private_key(key)?;

    let builder = ServerConfig::builder();
    let mut config = if let Some(ca) = client_ca {
        let root = node_root
            .ok_or_else(|| anyhow::anyhow!("mTLS requires node root for TrustStore CN mapping"))?;
        let roots = Arc::new(load_client_ca_roots(ca)?);
        let inner = WebPkiClientVerifier::builder(roots)
            .build()
            .map_err(|e| anyhow::anyhow!("client cert verifier: {e}"))?;
        let verifier: Arc<dyn ClientCertVerifier> = Arc::new(TrustMappedClientVerifier {
            inner,
            node_root: root.to_path_buf(),
        });
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

/// Serve the axum router over HTTPS (server-only or mTLS + TrustStore CN).
pub async fn serve_https(
    addr: std::net::SocketAddr,
    app: Router,
    cert: &Path,
    key: &Path,
    client_ca: Option<&Path>,
    node_root: &Path,
) -> Result<()> {
    let server_config = build_server_config(cert, key, client_ca, Some(node_root))?;
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
    use aira_flow::init_node;
    use ed25519_dalek::SigningKey;
    use rcgen::{BasicConstraints, DnType, IsCa, KeyUsagePurpose};
    use rustls::pki_types::ServerName;
    use rustls::ClientConfig;
    use tempfile::tempdir;

    fn write_pair(dir: &Path, name: &str, cert_pem: &str, key_pem: &str) -> (PathBuf, PathBuf) {
        let cert = dir.join(format!("{name}.crt.pem"));
        let key = dir.join(format!("{name}.key.pem"));
        fs::write(&cert, cert_pem).unwrap();
        fs::write(&key, key_pem).unwrap();
        (cert, key)
    }

    fn client_identity_id() -> &'static str {
        "aira:identity:mtls-client-good"
    }

    /// CA + server EE + clients (trusted CN / unknown CN / wrong-CA).
    struct Fixture {
        _dir: tempfile::TempDir,
        node_root: PathBuf,
        server_cert: PathBuf,
        server_key: PathBuf,
        ca_path: PathBuf,
        client_cert_pem: String,
        client_key_pem: String,
        unknown_client_cert_pem: String,
        unknown_client_key_pem: String,
        wrong_client_cert_pem: String,
        wrong_client_key_pem: String,
    }

    fn set_cn(params: &mut CertificateParams, cn: &str) {
        params
            .distinguished_name
            .push(DnType::CommonName, cn.to_string());
    }

    fn fixture() -> Fixture {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        init_node(&root).unwrap();

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
        let srv_cert = srv_params.signed_by(&srv_key, &ca_cert, &ca_key).unwrap();

        let mut client_params = CertificateParams::default();
        set_cn(&mut client_params, client_identity_id());
        client_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        let client_key = KeyPair::generate().unwrap();
        let client_cert = client_params
            .signed_by(&client_key, &ca_cert, &ca_key)
            .unwrap();

        let mut unknown_params = CertificateParams::default();
        set_cn(&mut unknown_params, "aira:identity:mtls-unknown");
        unknown_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        let unknown_key = KeyPair::generate().unwrap();
        let unknown_cert = unknown_params
            .signed_by(&unknown_key, &ca_cert, &ca_key)
            .unwrap();

        let mut ca2_params = CertificateParams::new(vec!["Other CA".into()]).unwrap();
        ca2_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca2_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];
        let ca2_key = KeyPair::generate().unwrap();
        let ca2_cert = ca2_params.self_signed(&ca2_key).unwrap();
        let mut bad_params = CertificateParams::default();
        set_cn(&mut bad_params, client_identity_id());
        bad_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        let bad_key = KeyPair::generate().unwrap();
        let bad_cert = bad_params.signed_by(&bad_key, &ca2_cert, &ca2_key).unwrap();

        // TrustStore knows the good client identity (Ed25519 pubkey unrelated to TLS key).
        let sk = SigningKey::from_bytes(&[42u8; 32]);
        let pk = hex::encode(sk.verifying_key().to_bytes());
        let mut trust = TrustStore::load(&root).unwrap();
        trust.upsert(client_identity_id(), &pk).unwrap();
        trust.save(&root).unwrap();

        let (server_cert, server_key) =
            write_pair(&root, "server", &srv_cert.pem(), &srv_key.serialize_pem());
        let ca_path = root.join("ca.pem");
        fs::write(&ca_path, ca_cert.pem()).unwrap();

        Fixture {
            _dir: dir,
            node_root: root,
            server_cert,
            server_key,
            ca_path,
            client_cert_pem: client_cert.pem(),
            client_key_pem: client_key.serialize_pem(),
            unknown_client_cert_pem: unknown_cert.pem(),
            unknown_client_key_pem: unknown_key.serialize_pem(),
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
        let cfg = build_server_config(&cert, &key, None, None).unwrap();
        assert!(cfg.alpn_protocols.iter().any(|p| p == b"http/1.1"));
        let _ = RustlsConfig::from_config(Arc::new(cfg));
    }

    #[test]
    fn resolve_requires_pair() {
        let dir = tempdir().unwrap();
        let err =
            resolve_tls_paths(dir.path(), Some(dir.path().join("c.pem")), None, false).unwrap_err();
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
    fn mtls_accepts_trusted_cn() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(
                &f.server_cert,
                &f.server_key,
                Some(&f.ca_path),
                Some(&f.node_root),
            )
            .unwrap(),
        );
        let client = client_config(&f.ca_path, Some((&f.client_cert_pem, &f.client_key_pem)));
        handshake_mem(server, client).expect("valid mTLS + TrustStore CN");
    }

    #[test]
    fn mtls_rejects_unknown_truststore_cn() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(
                &f.server_cert,
                &f.server_key,
                Some(&f.ca_path),
                Some(&f.node_root),
            )
            .unwrap(),
        );
        let client = client_config(
            &f.ca_path,
            Some((&f.unknown_client_cert_pem, &f.unknown_client_key_pem)),
        );
        let err = handshake_mem(server, client).expect_err("must reject unknown CN");
        assert!(!err.is_empty(), "{err}");
    }

    #[test]
    fn mtls_rejects_revoked_truststore_cn() {
        let f = fixture();
        let mut trust = TrustStore::load(&f.node_root).unwrap();
        trust.revoke(client_identity_id(), Some("test")).unwrap();
        trust.save(&f.node_root).unwrap();

        let server = Arc::new(
            build_server_config(
                &f.server_cert,
                &f.server_key,
                Some(&f.ca_path),
                Some(&f.node_root),
            )
            .unwrap(),
        );
        let client = client_config(&f.ca_path, Some((&f.client_cert_pem, &f.client_key_pem)));
        let err = handshake_mem(server, client).expect_err("must reject revoked CN");
        assert!(!err.is_empty(), "{err}");
    }

    #[test]
    fn mtls_rejects_missing_client_cert() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(
                &f.server_cert,
                &f.server_key,
                Some(&f.ca_path),
                Some(&f.node_root),
            )
            .unwrap(),
        );
        let client = client_config(&f.ca_path, None);
        let err = handshake_mem(server, client).expect_err("must reject anonymous");
        assert!(!err.is_empty(), "{err}");
    }

    #[test]
    fn mtls_rejects_wrong_ca_client_cert() {
        let f = fixture();
        let server = Arc::new(
            build_server_config(
                &f.server_cert,
                &f.server_key,
                Some(&f.ca_path),
                Some(&f.node_root),
            )
            .unwrap(),
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
        let cfg = build_server_config(&f.server_cert, &f.server_key, None, None).unwrap();
        assert!(cfg.alpn_protocols.iter().any(|p| p == b"h2"));
        assert!(cfg.alpn_protocols.iter().any(|p| p == b"http/1.1"));
    }

    #[test]
    fn assert_cn_helpers() {
        let f = fixture();
        assert_cn_in_trust_store(&f.node_root, client_identity_id()).unwrap();
        assert!(assert_cn_in_trust_store(&f.node_root, "not-a-ref").is_err());
        assert!(assert_cn_in_trust_store(&f.node_root, "aira:identity:missing").is_err());
    }
}
