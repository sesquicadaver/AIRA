use std::fs;
use std::path::Path;

use aira_csu::support::make_event;
use aira_event::EventType;
use aira_object::{
    is_cryptographic_signature, utc_now_rfc3339, verify_ed25519, AiraRef, ContentHash, Signature,
};
use serde_json::Value;

use crate::error::AcquisitionError;
use crate::types::{
    QuarantinePointer, VerifiedPointer, VerifyOutcome, QUARANTINE_POINTER_REL,
    VERIFIED_POINTER_REL, VERIFIED_REL,
};
use crate::util::{
    append_custom_event, ensure_under_models, publish_verify_evidence, sanitize_slot,
    signing_bytes_without_signature, VerifyEvidenceInput,
};

pub fn verify_quarantine(
    aira_root: impl AsRef<Path>,
    artifact_path: impl AsRef<Path>,
) -> Result<VerifyOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);

    let qpath = root.join(QUARANTINE_POINTER_REL);
    if !qpath.exists() {
        return Err(AcquisitionError::NoQuarantine);
    }
    let pointer: QuarantinePointer = serde_json::from_str(
        &fs::read_to_string(&qpath).map_err(|e| AcquisitionError::Io(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Other(e.to_string()))?;

    let qfile = Path::new(&pointer.quarantine_path);
    if !qfile.is_file() {
        return Err(AcquisitionError::SourceMissing(
            pointer.quarantine_path.clone(),
        ));
    }
    ensure_under_models(root, qfile)?;

    let file_bytes = fs::read(qfile).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let observed = ContentHash::sha256_bytes(&file_bytes);
    let observed_hash = observed.as_str().to_string();

    let art_raw = fs::read_to_string(artifact_path.as_ref())
        .map_err(|e| AcquisitionError::BadArtifact(e.to_string()))?;
    let artifact: Value =
        serde_json::from_str(&art_raw).map_err(|e| AcquisitionError::BadArtifact(e.to_string()))?;
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:artifact:0.1", &artifact)
                .map_err(|e| AcquisitionError::BadArtifact(e.to_string()))?;
        }
    }

    let expected_hash = artifact
        .get("content_hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let sig_val = artifact.get("signature").cloned();
    let signature: Option<Signature> = match sig_val {
        Some(v) => Some(
            serde_json::from_value(v)
                .map_err(|e| AcquisitionError::BadArtifact(format!("signature: {e}")))?,
        ),
        None => None,
    };

    let mut reject_reason: Option<(String, String)> = None;

    match &signature {
        None => {
            reject_reason = Some((
                "model artifact has no signature".into(),
                "aira:reason:model-unsigned".into(),
            ));
        }
        Some(sig) if !is_cryptographic_signature(sig) => {
            reject_reason = Some((
                "model artifact signature is not cryptographic (unsigned/legacy)".into(),
                "aira:reason:model-unsigned".into(),
            ));
        }
        Some(sig) => {
            let msg = signing_bytes_without_signature(&artifact)?;
            if verify_ed25519(sig, &msg).is_err() {
                reject_reason = Some((
                    "model artifact signature verification failed".into(),
                    "aira:reason:model-signature-invalid".into(),
                ));
            }
        }
    }

    if reject_reason.is_none() {
        match &expected_hash {
            Some(exp) if exp != &observed_hash => {
                reject_reason = Some((
                    format!("content_hash mismatch: expected {exp}, observed {observed_hash}"),
                    "aira:reason:model-hash-mismatch".into(),
                ));
            }
            None => {
                reject_reason = Some((
                    "model artifact missing content_hash".into(),
                    "aira:reason:model-hash-mismatch".into(),
                ));
            }
            Some(_) => {}
        }
    }

    if let Some((reason, reason_ref)) = reject_reason {
        let evidence_id = publish_verify_evidence(
            root,
            VerifyEvidenceInput {
                model_ref: &pointer.model_ref,
                verified: false,
                quarantine_path: &pointer.quarantine_path,
                verified_path: None,
                observed_hash: &observed_hash,
                expected_hash: expected_hash.as_deref(),
                reason_ref: &reason_ref,
            },
        )?;
        let ev_id = format!(
            "aira:event:acq-verify-reject-{}",
            &observed_hash.trim_start_matches("sha256:")
                [..16.min(observed_hash.trim_start_matches("sha256:").len())]
        );
        let event = make_event(
            &ev_id,
            EventType::CustomEvent,
            vec![],
            vec![AiraRef::parse(&evidence_id).expect("aid")],
            vec![],
            Some(format!(
                "op:verify-rejected:download:{}:{reason_ref}",
                pointer.model_ref
            )),
        );
        append_custom_event(root, event)?;
        return Ok(VerifyOutcome::Rejected {
            model_ref: pointer.model_ref,
            quarantine_path: pointer.quarantine_path,
            observed_hash,
            expected_hash,
            reason,
            reason_ref,
            evidence_artifact_id: evidence_id,
        });
    }

    let file_name = qfile
        .file_name()
        .ok_or_else(|| AcquisitionError::Other("quarantine path has no file name".into()))?
        .to_string_lossy()
        .to_string();
    let slot = sanitize_slot(&pointer.model_ref);
    let dest_dir = root.join(VERIFIED_REL).join(&slot);
    fs::create_dir_all(&dest_dir).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest_dir)?;
    let dest = dest_dir.join(&file_name);
    fs::copy(qfile, &dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest)?;

    let dest_display = dest.display().to_string();
    let reason_ref = "aira:reason:model-verified";
    let evidence_id = publish_verify_evidence(
        root,
        VerifyEvidenceInput {
            model_ref: &pointer.model_ref,
            verified: true,
            quarantine_path: &pointer.quarantine_path,
            verified_path: Some(&dest_display),
            observed_hash: &observed_hash,
            expected_hash: expected_hash.as_deref(),
            reason_ref,
        },
    )?;
    let hash_hex = observed_hash.trim_start_matches("sha256:");
    let ev_id = format!(
        "aira:event:acq-verify-ok-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&evidence_id).expect("aid")],
        vec![],
        Some(format!(
            "op:verify-passed:download:{}:{reason_ref}",
            pointer.model_ref
        )),
    );
    append_custom_event(root, event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let vpointer = VerifiedPointer {
        updated_at,
        model_ref: pointer.model_ref.clone(),
        verified_path: dest.display().to_string(),
        quarantine_path: pointer.quarantine_path.clone(),
        content_hash: observed_hash.clone(),
        evidence_artifact_id: evidence_id.clone(),
    };
    let vpath = root.join(VERIFIED_POINTER_REL);
    if let Some(parent) = vpath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &vpath,
        serde_json::to_string_pretty(&vpointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(VerifyOutcome::Verified {
        model_ref: pointer.model_ref,
        quarantine_path: pointer.quarantine_path,
        verified_path: dest.display().to_string(),
        content_hash: observed_hash,
        evidence_artifact_id: evidence_id,
    })
}
