use std::fs;
use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact};
use aira_event::EventDescriptor;
use aira_object::{active_signature, AiraRef, ContentHash, Signature};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::error::AcquisitionError;
use crate::types::{GateDecision, CSU_ID};

pub(crate) fn signing_bytes_without_signature(
    artifact: &Value,
) -> Result<Vec<u8>, AcquisitionError> {
    let obj = artifact
        .as_object()
        .ok_or_else(|| AcquisitionError::BadArtifact("artifact must be object".into()))?;
    let mut body = Map::new();
    for (k, v) in obj {
        if k != "signature" {
            body.insert(k.clone(), v.clone());
        }
    }
    serde_json::to_vec(&Value::Object(body)).map_err(|e| AcquisitionError::Other(e.to_string()))
}

pub(crate) struct VerifyEvidenceInput<'a> {
    pub(crate) model_ref: &'a str,
    pub(crate) verified: bool,
    pub(crate) quarantine_path: &'a str,
    pub(crate) verified_path: Option<&'a str>,
    pub(crate) observed_hash: &'a str,
    pub(crate) expected_hash: Option<&'a str>,
    pub(crate) reason_ref: &'a str,
}

pub(crate) fn publish_verify_evidence(
    root: &Path,
    input: VerifyEvidenceInput<'_>,
) -> Result<String, AcquisitionError> {
    let mut body = Map::new();
    body.insert("kind".into(), json!("model-verify-evidence"));
    body.insert("model_ref".into(), json!(input.model_ref));
    body.insert("verified".into(), json!(input.verified));
    body.insert("activated".into(), json!(false));
    body.insert("quarantine_path".into(), json!(input.quarantine_path));
    if let Some(p) = input.verified_path {
        body.insert("verified_path".into(), json!(p));
    }
    body.insert("observed_hash".into(), json!(input.observed_hash));
    if let Some(e) = input.expected_hash {
        body.insert("expected_hash".into(), json!(e));
    }
    body.insert("reason_refs".into(), json!([input.reason_ref]));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature =
        active_signature(&raw).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    let payload = Value::Object(body);
    let bytes = json_bytes(&payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let kind = if input.verified {
        "acq-verify-ok"
    } else {
        "acq-verify-reject"
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
    Ok(artifact_id)
}

pub(crate) fn reject_remote_source(source: &Path) -> Result<(), AcquisitionError> {
    let s = source.to_string_lossy();
    let lower = s.to_ascii_lowercase();
    for scheme in ["http://", "https://", "ftp://", "sftp://"] {
        if lower.starts_with(scheme) {
            return Err(AcquisitionError::RemoteSource(s.to_string()));
        }
    }
    Ok(())
}

pub(crate) fn sanitize_slot(model_ref: &str) -> String {
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

pub(crate) fn ensure_under_models(root: &Path, path: &Path) -> Result<(), AcquisitionError> {
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

pub(crate) fn build_quarantine_receipt(
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
    let sig: Signature =
        active_signature(&raw).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

pub(crate) fn build_decision(
    decision: GateDecision,
    reason_ref: &str,
) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("decision".into(), json!(decision.as_str()));
    body.insert("requirements".into(), json!([]));
    body.insert("reason_refs".into(), json!([reason_ref]));
    let for_sign = Value::Object(body.clone());
    let bytes =
        serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature =
        active_signature(&bytes).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

pub(crate) fn append_custom_event(
    root: &Path,
    event: EventDescriptor,
) -> Result<(), AcquisitionError> {
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
