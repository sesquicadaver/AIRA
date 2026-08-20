//! Model acquisition policy gate (QUEUE #60 / Analyze-95).
//!
//! Default-deny download: missing policy or `auto_download=false` → DENY +
//! Policy decision artifact + CustomEvent. **Never** transfers model bytes
//! (real download is D4 / Out of this row). Not wired into C1.

use std::fs;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, local_identity, make_artifact, make_event, mvp_timestamp};
use aira_csu::{CsuManifest, CsuSandbox, CsuType, SUPPORTED_ABI_VERSION};
use aira_event::{EventDescriptor, EventType};
use aira_object::{
    active_identity, active_signature, utc_now_rfc3339, AiraRef, ContentHash, Signature,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use thiserror::Error;

/// Stable CSU id.
pub const CSU_ID: &str = "aira:csu:model.acquisition";
/// Optional on-disk acquisition policy payload (schema `acquisition-policy:0.1`).
pub const POLICY_FILE_REL: &str = "models/acquisition.policy.json";
/// Latest DENY/ALLOW decision pointer.
pub const DECISION_POINTER_REL: &str = "models/acquisition.decision.latest.json";

/// Acquisition gate errors.
#[derive(Debug, Error)]
pub enum AcquisitionError {
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("{0}")]
    Other(String),
}

/// Gate decision — D3 never performs a byte transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GateDecision {
    Deny,
}

impl GateDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "DENY",
        }
    }
}

/// Outcome of a download request evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireOutcome {
    pub decision: GateDecision,
    pub reason: String,
    pub reason_ref: String,
    pub model_ref: String,
    pub decision_artifact_id: String,
    pub policy_present: bool,
    pub auto_download: Option<bool>,
}

/// Pointer to the latest policy decision artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPointer {
    pub updated_at: String,
    pub decision: String,
    pub model_ref: String,
    pub reason: String,
    pub decision_artifact_id: String,
}

/// Loaded acquisition policy view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyView {
    pub auto_download: bool,
    pub allow_untrusted_models: bool,
    pub share_custom_models: bool,
}

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Signed manifest: no network; no download capability.
pub fn acquisition_manifest() -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(CSU_ID).expect("csu_id"),
        csu_name: "model-acquisition".into(),
        csu_type: CsuType::Custom,
        csu_version: "0.1.0".into(),
        abi_version: SUPPORTED_ABI_VERSION.into(),
        manifest_version: "0.1".into(),
        identity_ref: local_identity(),
        publisher_identity: local_identity(),
        capabilities: vec![],
        permissions: vec![json!({"filesystem": "read_only", "paths": ["models"]})],
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
            filesystem: "read_only".into(),
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
    .expect("canonical acquisition manifest")
}

/// Load optional local acquisition policy file.
pub fn load_policy(aira_root: impl AsRef<Path>) -> Result<Option<PolicyView>, AcquisitionError> {
    let path = aira_root.as_ref().join(POLICY_FILE_REL);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let v: Value =
        serde_json::from_str(&raw).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    if let Ok(schema_root) = aira_schema::find_repo_root(
        std::env::current_dir().unwrap_or_else(|_| aira_root.as_ref().to_path_buf()),
    ) {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:acquisition-policy:0.1", &v)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    Ok(Some(PolicyView {
        auto_download: v
            .get("auto_download")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        allow_untrusted_models: v
            .get("allow_untrusted_models")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        share_custom_models: v
            .get("share_custom_models")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
    }))
}

