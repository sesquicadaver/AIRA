//! Local contextual model rating evidence CSU (QUEUE #70 / Analyze-105).
//!
//! Publishes signed `CustomArtifact` payloads (`aira:schema:model:rating-evidence:0.1`).
//! Context is required. No network, no global scoreboard, not wired into C1.

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
pub const CSU_ID: &str = "aira:csu:model.rating";
/// Latest rating evidence pointer.
pub const RATING_POINTER_REL: &str = "models/rating.latest.json";

/// Rating CSU errors.
#[derive(Debug, Error)]
pub enum RatingError {
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("context required (context_id + task_class)")]
    MissingContext,
    #[error("{0}")]
    Other(String),
}

/// Input for a single contextual rating publish.
#[derive(Debug, Clone)]
pub struct RatingRequest {
    pub model_ref: String,
    pub context_id: String,
    pub task_class: String,
    pub reason: String,
    pub confidence: f64,
    pub backend: Option<String>,
    pub quantization: Option<String>,
    pub notes: Option<String>,
    pub fit: Option<f64>,
    pub latency: Option<f64>,
    pub quality: Option<f64>,
}

/// Result of publishing rating evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingOutcome {
    pub model_ref: String,
    pub context_id: String,
    pub task_class: String,
    pub artifact_id: String,
    pub content_hash: String,
    pub pointer_path: String,
}

/// Pointer to the latest local rating evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingPointer {
    pub updated_at: String,
    pub model_ref: String,
    pub context_id: String,
    pub task_class: String,
    pub artifact_id: String,
    pub content_hash: String,
}

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Signed manifest: scoped models FS; no network.
pub fn rating_manifest() -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(CSU_ID).expect("csu_id"),
        csu_name: "model-rating".into(),
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
    .expect("canonical rating manifest")
}

/// Publish contextual rating evidence as CustomArtifact (no network).
pub fn publish_rating(
    aira_root: impl AsRef<Path>,
    req: RatingRequest,
) -> Result<RatingOutcome, RatingError> {
    if req.context_id.trim().is_empty() || req.task_class.trim().is_empty() {
        return Err(RatingError::MissingContext);
    }
    if !(0.0..=1.0).contains(&req.confidence) {
        return Err(RatingError::Other("confidence must be in [0,1]".into()));
    }
    if req.reason.trim().is_empty() {
        return Err(RatingError::Other("reason required".into()));
    }

    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let rater = active_identity();
    let assessed_at = utc_now_rfc3339().map_err(|e| RatingError::Crypto(e.to_string()))?;

    let mut context = Map::new();
    context.insert("context_id".into(), json!(req.context_id));
    context.insert("task_class".into(), json!(req.task_class));
    context.insert("host_ref".into(), json!(rater.as_str()));
    if let Some(b) = &req.backend {
        context.insert("backend".into(), json!(b));
    }
    if let Some(q) = &req.quantization {
        context.insert("quantization".into(), json!(q));
    }
    if let Some(n) = &req.notes {
        context.insert("notes".into(), json!(n));
    }

    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:rating-evidence:0.1"),
    );
    body.insert("model_ref".into(), json!(req.model_ref));
    body.insert("rater_ref".into(), json!(rater.as_str()));
    body.insert("context".into(), Value::Object(context));
    body.insert("reason".into(), json!(req.reason));
    body.insert("confidence".into(), json!(req.confidence));
    body.insert(
        "scope".into(),
        json!({
            "scope_type": "local",
            "description": "host-local contextual rating evidence; not a global score",
        }),
    );
    body.insert("assessed_at".into(), json!(assessed_at));

    let mut scores = Map::new();
    if let Some(v) = req.fit {
        scores.insert("fit".into(), json!(v));
    }
    if let Some(v) = req.latency {
        scores.insert("latency".into(), json!(v));
    }
    if let Some(v) = req.quality {
        scores.insert("quality".into(), json!(v));
    }
    if !scores.is_empty() {
        body.insert("scores".into(), Value::Object(scores));
    }

    let payload = Value::Object(body);
    validate_rating_payload(root, &payload)?;

    let bytes = json_bytes(&payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:rating-evidence:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| RatingError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(RatingError::Artifact(e.to_string())),
    }

    let ev_id = format!("aira:event:rating-{}", &hash_hex[..16.min(hash_hex.len())]);
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!(
            "op:rating-published:{}:{}:{}",
            req.model_ref, req.task_class, req.context_id
        )),
    );
    append_custom_event(root, event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| RatingError::Crypto(e.to_string()))?;
    let pointer = RatingPointer {
        updated_at,
        model_ref: req.model_ref.clone(),
        context_id: req.context_id.clone(),
        task_class: req.task_class.clone(),
        artifact_id: artifact_id.clone(),
        content_hash: content_hash.as_str().to_string(),
    };
    let ppath = root.join(RATING_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| RatingError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer).map_err(|e| RatingError::Other(e.to_string()))?,
    )
    .map_err(|e| RatingError::Io(e.to_string()))?;

    Ok(RatingOutcome {
        model_ref: req.model_ref,
        context_id: req.context_id,
        task_class: req.task_class,
        artifact_id,
        content_hash: content_hash.as_str().to_string(),
        pointer_path: ppath.display().to_string(),
    })
}

