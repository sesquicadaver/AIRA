//! Local advisory upgrade recommendation CSU (QUEUE #73 / Analyze-108).
//!
//! Publishes `CustomArtifact` (`aira:schema:model:upgrade-recommendation:0.1`).
//! Evidence-backed; network=none; not marketplace/settlement.

use std::fs;
use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, local_identity, make_artifact, make_event, mvp_timestamp};
use aira_csu::{CsuManifest, CsuSandbox, CsuType, SUPPORTED_ABI_VERSION};
use aira_event::{EventDescriptor, EventType};
use aira_object::{active_identity, utc_now_rfc3339, AiraRef, ContentHash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use thiserror::Error;

/// Stable CSU id.
pub const CSU_ID: &str = "aira:csu:model.recommendation";
/// Latest recommendation pointer.
pub const RECOMMEND_POINTER_REL: &str = "models/recommendation.latest.json";

/// Recommendation errors.
#[derive(Debug, Error)]
pub enum RecommendError {
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("evidence_refs required (min 1)")]
    MissingEvidence,
    #[error("invalid recommendation_type (hardware|model|storage|backend|none): {0}")]
    BadType(String),
    #[error("{0}")]
    Other(String),
}

/// Input for advisory recommendation publish.
#[derive(Debug, Clone)]
pub struct RecommendRequest {
    pub recommendation_type: String,
    pub reason: String,
    pub evidence_refs: Vec<String>,
    pub confidence: f64,
    pub alternatives: Vec<String>,
}

/// Publish outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendOutcome {
    pub recommendation_id: String,
    pub recommendation_type: String,
    pub artifact_id: String,
    pub content_hash: String,
    pub pointer_path: String,
}

/// Pointer file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendPointer {
    pub updated_at: String,
    pub recommendation_id: String,
    pub recommendation_type: String,
    pub artifact_id: String,
    pub content_hash: String,
}

/// Crate version.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Manifest: scoped FS, network=none.
pub fn recommendation_manifest() -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(CSU_ID).expect("csu_id"),
        csu_name: "model-recommendation".into(),
        csu_type: CsuType::Custom,
        csu_version: "0.1.0".into(),
        abi_version: SUPPORTED_ABI_VERSION.into(),
        manifest_version: "0.1".into(),
        identity_ref: local_identity(),
        publisher_identity: local_identity(),
        capabilities: vec![],
        permissions: vec![json!({"filesystem": "scoped", "paths": ["models"]})],
        event_subscriptions: vec![json!({"event_type": "CustomEvent"})],
        event_outputs: vec![
            json!({"event_type": "CustomEvent"}),
            json!({"event_type": "ArtifactPublished"}),
        ],
        artifact_inputs: vec![],
        artifact_outputs: vec![json!({"artifact_type": "CustomArtifact"})],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        resource_requirements: None,
        sandbox: CsuSandbox {
            filesystem: "scoped".into(),
            network: "none".into(),
            process: "in_process".into(),
            device_access: "none".into(),
            secret_access: "none".into(),
        },
        lifecycle_hooks: None,
        provenance_refs: None,
        signature: aira_csu::support::local_signature(),
        created_at: mvp_timestamp(),
    }
    .attach_canonical_signature()
    .expect("canonical recommendation manifest")
}

