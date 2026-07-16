//! Local AIRA-AP Artifact Protocol adapter (Issue #73).

use aira_artifact::{ArtifactDescriptor, ArtifactStore, CasArtifactStore, PublishResult};
use aira_object::{AiraRef, ContentHash};

use crate::envelope::{
    local_identity, local_signature, mvp_timestamp, ProtocolEnvelope, ProtocolError, ProtocolId,
    ProtocolResponse, ProtocolStatus, ScopeDescriptor,
};

/// Supported local artifact protocol version.
pub const AP_VERSION: &str = "0.1";

/// Outcome of an artifact protocol operation.
#[derive(Debug)]
pub struct ArtifactOpResult {
    pub envelope: ProtocolEnvelope,
    pub response: ProtocolResponse,
    pub published: Option<PublishResult>,
    pub resolved: Option<(ArtifactDescriptor, Vec<u8>)>,
}

/// Local AIRA-AP adapter.
pub struct ArtifactProtocolAdapter {
    store: CasArtifactStore,
    seq: u64,
}

impl ArtifactProtocolAdapter {
    pub fn open(root: impl AsRef<std::path::Path>) -> Result<Self, ProtocolError> {
        let store =
            CasArtifactStore::open(root).map_err(|e| ProtocolError::Storage(e.to_string()))?;
        Ok(Self { store, seq: 1 })
    }

    pub fn store(&self) -> &CasArtifactStore {
        &self.store
    }

    /// Publish artifact via protocol (hash validation enforced).
    pub fn publish(
        &mut self,
        descriptor: ArtifactDescriptor,
        payload: &[u8],
        protocol_version: &str,
    ) -> Result<ArtifactOpResult, ProtocolError> {
        if protocol_version != AP_VERSION {
            let envelope = self.wrap(
                "ArtifactPublish",
                &descriptor.artifact_id,
                &descriptor.content_hash,
                protocol_version,
            )?;
            let response = self.response(
                &descriptor.artifact_id,
                ProtocolStatus::UnsupportedVersion,
                Some(envelope.message_id.as_str()),
            )?;
            return Ok(ArtifactOpResult {
                envelope,
                response,
                published: None,
                resolved: None,
            });
        }

        let actual = ContentHash::sha256_bytes(payload);
        if actual != descriptor.content_hash {
            let envelope = self.wrap(
                "ArtifactPublish",
                &descriptor.artifact_id,
                &descriptor.content_hash,
                protocol_version,
            )?;
            let response = self.response(
                &descriptor.artifact_id,
                ProtocolStatus::InvalidArtifact,
                Some(envelope.message_id.as_str()),
            )?;
            return Ok(ArtifactOpResult {
                envelope,
                response,
                published: None,
                resolved: None,
            });
        }

        let published = self
            .store
            .publish(descriptor.clone(), payload)
            .map_err(|e| ProtocolError::InvalidArtifact(e.to_string()))?;
        let envelope = self.wrap(
            "ArtifactPublish",
            &published.descriptor.artifact_id,
            &published.descriptor.content_hash,
            protocol_version,
        )?;
        envelope.validate_signature()?;
        let response = self.response(
            &published.descriptor.artifact_id,
            ProtocolStatus::Accepted,
            Some(envelope.message_id.as_str()),
        )?;
        Ok(ArtifactOpResult {
            envelope,
            response,
            published: Some(published),
            resolved: None,
        })
    }

    /// Resolve artifact by id.
    pub fn resolve(
        &mut self,
        artifact_id: &AiraRef,
        protocol_version: &str,
    ) -> Result<ArtifactOpResult, ProtocolError> {
        if protocol_version != AP_VERSION {
            let placeholder = ContentHash::parse(
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
            let envelope = self.wrap(
                "ArtifactResolve",
                artifact_id,
                &placeholder,
                protocol_version,
            )?;
            let response = self.response(
                artifact_id,
                ProtocolStatus::UnsupportedVersion,
                Some(envelope.message_id.as_str()),
            )?;
            return Ok(ArtifactOpResult {
                envelope,
                response,
                published: None,
                resolved: None,
            });
        }
        let (desc, bytes) = self
            .store
            .resolve(artifact_id)
            .map_err(|e| ProtocolError::NotFound(e.to_string()))?;
        let envelope = self.wrap(
            "ArtifactResolve",
            artifact_id,
            &desc.content_hash,
            protocol_version,
        )?;
        let response = self.response(
            artifact_id,
            ProtocolStatus::Accepted,
            Some(envelope.message_id.as_str()),
        )?;
        Ok(ArtifactOpResult {
            envelope,
            response,
            published: None,
            resolved: Some((desc, bytes)),
        })
    }

    fn wrap(
        &mut self,
        message_type: &str,
        subject: &AiraRef,
        hash: &ContentHash,
        protocol_version: &str,
    ) -> Result<ProtocolEnvelope, ProtocolError> {
        self.seq += 1;
        let message_id = AiraRef::parse(format!("aira:message:ap{}", self.seq))
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
        Ok(ProtocolEnvelope {
            protocol_id: ProtocolId::Artifact,
            protocol_version: protocol_version.into(),
            message_type: message_type.into(),
            message_id,
            correlation_id: Some(subject.as_str().to_string()),
            causal_refs: vec![],
            issuer_identity: local_identity(),
            target_scope: ScopeDescriptor::local("artifact-protocol"),
            policy_refs: vec![AiraRef::parse("aira:policy:default")
                .map_err(|e| ProtocolError::Storage(e.to_string()))?],
            payload_hash: hash.clone(),
            payload_ref: Some(format!("artifact:{}", subject)),
            created_at: mvp_timestamp(),
            expires_at: None,
            signature: local_signature(),
        })
    }

    fn response(
        &mut self,
        subject: &AiraRef,
        status: ProtocolStatus,
        correlation: Option<&str>,
    ) -> Result<ProtocolResponse, ProtocolError> {
        self.seq += 1;
        let message_id = AiraRef::parse(format!("aira:message:apresp{}", self.seq))
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
        Ok(ProtocolResponse {
            message_id,
            correlation_id: correlation.map(|s| s.to_string()),
            status,
            reason_refs: vec![subject.clone()],
            created_at: mvp_timestamp(),
            signature: local_signature(),
        })
    }
}