fn validate_rating_payload(root: &Path, payload: &Value) -> Result<(), RatingError> {
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:rating-evidence:0.1", payload)
                .map_err(|e| RatingError::Schema(e.to_string()))?;
        }
    }
    Ok(())
}

fn append_custom_event(root: &Path, event: EventDescriptor) -> Result<(), RatingError> {
    let path = root.join("events").join("event-log.json");
    #[derive(Serialize, Deserialize, Default)]
    struct EventLogFile {
        events: Vec<EventDescriptor>,
    }
    let mut log: EventLogFile = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| RatingError::Io(e.to_string()))?;
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
        fs::create_dir_all(parent).map_err(|e| RatingError::Io(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(&log).map_err(|e| RatingError::Other(e.to_string()))?;
    fs::write(&path, json).map_err(|e| RatingError::Io(e.to_string()))?;
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
        fs::write(
            root.join("events").join("event-log.json"),
            "{\"events\":[]}",
        )
        .unwrap();
        fs::write(
            root.join("config.json"),
            r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
        )
        .unwrap();
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_network_none() {
        let m = rating_manifest();
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.sandbox.filesystem, "scoped");
    }

    #[test]
    fn publish_requires_context() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let err = publish_rating(
            dir.path(),
            RatingRequest {
                model_ref: "aira:model:x".into(),
                context_id: "".into(),
                task_class: "coding".into(),
                reason: "r".into(),
                confidence: 0.5,
                backend: None,
                quantization: None,
                notes: None,
                fit: None,
                latency: None,
                quality: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, RatingError::MissingContext));
    }

    #[test]
    fn publish_writes_artifact_pointer_event() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = publish_rating(
            dir.path(),
            RatingRequest {
                model_ref: "aira:model:example-7b".into(),
                context_id: "sess-1".into(),
                task_class: "coding.llm.decode".into(),
                reason: "fits local coding workload".into(),
                confidence: 0.8,
                backend: Some("llama.cpp".into()),
                quantization: Some("int4".into()),
                notes: None,
                fit: Some(0.9),
                latency: Some(0.7),
                quality: None,
            },
        )
        .unwrap();
        assert!(out.artifact_id.contains("rating-evidence"));
        assert!(PathBuf::from(&out.pointer_path).exists());
        let ptr: RatingPointer =
            serde_json::from_str(&fs::read_to_string(dir.path().join(RATING_POINTER_REL)).unwrap())
                .unwrap();
        assert_eq!(ptr.task_class, "coding.llm.decode");
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let joined: String = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:rating-published:"));
    }
}
