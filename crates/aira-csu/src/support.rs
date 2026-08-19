//! Shared builders for basic CSU implementations.

use aira_artifact::{ArtifactDescriptor, ArtifactType};
use aira_event::{EventDescriptor, EventType};
use aira_object::{
    active_identity, active_signature, AiraRef, ContentHash, CryptoError, Timestamp,
};
use serde_json::{json, Value};

use crate::manifest::{CsuManifest, CsuSandbox, CsuType, SUPPORTED_ABI_VERSION};

/// Signature over a message using the process primary signer (node identity when set).
pub fn local_signature_over(message: &[u8]) -> aira_object::Signature {
    active_signature(message)
}

/// Signature over the standard domain message using the primary signer.
pub fn local_signature() -> aira_object::Signature {
    active_signature(aira_object::LOCAL_TEST_DOMAIN_MSG)
}

/// Producer identity — primary signer (node identity when registered, else local-test).
pub fn local_identity() -> AiraRef {
    active_identity()
}

/// Fixed MVP timestamp.
pub fn mvp_timestamp() -> Timestamp {
    Timestamp::parse("2026-07-10T12:00:00Z").expect("ts")
}

/// Override `publisher_identity` on a manifest (emit signer; identity_ref unchanged).
///
/// For a non-primary / non-local-test publisher, also call
/// [`aira_object::register_csu_tenant_signing`] so emits can sign under tenant isolation.
pub fn apply_publisher(manifest: &mut CsuManifest, publisher: AiraRef) {
    manifest.publisher_identity = publisher;
    manifest
        .resign_canonical()
        .expect("canonical manifest after publisher override");
}

/// Build a minimal signed manifest for a basic CSU.
pub fn basic_manifest(
    csu_id: &str,
    csu_name: &str,
    csu_type: CsuType,
    subscriptions: &[&str],
    outputs: &[&str],
) -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(csu_id).expect("csu_id"),
        csu_name: csu_name.into(),
        csu_type,
        csu_version: "0.1.0".into(),
        abi_version: SUPPORTED_ABI_VERSION.into(),
        manifest_version: "0.1".into(),
        identity_ref: local_identity(),
        publisher_identity: local_identity(),
        capabilities: vec![],
        permissions: vec![],
        event_subscriptions: subscriptions
            .iter()
            .map(|t| json!({"event_type": t}))
            .collect(),
        event_outputs: outputs.iter().map(|t| json!({"event_type": t})).collect(),
        artifact_inputs: vec![],
        artifact_outputs: vec![],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        resource_requirements: None,
        sandbox: CsuSandbox {
            filesystem: "none".into(),
            network: "none".into(),
            process: "in_process".into(),
            device_access: "none".into(),
            secret_access: "none".into(),
        },
        lifecycle_hooks: None,
        provenance_refs: None,
        signature: local_signature(),
        created_at: mvp_timestamp(),
    }
    .attach_canonical_signature()
    .expect("canonical basic_manifest")
}

