//! Client-cert CN → TrustStore mapping (Analyze-55 / Analyze-85).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use aira_object::{AiraRef, TrustStore};
use rustls::pki_types::{CertificateDer, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{
    CertificateError, DigitallySignedStruct, DistinguishedName, Error as RustlsError,
    SignatureScheme,
};
use x509_parser::prelude::*;

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
pub(super) struct TrustMappedClientVerifier {
    pub(super) inner: Arc<dyn ClientCertVerifier>,
    pub(super) node_root: PathBuf,
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
