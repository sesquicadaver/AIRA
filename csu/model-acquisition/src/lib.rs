//! Model acquisition policy gate + local quarantine fetch (QUEUE #60–#62).
//!
//! Default-deny download: missing policy or `auto_download=false` → DENY.
//! With policy and `auto_download=true` → ALLOW. Optional local `--source`
//! copy into `<root>/models/quarantine/` (no verify/activate; `#63`/`#64`).
//! Not wired into C1. `network=none`.

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
/// Quarantine directory under scoped models tree.
pub const QUARANTINE_REL: &str = "models/quarantine";
/// Latest quarantine fetch pointer.
pub const QUARANTINE_POINTER_REL: &str = "models/quarantine.latest.json";

/// Acquisition gate / quarantine errors.
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
    #[error("remote source rejected (local --source only): {0}")]
    RemoteSource(String),
    #[error("source is not a file: {0}")]
    SourceNotFile(String),
    #[error("source not found: {0}")]
    SourceMissing(String),
    #[error("quarantine path outside scoped models: {0}")]
    OutsideScope(String),
    #[error("{0}")]
    Other(String),
}

/// Gate decision — ALLOW authorizes a future transfer; gate alone never copies bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GateDecision {
    Allow,
    Deny,
}

impl GateDecision {
    /// Stable uppercase label for CLI / pointer JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
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

/// Result of gate + optional local quarantine copy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchOutcome {
    /// Policy DENY — no bytes copied.
    Denied(AcquireOutcome),
    /// Policy ALLOW and local source copied into quarantine.
    Quarantined {
        gate: AcquireOutcome,
        quarantine_path: String,
        bytes: u64,
        content_hash: String,
        source_path: String,
    },
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

/// Pointer to the latest quarantine object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinePointer {
    pub updated_at: String,
    pub model_ref: String,
    pub quarantine_path: String,
    pub source_path: String,
    pub bytes: u64,
    pub content_hash: String,
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

/// Signed manifest: scoped models FS; no network.
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

/// Evaluate a download request; publish decision evidence. Gate alone never copies weights.
///
/// Rules:
/// - no policy → DENY (`aira:reason:no-acquisition-policy`)
/// - `auto_download=false` → DENY (`aira:reason:auto-download-false`)
/// - `auto_download=true` → ALLOW (`aira:reason:auto-download-true`)
pub fn request_download(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<AcquireOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let policy = load_policy(root)?;

    let (decision, reason, reason_ref, auto_download) = match &policy {
        None => (
            GateDecision::Deny,
            "no acquisition policy; default DENY".to_string(),
            "aira:reason:no-acquisition-policy".to_string(),
            None,
        ),
        Some(p) if !p.auto_download => (
            GateDecision::Deny,
            "auto_download=false; download DENY".to_string(),
            "aira:reason:auto-download-false".to_string(),
            Some(false),
        ),
        Some(p) => (
            GateDecision::Allow,
            "auto_download=true; download ALLOW (no transfer in this step)".to_string(),
            "aira:reason:auto-download-true".to_string(),
            Some(p.auto_download),
        ),
    };

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let decision_payload = build_decision(decision, &reason_ref)?;
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
    let kind = match decision {
        GateDecision::Allow => "acq-allow",
        GateDecision::Deny => "acq-deny",
    };
    let artifact_id = format!("aira:artifact:{kind}:{hash_hex}");
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

    let op = match decision {
        GateDecision::Allow => "policy-allowed",
        GateDecision::Deny => "policy-denied",
    };
    let ev_id = format!("aira:event:{kind}-{}", &hash_hex[..16.min(hash_hex.len())]);
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!("op:{op}:download:{model_ref}:{reason_ref}")),
    );
    append_custom_event(root, event)?;

    let pointer = DecisionPointer {
        updated_at,
        decision: decision.as_str().into(),
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

    let _host = active_identity();

    Ok(AcquireOutcome {
        decision,
        reason,
        reason_ref,
        model_ref: model_ref.to_string(),
        decision_artifact_id: artifact_id,
        policy_present: policy.is_some(),
        auto_download,
    })
}