/// Build an event descriptor signed by `producer` under CSU tenant isolation.
#[allow(clippy::too_many_arguments)]
pub fn make_event_as(
    tenant_csu: AiraRef,
    producer: AiraRef,
    event_id: &str,
    event_type: EventType,
    object_refs: Vec<AiraRef>,
    artifact_refs: Vec<AiraRef>,
    causal_refs: Vec<AiraRef>,
    payload_ref: Option<String>,
) -> Result<EventDescriptor, CryptoError> {
    let payload = payload_ref.clone().unwrap_or_default();
    let hash = if payload.is_empty() {
        ContentHash::parse(
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .expect("hash")
    } else {
        ContentHash::sha256_bytes(payload.as_bytes())
    };
    let unsigned = EventDescriptor {
        event_id: AiraRef::parse(event_id).expect("event_id"),
        event_type,
        schema_version: "0.1".into(),
        producer_identity: producer,
        causal_refs,
        object_refs,
        artifact_refs,
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref,
        created_at: mvp_timestamp(),
        signature: local_signature(),
    };
    unsigned.attach_canonical_signature_for_tenant(&tenant_csu)
}

/// Build an event descriptor signed by the process primary (Analyze-22 path).
pub fn make_event(
    event_id: &str,
    event_type: EventType,
    object_refs: Vec<AiraRef>,
    artifact_refs: Vec<AiraRef>,
    causal_refs: Vec<AiraRef>,
    payload_ref: Option<String>,
) -> EventDescriptor {
    let payload = payload_ref.clone().unwrap_or_default();
    let hash = if payload.is_empty() {
        ContentHash::parse(
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .expect("hash")
    } else {
        ContentHash::sha256_bytes(payload.as_bytes())
    };
    let unsigned = EventDescriptor {
        event_id: AiraRef::parse(event_id).expect("event_id"),
        event_type,
        schema_version: "0.1".into(),
        producer_identity: local_identity(),
        causal_refs,
        object_refs,
        artifact_refs,
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref,
        created_at: mvp_timestamp(),
        signature: local_signature(),
    };
    unsigned
        .attach_canonical_signature()
        .expect("canonical event signature")
}

/// Build an artifact descriptor signed by `producer` under CSU tenant isolation.
pub fn make_artifact_as(
    tenant_csu: AiraRef,
    producer: AiraRef,
    artifact_id: &str,
    artifact_type: ArtifactType,
    payload: &[u8],
    provenance: Vec<AiraRef>,
) -> Result<ArtifactDescriptor, CryptoError> {
    let hash = ContentHash::sha256_bytes(payload);
    let unsigned = ArtifactDescriptor {
        artifact_id: AiraRef::parse(artifact_id).expect("artifact_id"),
        artifact_type,
        schema_version: "0.1".into(),
        content_hash: hash.clone(),
        content_ref: format!("cas://{}", hash.as_str()),
        producer_identity: producer,
        provenance_refs: provenance,
        dependency_refs: vec![],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        signature: local_signature(),
        created_at: mvp_timestamp(),
    };
    unsigned.attach_canonical_signature_for_tenant(&tenant_csu)
}

/// Build an artifact descriptor signed by the process primary (Analyze-22 path).
pub fn make_artifact(
    artifact_id: &str,
    artifact_type: ArtifactType,
    payload: &[u8],
    provenance: Vec<AiraRef>,
) -> ArtifactDescriptor {
    let hash = ContentHash::sha256_bytes(payload);
    let unsigned = ArtifactDescriptor {
        artifact_id: AiraRef::parse(artifact_id).expect("artifact_id"),
        artifact_type,
        schema_version: "0.1".into(),
        content_hash: hash.clone(),
        content_ref: format!("cas://{}", hash.as_str()),
        producer_identity: local_identity(),
        provenance_refs: provenance,
        dependency_refs: vec![],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        signature: local_signature(),
        created_at: mvp_timestamp(),
    };
    unsigned
        .attach_canonical_signature()
        .expect("canonical artifact signature")
}

/// Encode JSON value as artifact payload bytes.
pub fn json_bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).expect("json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{
        register_csu_tenant_signing, reset_primary_signer, set_primary_signer, signature_for,
        unregister_csu_tenant, LOCAL_TEST_KEY_REF,
    };
    use ed25519_dalek::SigningKey;

    #[test]
    fn publisher_override_signs_distinct_from_primary() {
        let pub_sk = SigningKey::from_bytes(&[31u8; 32]);
        let pub_id = AiraRef::parse("aira:identity:csu-publisher").unwrap();
        set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap());

        let mut manifest = basic_manifest(
            "aira:csu:context.basic",
            "context-basic",
            CsuType::Context,
            &["ProblemSubmitted"],
            &["ContextResolved"],
        );
        apply_publisher(&mut manifest, pub_id.clone());
        register_csu_tenant_signing(&manifest.csu_id, pub_id.clone(), pub_sk).unwrap();
        assert_eq!(manifest.identity_ref.as_str(), LOCAL_TEST_KEY_REF);
        assert_eq!(manifest.publisher_identity.as_str(), pub_id.as_str());

        let art = make_artifact_as(
            manifest.csu_id.clone(),
            manifest.publisher_identity.clone(),
            "aira:artifact:pub1",
            ArtifactType::ContextArtifact,
            b"{}",
            vec![],
        )
        .unwrap();
        assert_eq!(art.producer_identity.as_str(), pub_id.as_str());
        assert_eq!(art.signature.key_ref.as_str(), pub_id.as_str());
        art.verify_canonical().unwrap();

        // Tenant signing secret is not in the process signing map.
        assert!(matches!(
            signature_for(&pub_id, b"{}"),
            Err(aira_object::CryptoError::NoSigningKey(_))
        ));

        let missing = AiraRef::parse("aira:identity:no-signing-key").unwrap();
        assert!(make_artifact_as(
            manifest.csu_id.clone(),
            missing,
            "aira:artifact:x",
            ArtifactType::ContextArtifact,
            b"{}",
            vec![],
        )
        .is_err());

        unregister_csu_tenant(&manifest.csu_id);
        reset_primary_signer();
    }
}
