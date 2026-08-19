//! Optional HTTPS helpers for `aira-node --http` (Analyze-45/51/55 mTLS + CN→TrustStore).
//!
//! Mechanical split (Analyze-85 / QUEUE #50).

mod paths;
mod pem;
mod serve;
mod verifier;

pub use paths::resolve_tls_paths;
pub use pem::load_client_ca_roots;
pub use serve::serve_https;

#[cfg(test)]
mod tests {
    use super::paths::ensure_self_signed;
    use super::pem::load_certs;
    use super::serve::{build_server_config, install_crypto_provider};
    use super::verifier::assert_cn_in_trust_store;
    use super::*;
    use aira_flow::init_node;
    use aira_object::TrustStore;
    use anyhow::Result;
    use axum_server::tls_rustls::RustlsConfig;
    use ed25519_dalek::SigningKey;
    use rcgen::{BasicConstraints, DnType, IsCa, KeyUsagePurpose};
    use rcgen::{CertificateParams, KeyPair, SanType};
    use rustls::pki_types::PrivateKeyDer;
    use rustls::pki_types::ServerName;
    use rustls::ClientConfig;
    use rustls::{RootCertStore, ServerConfig};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
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
