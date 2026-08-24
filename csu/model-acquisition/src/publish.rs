use std::fs;
use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact, make_event};
use aira_event::EventType;
use aira_object::{
    active_identity, active_signature, utc_now_rfc3339, AiraRef, ContentHash, Signature,
};
use serde_json::{json, Map, Value};

use crate::error::AcquisitionError;
use crate::policy::request_publish;
use crate::types::{
    ActivatedPointer, CapabilityAdPointer, GateDecision, PublishOutcome, ShareOfferPointer,
    ACTIVATED_POINTER_REL, CAPABILITY_AD_POINTER_REL, CSU_ID, SHARE_OFFER_POINTER_REL,
};
use crate::util::{append_custom_event, ensure_under_models, sanitize_slot};

/// Gate + local signed ModelArtifact + ShareOffer + local capability ad (no remote push).
///
/// DENY from [`request_publish`] → [`PublishOutcome::Denied`] (no descriptors).
/// ALLOW requires matching [`ACTIVATED_POINTER_REL`] and cache file under `models/`.
/// Capability advertisement always uses `scope_type=local` (no federation/DHT).
pub fn publish_local(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
    visibility: &str,
    allow_download: bool,
) -> Result<PublishOutcome, AcquisitionError> {
    let visibility = visibility.trim();
    if visibility != "local" && visibility != "opt_in" {
        return Err(AcquisitionError::BadVisibility(visibility.to_string()));
    }

    let root = aira_root.as_ref();
    let gate = request_publish(root, model_ref)?;
    if gate.decision == GateDecision::Deny {
        return Ok(PublishOutcome::Denied(gate));
    }

    let apath = root.join(ACTIVATED_POINTER_REL);
    if !apath.exists() {
        return Err(AcquisitionError::NoActivated);
    }
    let activated: ActivatedPointer = serde_json::from_str(
        &fs::read_to_string(&apath).map_err(|e| AcquisitionError::Io(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Other(e.to_string()))?;
    if activated.model_ref != model_ref {
        return Err(AcquisitionError::NoActivated);
    }

    let cache_file = Path::new(&activated.cache_path);
    if !cache_file.is_file() {
        return Err(AcquisitionError::SourceMissing(
            activated.cache_path.clone(),
        ));
    }
    ensure_under_models(root, cache_file)?;

    let file_bytes = fs::read(cache_file).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let content_hash = ContentHash::sha256_bytes(&file_bytes);
    let content_hash_str = content_hash.as_str().to_string();
    if content_hash_str != activated.content_hash {
        return Err(AcquisitionError::Other(format!(
            "activated content_hash mismatch: pointer {} vs file {content_hash_str}",
            activated.content_hash
        )));
    }

    let format = infer_weight_format(cache_file);
    let publisher = active_identity();
    let model_payload = build_signed_model_artifact(
        model_ref,
        format,
        &content_hash_str,
        publisher.as_str(),
        &activated.evidence_artifact_id,
    )?;
    validate_model_payload(root, &model_payload)?;

    let model_bytes = json_bytes(&model_payload);
    let model_hash = ContentHash::sha256_bytes(&model_bytes);
    let model_hash_hex = model_hash.as_str().trim_start_matches("sha256:");
    let model_artifact_id = format!("aira:artifact:model-desc:{model_hash_hex}");
    publish_custom_payload(root, &model_artifact_id, &model_bytes)?;

    let offer_id = format!("aira:share:{}", sanitize_slot(model_ref));
    let created_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let offer_payload = build_signed_share_offer(
        &offer_id,
        publisher.as_str(),
        &model_artifact_id,
        &content_hash_str,
        visibility,
        allow_download,
        &created_at,
    )?;
    validate_share_offer_payload(root, &offer_payload)?;

    let offer_bytes = json_bytes(&offer_payload);
    let offer_hash = ContentHash::sha256_bytes(&offer_bytes);
    let offer_hash_hex = offer_hash.as_str().trim_start_matches("sha256:");
    let share_offer_artifact_id = format!("aira:artifact:share-offer:{offer_hash_hex}");
    publish_custom_payload(root, &share_offer_artifact_id, &offer_bytes)?;

    let capability_id = format!("aira:capability:model.share:{}", sanitize_slot(model_ref));
    let cap_payload = build_signed_capability_ad(
        &capability_id,
        CSU_ID,
        model_ref,
        &model_artifact_id,
        &share_offer_artifact_id,
    )?;
    validate_capability_payload(root, &cap_payload)?;
    let cap_bytes = json_bytes(&cap_payload);
    let cap_hash = ContentHash::sha256_bytes(&cap_bytes);
    let cap_hash_hex = cap_hash.as_str().trim_start_matches("sha256:");
    let capability_artifact_id = format!("aira:artifact:capability-ad:{cap_hash_hex}");
    publish_custom_payload(root, &capability_artifact_id, &cap_bytes)?;

    let ev_id = format!(
        "aira:event:share-published-{}",
        &offer_hash_hex[..16.min(offer_hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![
            AiraRef::parse(&model_artifact_id).expect("mid"),
            AiraRef::parse(&share_offer_artifact_id).expect("oid"),
            AiraRef::parse(&capability_artifact_id).expect("cid"),
        ],
        vec![],
        Some(format!(
            "op:share-published:publish:{model_ref}:{visibility}"
        )),
    );
    append_custom_event(root, event)?;

    let cap_ev_id = format!(
        "aira:event:capability-ad-{}",
        &cap_hash_hex[..16.min(cap_hash_hex.len())]
    );
    let cap_event = make_event(
        &cap_ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&capability_artifact_id).expect("cid")],
        vec![],
        Some(format!("op:capability-advertised:share:{model_ref}:local")),
    );
    append_custom_event(root, cap_event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let pointer = ShareOfferPointer {
        updated_at: updated_at.clone(),
        model_ref: model_ref.to_string(),
        offer_id: offer_id.clone(),
        model_artifact_id: model_artifact_id.clone(),
        share_offer_artifact_id: share_offer_artifact_id.clone(),
        content_hash: content_hash_str.clone(),
        visibility: visibility.to_string(),
        cache_path: activated.cache_path.clone(),
    };
    let ppath = root.join(SHARE_OFFER_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    let cap_pointer = CapabilityAdPointer {
        updated_at,
        model_ref: model_ref.to_string(),
        capability_id: capability_id.clone(),
        capability_artifact_id: capability_artifact_id.clone(),
        share_offer_artifact_id: share_offer_artifact_id.clone(),
        model_artifact_id: model_artifact_id.clone(),
        scope_type: "local".into(),
    };
    let cpath = root.join(CAPABILITY_AD_POINTER_REL);
    if let Some(parent) = cpath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &cpath,
        serde_json::to_string_pretty(&cap_pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(PublishOutcome::Published {
        gate,
        model_artifact_id,
        share_offer_artifact_id,
        offer_id,
        capability_artifact_id,
        capability_id,
        content_hash: content_hash_str,
        visibility: visibility.to_string(),
        cache_path: activated.cache_path,
    })
}

fn infer_weight_format(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("gguf") => "gguf",
        Some("safetensors") => "safetensors",
        _ => "custom",
    }
}

fn build_signed_model_artifact(
    model_id: &str,
    format: &str,
    content_hash: &str,
    publisher_ref: &str,
    provenance_extra: &str,
) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:artifact:0.1"),
    );
    body.insert("model_id".into(), json!(model_id));
    body.insert("format".into(), json!(format));
    body.insert("quantization".into(), json!("unspecified"));
    body.insert("parameter_class".into(), json!("unspecified"));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert(
        "provenance_refs".into(),
        json!([publisher_ref, provenance_extra]),
    );
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn build_signed_share_offer(
    offer_id: &str,
    publisher_ref: &str,
    model_artifact_ref: &str,
    content_hash: &str,
    visibility: &str,
    allow_download: bool,
    created_at: &str,
) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:share-offer:0.1"),
    );
    body.insert("offer_id".into(), json!(offer_id));
    body.insert("publisher_ref".into(), json!(publisher_ref));
    body.insert("model_artifact_ref".into(), json!(model_artifact_ref));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert("visibility".into(), json!(visibility));
    body.insert("allow_download".into(), json!(allow_download));
    body.insert("created_at".into(), json!(created_at));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn build_signed_capability_ad(
    capability_id: &str,
    provider_csu: &str,
    model_ref: &str,
    model_artifact_id: &str,
    share_offer_artifact_id: &str,
) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("capability_id".into(), json!(capability_id));
    body.insert("capability_type".into(), json!("model.share.local"));
    body.insert("schema_version".into(), json!("0.1"));
    body.insert("provider_csu".into(), json!(provider_csu));
    body.insert("input_artifact_types".into(), json!(["CustomArtifact"]));
    body.insert("output_artifact_types".into(), json!(["CustomArtifact"]));
    body.insert(
        "constraints".into(),
        json!({
            "privacy_class": "local",
            "model_ref": model_ref,
        }),
    );
    body.insert(
        "scope".into(),
        json!({
            "scope_type": "local",
            "description": "host-local model share capability; no federation/DHT advertise",
        }),
    );
    body.insert("policy_refs".into(), json!(["aira:policy:default"]));
    body.insert(
        "evidence_refs".into(),
        json!([model_artifact_id, share_offer_artifact_id]),
    );
    body.insert("confidence".into(), json!(1.0));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn validate_model_payload(root: &Path, payload: &Value) -> Result<(), AcquisitionError> {
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:artifact:0.1", payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    Ok(())
}

fn validate_share_offer_payload(root: &Path, payload: &Value) -> Result<(), AcquisitionError> {
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:share-offer:0.1", payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    Ok(())
}

fn validate_capability_payload(root: &Path, payload: &Value) -> Result<(), AcquisitionError> {
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:capability:descriptor:0.1", payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    Ok(())
}

fn publish_custom_payload(
    root: &Path,
    artifact_id: &str,
    bytes: &[u8],
) -> Result<(), AcquisitionError> {
    let desc = make_artifact(
        artifact_id,
        ArtifactType::CustomArtifact,
        bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }
    Ok(())
}