/// Run policy gate; on ALLOW copy local `source` into scoped quarantine.
///
/// Does **not** verify hash/signature (`#63`) or activate (`#64`). Rejects URL schemes.
pub fn fetch_to_quarantine(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
    source: impl AsRef<Path>,
) -> Result<FetchOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let source = source.as_ref();
    reject_remote_source(source)?;

    let gate = request_download(root, model_ref)?;
    if gate.decision == GateDecision::Deny {
        return Ok(FetchOutcome::Denied(gate));
    }

    if !source.exists() {
        return Err(AcquisitionError::SourceMissing(
            source.display().to_string(),
        ));
    }
    if !source.is_file() {
        return Err(AcquisitionError::SourceNotFile(
            source.display().to_string(),
        ));
    }

    let file_name = source
        .file_name()
        .ok_or_else(|| AcquisitionError::Other("source has no file name".into()))?
        .to_string_lossy()
        .to_string();
    let slot = sanitize_slot(model_ref);
    let dest_dir = root.join(QUARANTINE_REL).join(&slot);
    fs::create_dir_all(&dest_dir).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest_dir)?;

    let dest = dest_dir.join(&file_name);
    fs::copy(source, &dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest)?;

    let file_bytes = fs::read(&dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let bytes = file_bytes.len() as u64;
    let content_hash = ContentHash::sha256_bytes(&file_bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;

    let receipt = build_quarantine_receipt(
        model_ref,
        &dest.display().to_string(),
        &source.display().to_string(),
        bytes,
        content_hash.as_str(),
        &gate.decision_artifact_id,
    )?;
    let receipt_bytes = json_bytes(&receipt);
    let receipt_hash = ContentHash::sha256_bytes(&receipt_bytes);
    let receipt_hex = receipt_hash.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:acq-quarantine:{receipt_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &receipt_bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &receipt_bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }

    let ev_id = format!(
        "aira:event:acq-quarantine-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!("op:quarantine-fetched:download:{model_ref}")),
    );
    append_custom_event(root, event)?;

    let pointer = QuarantinePointer {
        updated_at,
        model_ref: model_ref.to_string(),
        quarantine_path: dest.display().to_string(),
        source_path: source.display().to_string(),
        bytes,
        content_hash: content_hash.as_str().to_string(),
        decision_artifact_id: gate.decision_artifact_id.clone(),
    };
    let ppath = root.join(QUARANTINE_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(FetchOutcome::Quarantined {
        gate,
        quarantine_path: dest.display().to_string(),
        bytes,
        content_hash: content_hash.as_str().to_string(),
        source_path: source.display().to_string(),
    })
}

fn reject_remote_source(source: &Path) -> Result<(), AcquisitionError> {
    let s = source.to_string_lossy();
    let lower = s.to_ascii_lowercase();
    for scheme in ["http://", "https://", "ftp://", "sftp://"] {
        if lower.starts_with(scheme) {
            return Err(AcquisitionError::RemoteSource(s.to_string()));
        }
    }
    Ok(())
}

fn sanitize_slot(model_ref: &str) -> String {
    let mut out = String::with_capacity(model_ref.len());
    for c in model_ref.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "model".into()
    } else {
        out
    }
}

fn ensure_under_models(root: &Path, path: &Path) -> Result<(), AcquisitionError> {
    let models = root.join("models");
    fs::create_dir_all(&models).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let scope = models
        .canonicalize()
        .map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let canon = path
        .canonicalize()
        .map_err(|e| AcquisitionError::Io(format!("{}: {e}", path.display())))?;
    if !canon.starts_with(&scope) {
        return Err(AcquisitionError::OutsideScope(path.display().to_string()));
    }
    Ok(())
}

fn build_quarantine_receipt(
    model_ref: &str,
    quarantine_path: &str,
    source_path: &str,
    bytes: u64,
    content_hash: &str,
    decision_artifact_id: &str,
) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("kind".into(), json!("model-quarantine-receipt"));
    body.insert("model_ref".into(), json!(model_ref));
    body.insert("quarantine_path".into(), json!(quarantine_path));
    body.insert("source_path".into(), json!(source_path));
    body.insert("bytes".into(), json!(bytes));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert("decision_artifact_id".into(), json!(decision_artifact_id));
    body.insert("verified".into(), json!(false));
    body.insert("activated".into(), json!(false));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn build_decision(decision: GateDecision, reason_ref: &str) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("decision".into(), json!(decision.as_str()));
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

/// Write an acquisition policy file (for `policy set`).
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

    fn weight_files(root: &Path) -> Vec<String> {
        fs::read_dir(root.join("models"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                if n.ends_with(".gguf") || n.ends_with(".safetensors") {
                    Some(n)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_network_none_scoped_fs() {
        let m = acquisition_manifest();
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.sandbox.filesystem, "scoped");
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
    fn allow_when_auto_download_true_no_transfer() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Allow);
        assert_eq!(out.auto_download, Some(true));
        assert_eq!(out.reason_ref, "aira:reason:auto-download-true");
        assert!(out.decision_artifact_id.contains("acq-allow"));
        let pointer: DecisionPointer = serde_json::from_str(
            &fs::read_to_string(dir.path().join(DECISION_POINTER_REL)).unwrap(),
        )
        .unwrap();
        assert_eq!(pointer.decision, "ALLOW");
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        let payload = events
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-allowed:download:"));
        assert!(weight_files(dir.path()).is_empty());
        assert!(!dir.path().join("models/quarantine").exists());
    }

    #[test]
    fn quarantine_fetch_after_allow_copies_local_source() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("outside-weights.gguf");
        fs::write(&src, b"fake-gguf-bytes").unwrap();
        let out = fetch_to_quarantine(dir.path(), "aira:model:example", &src).unwrap();
        match out {
            FetchOutcome::Quarantined {
                gate,
                quarantine_path,
                bytes,
                ..
            } => {
                assert_eq!(gate.decision, GateDecision::Allow);
                assert_eq!(bytes, 15);
                assert!(Path::new(&quarantine_path).exists());
                assert!(quarantine_path.contains("quarantine"));
                assert!(!quarantine_path.contains("verified"));
            }
            FetchOutcome::Denied(_) => panic!("expected quarantine"),
        }
        assert!(dir.path().join(QUARANTINE_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        let joined: String = events
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:quarantine-fetched:download:"));
        // Not activated into inventory cache root as loose weight.
        assert!(weight_files(dir.path()).is_empty());
    }

    #[test]
    fn quarantine_denied_without_policy_no_copy() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let src = dir.path().join("x.gguf");
        fs::write(&src, b"data").unwrap();
        let out = fetch_to_quarantine(dir.path(), "aira:model:x", &src).unwrap();
        assert!(matches!(out, FetchOutcome::Denied(_)));
        assert!(!dir.path().join("models/quarantine").exists());
    }

    #[test]
    fn quarantine_rejects_http_source() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let err = fetch_to_quarantine(
            dir.path(),
            "aira:model:x",
            Path::new("https://example.com/m.gguf"),
        )
        .unwrap_err();
        assert!(matches!(err, AcquisitionError::RemoteSource(_)));
    }
}
