use std::fs;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact, make_event};
use aira_event::EventType;
use aira_object::{
    active_identity, active_signature, utc_now_rfc3339, AiraRef, ContentHash, Signature,
};
use serde_json::{json, Map, Value};

use crate::error::AcquisitionError;
use crate::types::{
    AcquireOutcome, DecisionPointer, GateDecision, PolicyView, ShareOutcome, CSU_ID,
    DECISION_POINTER_REL, POLICY_FILE_REL, SHARE_DECISION_POINTER_REL,
};
use crate::util::{append_custom_event, build_decision};

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

/// Evaluate a publish/share request; publish decision evidence. Gate alone never writes ShareOffer.
///
/// Rules:
/// - no policy → DENY (`aira:reason:no-acquisition-policy`)
/// - `share_custom_models=false` → DENY (`aira:reason:share-custom-models-false`)
/// - `share_custom_models=true` → ALLOW (`aira:reason:share-custom-models-true`)
pub fn request_publish(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<ShareOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let policy = load_policy(root)?;

    let (decision, reason, reason_ref, share_custom_models) = match &policy {
        None => (
            GateDecision::Deny,
            "no acquisition policy; publish DENY".to_string(),
            "aira:reason:no-acquisition-policy".to_string(),
            None,
        ),
        Some(p) if !p.share_custom_models => (
            GateDecision::Deny,
            "share_custom_models=false; publish DENY".to_string(),
            "aira:reason:share-custom-models-false".to_string(),
            Some(false),
        ),
        Some(p) => (
            GateDecision::Allow,
            "share_custom_models=true; publish ALLOW (gate; ShareOffer via publish_local)"
                .to_string(),
            "aira:reason:share-custom-models-true".to_string(),
            Some(p.share_custom_models),
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
        GateDecision::Allow => "share-allow",
        GateDecision::Deny => "share-deny",
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
        Some(format!("op:{op}:publish:{model_ref}:{reason_ref}")),
    );
    append_custom_event(root, event)?;

    let pointer = DecisionPointer {
        updated_at,
        decision: decision.as_str().into(),
        model_ref: model_ref.to_string(),
        reason: reason.clone(),
        decision_artifact_id: artifact_id.clone(),
    };
    let ppath = root.join(SHARE_DECISION_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(ShareOutcome {
        decision,
        reason,
        reason_ref,
        model_ref: model_ref.to_string(),
        decision_artifact_id: artifact_id,
        policy_present: policy.is_some(),
        share_custom_models,
    })
}
/// Write an acquisition policy file (for `policy set`).
///
/// Convenience: `share_custom_models=false`.
pub fn write_default_deny_policy(
    aira_root: impl AsRef<Path>,
    auto_download: bool,
) -> Result<PathBuf, AcquisitionError> {
    write_acquisition_policy(aira_root, auto_download, false)
}

/// Write acquisition policy with explicit download and share flags.
pub fn write_acquisition_policy(
    aira_root: impl AsRef<Path>,
    auto_download: bool,
    share_custom_models: bool,
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
    body.insert("share_custom_models".into(), json!(share_custom_models));
    body.insert("updated_at".into(), json!(updated_at));
    let for_sign = Value::Object(body.clone());
    let bytes =
        serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature =
        active_signature(&bytes).map_err(|e| AcquisitionError::Other(e.to_string()))?;
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
