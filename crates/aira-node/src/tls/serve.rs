//! rustls ServerConfig + HTTPS serve (Analyze-85). No new TLS modes.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use rustls::server::danger::ClientCertVerifier;
use rustls::server::WebPkiClientVerifier;
use rustls::ServerConfig;

use super::pem::{load_certs, load_client_ca_roots, load_private_key};
use super::verifier::TrustMappedClientVerifier;

/// Ensure rustls crypto provider is installed (ring; no aws-lc).
pub fn install_crypto_provider() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("rustls crypto provider already installed or unavailable"))
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
