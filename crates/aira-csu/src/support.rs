//! Shared builders for basic CSU implementations.

use aira_artifact::{ArtifactDescriptor, ArtifactType};
use aira_event::{EventDescriptor, EventType};
use aira_object::{active_identity, active_signature, AiraRef, ContentHash, Timestamp};
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
        signature: local_signature_over(csu_id.as_bytes()),
        created_at: mvp_timestamp(),
    }
}

/// Build an event descriptor.
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
    let sig = local_signature_over(hash.as_str().as_bytes());
    EventDescriptor {
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
        signature: sig,
    }
}

/// Build an artifact descriptor for given payload bytes.
pub fn make_artifact(
    artifact_id: &str,
    artifact_type: ArtifactType,
    payload: &[u8],
    provenance: Vec<AiraRef>,
) -> ArtifactDescriptor {
    let hash = ContentHash::sha256_bytes(payload);
    let sig = local_signature_over(hash.as_str().as_bytes());
    ArtifactDescriptor {
        artifact_id: AiraRef::parse(artifact_id).expect("artifact_id"),
        artifact_type,
        schema_version: "0.1".into(),
        content_hash: hash.clone(),
        content_ref: format!("cas://{}", hash.as_str()),
        producer_identity: local_identity(),
        provenance_refs: provenance,
        dependency_refs: vec![],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        signature: sig,
        created_at: mvp_timestamp(),
    }
}

/// Encode JSON value as artifact payload bytes.
pub fn json_bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).expect("json")
}
