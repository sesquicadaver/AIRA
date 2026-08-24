use std::fs;
use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact, make_event};
use aira_event::EventType;
use aira_object::{utc_now_rfc3339, AiraRef, ContentHash};

use crate::error::AcquisitionError;
use crate::policy::request_download;
use crate::types::{
    FetchOutcome, GateDecision, QuarantinePointer, CSU_ID, QUARANTINE_POINTER_REL, QUARANTINE_REL,
};
use crate::util::{
    append_custom_event, build_quarantine_receipt, ensure_under_models, reject_remote_source,
    sanitize_slot,
};

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
