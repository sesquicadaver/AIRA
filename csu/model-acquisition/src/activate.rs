use std::fs;
use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact, make_event};
use aira_event::EventType;
use aira_object::{
    active_signature, is_cryptographic_signature, utc_now_rfc3339, verify_ed25519, AiraRef,
    ContentHash, Signature,
};
use serde_json::{json, Map, Value};

use crate::error::AcquisitionError;
use crate::types::{
    ActivateOutcome, ActivatedPointer, VerifiedPointer, ACTIVATED_POINTER_REL, CACHE_REL, CSU_ID,
    VERIFIED_POINTER_REL,
};
use crate::util::{
    append_custom_event, ensure_under_models, sanitize_slot, signing_bytes_without_signature,
};

/// Authority extracted from primary signed `model-verify-evidence` (#336 / RFC-0220).
struct VerifyEvidenceAuthority {
    model_ref: String,
    content_hash: String,
}

/// Explicitly activate a verified model into local cache.
///
/// Copies `models/verified/…` → `models/cache/…`, publishes ModelInstalled-style
/// Evidence + Event. Does **not** execute the model. Inventory refresh is left to
/// the CLI (`scan_and_publish` on [`CACHE_REL`]) to respect CSU↛CSU firewall.
///
/// `#327` / RFC-0212: source and post-copy content hashes must equal the verify
/// evidence `observed_hash`. Mismatch is fail-closed (no activated pointer).
///
/// `#336` / RFC-0220: [`VerifiedPointer`] is locator-only. Model/hash/verified status
/// come from the signed verify evidence at `evidence_artifact_id`; pointer fields that
/// disagree with evidence reject before any cache write.
pub fn activate_verified(aira_root: impl AsRef<Path>) -> Result<ActivateOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);

    let vpath = root.join(VERIFIED_POINTER_REL);
    if !vpath.exists() {
        return Err(AcquisitionError::NoVerified);
    }
    let pointer: VerifiedPointer = serde_json::from_str(
        &fs::read_to_string(&vpath).map_err(|e| AcquisitionError::Io(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Other(e.to_string()))?;

    let authority = resolve_verify_evidence_authority(root, &pointer)?;

    let expected = ContentHash::parse(&authority.content_hash).map_err(|_| {
        AcquisitionError::ActivateEvidenceAuthority {
            detail: format!(
                "verify evidence observed_hash is not a valid ContentHash: {}",
                authority.content_hash
            ),
        }
    })?;

    let vfile = Path::new(&pointer.verified_path);
    if !vfile.is_file() {
        return Err(AcquisitionError::SourceMissing(
            pointer.verified_path.clone(),
        ));
    }
    ensure_under_models(root, vfile)?;

    let source_bytes = fs::read(vfile).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let source_hash = ContentHash::sha256_bytes(&source_bytes);
    if source_hash != expected {
        return Err(AcquisitionError::ActivateHashMismatch {
            expected: expected.as_str().to_string(),
            observed: source_hash.as_str().to_string(),
        });
    }

    let file_name = vfile
        .file_name()
        .ok_or_else(|| AcquisitionError::Other("verified path has no file name".into()))?
        .to_string_lossy()
        .to_string();
    let slot = sanitize_slot(&authority.model_ref);
    let dest_dir = root.join(CACHE_REL).join(&slot);
    fs::create_dir_all(&dest_dir).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest_dir)?;
    let dest = dest_dir.join(&file_name);
    fs::copy(vfile, &dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest)?;

    let file_bytes = fs::read(&dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let content_hash = ContentHash::sha256_bytes(&file_bytes);
    if content_hash != expected {
        let _ = fs::remove_file(&dest);
        return Err(AcquisitionError::ActivateHashMismatch {
            expected: expected.as_str().to_string(),
            observed: content_hash.as_str().to_string(),
        });
    }
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let dest_display = dest.display().to_string();

    let evidence_id = publish_activate_evidence(
        root,
        &authority.model_ref,
        &pointer.verified_path,
        &dest_display,
        content_hash.as_str(),
    )?;

    let ev_id = format!(
        "aira:event:acq-activate-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&evidence_id).expect("aid")],
        vec![],
        Some(format!(
            "op:model-installed:activate:{}",
            authority.model_ref
        )),
    );
    append_custom_event(root, event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let ap = ActivatedPointer {
        updated_at,
        model_ref: authority.model_ref.clone(),
        cache_path: dest_display.clone(),
        verified_path: pointer.verified_path.clone(),
        content_hash: content_hash.as_str().to_string(),
        evidence_artifact_id: evidence_id.clone(),
    };
    let apath = root.join(ACTIVATED_POINTER_REL);
    if let Some(parent) = apath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &apath,
        serde_json::to_string_pretty(&ap).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    let cache_scan = root.join(CACHE_REL);
    Ok(ActivateOutcome {
        model_ref: authority.model_ref,
        cache_path: dest_display,
        verified_path: pointer.verified_path,
        content_hash: content_hash.as_str().to_string(),
        evidence_artifact_id: evidence_id,
        cache_scan_dir: cache_scan.display().to_string(),
    })
}

/// Resolve and cryptographically verify primary verify evidence; treat pointer as locator.
fn resolve_verify_evidence_authority(
    root: &Path,
    pointer: &VerifiedPointer,
) -> Result<VerifyEvidenceAuthority, AcquisitionError> {
    if pointer.evidence_artifact_id.trim().is_empty() {
        return Err(AcquisitionError::ActivateEvidenceAuthority {
            detail: "VerifiedPointer.evidence_artifact_id is empty (locator requires evidence)"
                .into(),
        });
    }
    let evidence_id = AiraRef::parse(&pointer.evidence_artifact_id).map_err(|e| {
        AcquisitionError::ActivateEvidenceAuthority {
            detail: format!("evidence_artifact_id is not an aira ref: {e}"),
        }
    })?;

    let store = CasArtifactStore::open(root.join("artifacts")).map_err(|e| {
        AcquisitionError::ActivateEvidenceAuthority {
            detail: format!("evidence store open failed: {e}"),
        }
    })?;
    let (_desc, bytes) =
        store
            .resolve(&evidence_id)
            .map_err(|e| AcquisitionError::ActivateEvidenceAuthority {
                detail: format!("verify evidence artifact missing/unresolvable: {e}"),
            })?;

    let body: Value = serde_json::from_slice(&bytes).map_err(|e| {
        AcquisitionError::ActivateEvidenceAuthority {
            detail: format!("verify evidence is not JSON: {e}"),
        }
    })?;

    if body.get("kind").and_then(|v| v.as_str()) != Some("model-verify-evidence") {
        return Err(AcquisitionError::ActivateEvidenceAuthority {
            detail: "evidence kind must be model-verify-evidence".into(),
        });
    }
    if body.get("verified") != Some(&Value::Bool(true)) {
        return Err(AcquisitionError::ActivateEvidenceAuthority {
            detail: "verify evidence verified!=true".into(),
        });
    }

    let model_ref = body
        .get("model_ref")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AcquisitionError::ActivateEvidenceAuthority {
            detail: "verify evidence missing model_ref".into(),
        })?
        .to_string();

    let observed_hash = body
        .get("observed_hash")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AcquisitionError::ActivateEvidenceAuthority {
            detail: "verify evidence missing observed_hash".into(),
        })?
        .to_string();

    let sig: Signature = serde_json::from_value(
        body.get("signature").cloned().unwrap_or(Value::Null),
    )
    .map_err(|_| AcquisitionError::ActivateEvidenceAuthority {
        detail: "verify evidence missing or invalid signature".into(),
    })?;
    if !is_cryptographic_signature(&sig) {
        return Err(AcquisitionError::ActivateEvidenceAuthority {
            detail: "verify evidence signature is not cryptographic".into(),
        });
    }
    let msg = signing_bytes_without_signature(&body)?;
    verify_ed25519(&sig, &msg).map_err(|e| AcquisitionError::ActivateEvidenceAuthority {
        detail: format!("verify evidence signature verify failed: {e}"),
    })?;

    // Locator integrity: unsigned pointer must not disagree with signed evidence.
    if pointer.model_ref != model_ref {
        return Err(AcquisitionError::ActivateEvidenceAuthority {
            detail: format!(
                "pointer model_ref {:?} disagrees with evidence model_ref {:?}",
                pointer.model_ref, model_ref
            ),
        });
    }
    if pointer.content_hash != observed_hash {
        return Err(AcquisitionError::ActivateEvidenceAuthority {
            detail: "pointer content_hash disagrees with evidence observed_hash (pointer is locator-only)"
                .into(),
        });
    }
    if let Some(ev_path) = body.get("verified_path").and_then(|v| v.as_str()) {
        if pointer.verified_path != ev_path {
            return Err(AcquisitionError::ActivateEvidenceAuthority {
                detail: "pointer verified_path disagrees with evidence verified_path (pointer is locator-only)"
                    .into(),
            });
        }
    }

    Ok(VerifyEvidenceAuthority {
        model_ref,
        content_hash: observed_hash,
    })
}

fn publish_activate_evidence(
    root: &Path,
    model_ref: &str,
    verified_path: &str,
    cache_path: &str,
    content_hash: &str,
) -> Result<String, AcquisitionError> {
    let mut body = Map::new();
    body.insert("kind".into(), json!("model-installed-evidence"));
    body.insert("model_ref".into(), json!(model_ref));
    body.insert("verified".into(), json!(true));
    body.insert("activated".into(), json!(true));
    body.insert("executed".into(), json!(false));
    body.insert("verified_path".into(), json!(verified_path));
    body.insert("cache_path".into(), json!(cache_path));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert("reason_refs".into(), json!(["aira:reason:model-activated"]));
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
    let ch = ContentHash::sha256_bytes(&bytes);
    let hash_hex = ch.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:acq-activate:{hash_hex}");
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