/// Publish advisory recommendation (local-only).
pub fn publish_recommendation(
    aira_root: impl AsRef<Path>,
    req: RecommendRequest,
) -> Result<RecommendOutcome, RecommendError> {
    let rtype = req.recommendation_type.trim();
    if !matches!(rtype, "hardware" | "model" | "storage" | "backend" | "none") {
        return Err(RecommendError::BadType(rtype.to_string()));
    }
    if req.evidence_refs.is_empty() {
        return Err(RecommendError::MissingEvidence);
    }
    if req.reason.trim().is_empty() {
        return Err(RecommendError::Other("reason required".into()));
    }
    if !(0.0..=1.0).contains(&req.confidence) {
        return Err(RecommendError::Other("confidence must be in [0,1]".into()));
    }

    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let host = active_identity();
    let created_at = utc_now_rfc3339().map_err(|e| RecommendError::Crypto(e.to_string()))?;
    let slot = sanitize(rtype);
    let recommendation_id = format!(
        "aira:recommend:{slot}:{}",
        created_at[..19.min(created_at.len())].replace(':', "")
    );

    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:upgrade-recommendation:0.1"),
    );
    body.insert("recommendation_id".into(), json!(recommendation_id));
    body.insert("host_ref".into(), json!(host.as_str()));
    body.insert("recommendation_type".into(), json!(rtype));
    body.insert("reason".into(), json!(req.reason));
    body.insert("evidence_refs".into(), json!(req.evidence_refs));
    body.insert("confidence".into(), json!(req.confidence));
    if !req.alternatives.is_empty() {
        body.insert("alternatives".into(), json!(req.alternatives));
    }
    body.insert(
        "scope".into(),
        json!({
            "scope_type": "local",
            "description": "host-local advisory recommendation; not marketplace",
        }),
    );
    body.insert("created_at".into(), json!(created_at));

    let payload = Value::Object(body);
    validate_payload(root, &payload)?;

    let bytes = json_bytes(&payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:upgrade-recommend:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| RecommendError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(RecommendError::Artifact(e.to_string())),
    }

    let ev_id = format!(
        "aira:event:recommend-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!(
            "op:recommendation-published:{rtype}:{recommendation_id}"
        )),
    );
    append_custom_event(root, event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| RecommendError::Crypto(e.to_string()))?;
    let pointer = RecommendPointer {
        updated_at,
        recommendation_id: recommendation_id.clone(),
        recommendation_type: rtype.to_string(),
        artifact_id: artifact_id.clone(),
        content_hash: content_hash.as_str().to_string(),
    };
    let ppath = root.join(RECOMMEND_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| RecommendError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer).map_err(|e| RecommendError::Other(e.to_string()))?,
    )
    .map_err(|e| RecommendError::Io(e.to_string()))?;

    Ok(RecommendOutcome {
        recommendation_id,
        recommendation_type: rtype.to_string(),
        artifact_id,
        content_hash: content_hash.as_str().to_string(),
        pointer_path: ppath.display().to_string(),
    })
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn validate_payload(root: &Path, payload: &Value) -> Result<(), RecommendError> {
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:upgrade-recommendation:0.1", payload)
                .map_err(|e| RecommendError::Schema(e.to_string()))?;
        }
    }
    Ok(())
}

fn append_custom_event(root: &Path, event: EventDescriptor) -> Result<(), RecommendError> {
    let path = root.join("events").join("event-log.json");
    #[derive(Serialize, Deserialize, Default)]
    struct EventLogFile {
        events: Vec<EventDescriptor>,
    }
    let mut log: EventLogFile = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| RecommendError::Io(e.to_string()))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        EventLogFile::default()
    };
    if !log
        .events
        .iter()
        .any(|e| e.event_id.as_str() == event.event_id.as_str())
    {
        log.events.push(event);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| RecommendError::Io(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(&log).map_err(|e| RecommendError::Other(e.to_string()))?;
    fs::write(&path, json).map_err(|e| RecommendError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn init_min_root(root: &Path) {
        for d in ["artifacts", "events", "models", "identity"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        fs::write(root.join("events/event-log.json"), "{\"events\":[]}").unwrap();
        fs::write(
            root.join("config.json"),
            r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
        )
        .unwrap();
    }

    #[test]
    fn manifest_network_none() {
        assert_eq!(recommendation_manifest().sandbox.network, "none");
    }

    #[test]
    fn requires_evidence() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let err = publish_recommendation(
            dir.path(),
            RecommendRequest {
                recommendation_type: "model".into(),
                reason: "x".into(),
                evidence_refs: vec![],
                confidence: 0.5,
                alternatives: vec![],
            },
        )
        .unwrap_err();
        assert!(matches!(err, RecommendError::MissingEvidence));
    }

    #[test]
    fn publish_ok() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = publish_recommendation(
            dir.path(),
            RecommendRequest {
                recommendation_type: "model".into(),
                reason: "try smaller coding model before GPU upgrade".into(),
                evidence_refs: vec![
                    "aira:artifact:rating-evidence:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .into(),
                ],
                confidence: 0.7,
                alternatives: vec!["smaller_quantization".into(), "download_model".into()],
            },
        )
        .unwrap();
        assert!(PathBuf::from(&out.pointer_path).exists());
        assert!(out.artifact_id.contains("upgrade-recommend"));
    }
}