/// Evaluate a download request. **Always DENY transfer in D3**; publish decision evidence.
///
/// Rules:
/// - no policy → DENY (`aira:reason:no-acquisition-policy`)
/// - `auto_download=false` → DENY (`aira:reason:auto-download-false`)
/// - `auto_download=true` → still DENY (`aira:reason:download-not-implemented-d3`) — no implicit/real download
pub fn request_download(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<AcquireOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let policy = load_policy(root)?;

    let (reason, reason_ref, auto_download) = match &policy {
        None => (
            "no acquisition policy; default DENY".to_string(),
            "aira:reason:no-acquisition-policy".to_string(),
            None,
        ),
        Some(p) if !p.auto_download => (
            "auto_download=false; download DENY".to_string(),
            "aira:reason:auto-download-false".to_string(),
            Some(false),
        ),
        Some(p) => (
            "auto_download=true but download runtime not implemented (D3); DENY transfer"
                .to_string(),
            "aira:reason:download-not-implemented-d3".to_string(),
            Some(p.auto_download),
        ),
    };

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let decision_payload = build_deny_decision(&reason_ref)?;
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:policy:decision:0.1", &decision_payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }

    let bytes = json_bytes(&decision_payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:acq-deny:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }

    let ev_id = format!(
        "aira:event:acq-deny-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!(
            "op:policy-denied:download:{model_ref}:{reason_ref}"
        )),
    );
    append_custom_event(root, event)?;

    let pointer = DecisionPointer {
        updated_at,
        decision: GateDecision::Deny.as_str().into(),
        model_ref: model_ref.to_string(),
        reason: reason.clone(),
        decision_artifact_id: artifact_id.clone(),
    };
    let ppath = root.join(DECISION_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    // Explicit: no network, no file fetch, no CAS write of remote weights.
    let _host = active_identity();

    Ok(AcquireOutcome {
        decision: GateDecision::Deny,
        reason,
        reason_ref,
        model_ref: model_ref.to_string(),
        decision_artifact_id: artifact_id,
        policy_present: policy.is_some(),
        auto_download,
    })
}

fn build_deny_decision(reason_ref: &str) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("decision".into(), json!("DENY"));
    body.insert("requirements".into(), json!([]));
    body.insert("reason_refs".into(), json!([reason_ref]));
    let for_sign = Value::Object(body.clone());
    let bytes =
        serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn append_custom_event(root: &Path, event: EventDescriptor) -> Result<(), AcquisitionError> {
    let path = root.join("events").join("event-log.json");
    #[derive(Serialize, Deserialize, Default)]
    struct EventLogFile {
        events: Vec<EventDescriptor>,
    }
    let mut log: EventLogFile = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| AcquisitionError::Io(e.to_string()))?;
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
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(&log).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    fs::write(&path, json).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    Ok(())
}

/// Write a default-deny acquisition policy file (for `policy set`).
pub fn write_default_deny_policy(
    aira_root: impl AsRef<Path>,
    auto_download: bool,
) -> Result<PathBuf, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let host = active_identity();
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:acquisition-policy:0.1"),
    );
    body.insert("host_ref".into(), json!(host.as_str()));
    body.insert("auto_download".into(), json!(auto_download));
    body.insert("allow_untrusted_models".into(), json!(false));
    body.insert("share_custom_models".into(), json!(false));
    body.insert("updated_at".into(), json!(updated_at));
    let for_sign = Value::Object(body.clone());
    let bytes =
        serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    let payload = Value::Object(body);
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:acquisition-policy:0.1", &payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    let path = root.join(POLICY_FILE_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let m = acquisition_manifest();
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.csu_type, CsuType::Custom);
    }

    #[test]
    fn deny_without_policy_emits_decision() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = request_download(
            dir.path(),
            "aira:model:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert!(!out.policy_present);
        assert!(out.reason.contains("no acquisition policy"));
        assert!(dir.path().join(DECISION_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        assert!(!events.is_empty());
        let payload = events
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-denied:download:"));
    }

    #[test]
    fn deny_when_auto_download_false() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), false).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert_eq!(out.auto_download, Some(false));
        assert!(out.reason.contains("auto_download=false"));
    }

    #[test]
    fn still_deny_when_auto_download_true_no_transfer() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert!(out.reason.contains("not implemented"));
        // No downloaded weight files created.
        let models = fs::read_dir(dir.path().join("models")).unwrap();
        let weights: Vec<_> = models
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.ends_with(".gguf") || n.ends_with(".safetensors")
            })
            .collect();
        assert!(weights.is_empty());
    }
}
