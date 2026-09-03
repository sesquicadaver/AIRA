//! Phase D activate verification for generate-local (QUEUE #218).
//!
//! [`ActivatedPointerGate`] lives on the plane (CSU ↛ CSU): it does not Cargo-dep
//! `model-acquisition`. Presence of `models/activated.latest.json` is not enough;
//! cache bytes, `content_hash`, and a signed activate Evidence artifact must match.

use std::fs;
use std::path::{Component, Path, PathBuf};

use aira_artifact::{ArtifactStore, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact};
use aira_csu_execution_llm::{GenerateLocalPayload, ModelActivateGate, ACTIVATE_DENIED};
use aira_object::{
    active_signature, is_cryptographic_signature, utc_now_rfc3339, verify_ed25519, AiraRef,
    ContentHash, Signature,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};

/// Pointer written by Phase D `activate_verified` (`models/activated.latest.json`).
#[derive(Debug, Clone, Deserialize)]
struct ActivatedPointer {
    updated_at: String,
    model_ref: String,
    cache_path: String,
    verified_path: String,
    content_hash: String,
    evidence_artifact_id: String,
}

/// Phase D activate handle: evidence/hash, not pointer-exists.
#[derive(Debug, Clone)]
pub struct ActivatedPointerGate {
    aira_root: PathBuf,
    pointer_path: PathBuf,
}

impl ActivatedPointerGate {
    /// Pointer path relative to an `.aira` (or equivalent) root.
    pub fn from_aira_root(root: impl AsRef<Path>) -> Self {
        let aira_root = root.as_ref().to_path_buf();
        Self {
            pointer_path: aira_root.join("models/activated.latest.json"),
            aira_root,
        }
    }

    /// Write a Phase D-shaped activate fixture (cache + hash + signed evidence).
    ///
    /// Tests / HTTP helpers only. Does not download weights.
    pub fn install_fixture(aira_root: impl AsRef<Path>) -> Result<Self, String> {
        let root = aira_root.as_ref();
        let cache_rel = PathBuf::from("models/cache/l218/weights.bin");
        let cache_abs = root.join(&cache_rel);
        if let Some(parent) = cache_abs.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = b"aira-l218-activate-fixture";
        fs::write(&cache_abs, bytes).map_err(|e| e.to_string())?;
        let content_hash = ContentHash::sha256_bytes(bytes);
        let model_ref = "aira:model:test-activated";
        let verified_rel = "models/verified/l218/weights.bin";
        let evidence_id = publish_activate_evidence(
            root,
            model_ref,
            verified_rel,
            &cache_abs.display().to_string(),
            content_hash.as_str(),
        )?;
        let updated_at = utc_now_rfc3339().map_err(|e| e.to_string())?;
        let pointer = json!({
            "updated_at": updated_at,
            "model_ref": model_ref,
            "cache_path": cache_rel.to_string_lossy(),
            "verified_path": verified_rel,
            "content_hash": content_hash.as_str(),
            "evidence_artifact_id": evidence_id,
        });
        let apath = root.join("models/activated.latest.json");
        if let Some(parent) = apath.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(
            &apath,
            serde_json::to_string_pretty(&pointer).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self::from_aira_root(root))
    }
}

impl ModelActivateGate for ActivatedPointerGate {
    fn check_activated(&self, payload: &GenerateLocalPayload) -> Result<(), String> {
        if !self.pointer_path.is_file() {
            return Err(ACTIVATE_DENIED.into());
        }
        let raw =
            fs::read_to_string(&self.pointer_path).map_err(|_| ACTIVATE_DENIED.to_string())?;
        let pointer: ActivatedPointer = serde_json::from_str(&raw).map_err(|_| {
            "activated pointer is not a Phase D activation record (fail-closed; not VERIFIED)"
                .to_string()
        })?;
        if pointer.model_ref.is_empty()
            || pointer.cache_path.is_empty()
            || pointer.verified_path.is_empty()
            || pointer.content_hash.is_empty()
            || pointer.evidence_artifact_id.is_empty()
            || pointer.updated_at.is_empty()
        {
            return Err(ACTIVATE_DENIED.into());
        }
        if let Some(want) = &payload.model_artifact_ref {
            if want.as_str() != pointer.model_ref {
                return Err(format!(
                    "model {} is not Phase D activated (activated {}; fail-closed; not VERIFIED)",
                    want.as_str(),
                    pointer.model_ref
                ));
            }
        }
        let claimed = ContentHash::parse(&pointer.content_hash).map_err(|_| {
            "activated content_hash is not a valid hash (fail-closed; not VERIFIED)".to_string()
        })?;
        let cache = resolve_cache_path(&self.aira_root, &pointer.cache_path)?;
        if !cache.is_file() {
            return Err("activated cache file missing (fail-closed; not VERIFIED)".into());
        }
        let cache_bytes = fs::read(&cache).map_err(|_| ACTIVATE_DENIED.to_string())?;
        let observed = ContentHash::sha256_bytes(&cache_bytes);
        if observed != claimed {
            return Err("activated cache content_hash mismatch (fail-closed; not VERIFIED)".into());
        }
        let evidence_id = AiraRef::parse(&pointer.evidence_artifact_id).map_err(|_| {
            "activated evidence_artifact_id is not an aira ref (fail-closed; not VERIFIED)"
                .to_string()
        })?;
        let store = CasArtifactStore::open(self.aira_root.join("artifacts")).map_err(|_| {
            "activated evidence store missing (fail-closed; not VERIFIED)".to_string()
        })?;
        let (_desc, ev_bytes) = store.resolve(&evidence_id).map_err(|_| {
            "activated evidence artifact missing (fail-closed; not VERIFIED)".to_string()
        })?;
        verify_activate_evidence(&ev_bytes, &pointer.model_ref, claimed.as_str())?;
        Ok(())
    }
}

fn resolve_cache_path(aira_root: &Path, cache_path: &str) -> Result<PathBuf, String> {
    let p = Path::new(cache_path);
    if cache_path.is_empty() || p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(ACTIVATE_DENIED.into());
    }
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        aira_root.join(p)
    };
    let models = aira_root.join("models");
    if !abs.starts_with(&models) {
        return Err("activated cache_path escapes models/ (fail-closed; not VERIFIED)".into());
    }
    Ok(abs)
}

fn signing_bytes_without_signature(artifact: &Value) -> Result<Vec<u8>, String> {
    let obj = artifact
        .as_object()
        .ok_or_else(|| ACTIVATE_DENIED.to_string())?;
    let mut body = Map::new();
    for (k, v) in obj {
        if k != "signature" {
            body.insert(k.clone(), v.clone());
        }
    }
    serde_json::to_vec(&Value::Object(body)).map_err(|_| ACTIVATE_DENIED.to_string())
}

fn verify_activate_evidence(
    bytes: &[u8],
    model_ref: &str,
    content_hash: &str,
) -> Result<(), String> {
    let body: Value = serde_json::from_slice(bytes)
        .map_err(|_| "activated evidence is not JSON (fail-closed; not VERIFIED)".to_string())?;
    if body.get("activated") != Some(&Value::Bool(true)) {
        return Err("activated evidence activated!=true (fail-closed; not VERIFIED)".into());
    }
    if body.get("model_ref").and_then(|v| v.as_str()) != Some(model_ref) {
        return Err("activated evidence model_ref mismatch (fail-closed; not VERIFIED)".into());
    }
    if body.get("content_hash").and_then(|v| v.as_str()) != Some(content_hash) {
        return Err("activated evidence content_hash mismatch (fail-closed; not VERIFIED)".into());
    }
    let sig: Signature = serde_json::from_value(
        body.get("signature").cloned().unwrap_or(Value::Null),
    )
    .map_err(|_| "activated evidence missing signature (fail-closed; not VERIFIED)".to_string())?;
    if !is_cryptographic_signature(&sig) {
        return Err(
            "activated evidence signature is not cryptographic (fail-closed; not VERIFIED)".into(),
        );
    }
    let msg = signing_bytes_without_signature(&body)?;
    verify_ed25519(&sig, &msg).map_err(|_| {
        "activated evidence signature verify failed (fail-closed; not VERIFIED)".to_string()
    })?;
    Ok(())
}

fn publish_activate_evidence(
    root: &Path,
    model_ref: &str,
    verified_path: &str,
    cache_path: &str,
    content_hash: &str,
) -> Result<String, String> {
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
    let raw = serde_json::to_vec(&for_sign).map_err(|e| e.to_string())?;
    let sig: Signature = active_signature(&raw).map_err(|e| e.to_string())?;
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| e.to_string())?,
    );
    let payload = Value::Object(body);
    let bytes = json_bytes(&payload);
    let ch = ContentHash::sha256_bytes(&bytes);
    let hash_hex = ch.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:acq-activate:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        aira_artifact::ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse("aira:csu:model.acquisition").map_err(|e| e.to_string())?],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts")).map_err(|e| e.to_string())?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(e.to_string()),
    }
    Ok(artifact_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_csu_execution_llm::{
        GenerateLocalConstraints, ACTION_GENERATE_LOCAL, PAYLOAD_SCHEMA_ID,
    };
    use aira_object::local_test_signature;

    fn dummy_payload() -> GenerateLocalPayload {
        GenerateLocalPayload {
            payload_schema: PAYLOAD_SCHEMA_ID.into(),
            action: ACTION_GENERATE_LOCAL.into(),
            prompt: "hello".into(),
            problem_statement_ref: None,
            model_artifact_ref: None,
            constraints: GenerateLocalConstraints {
                network: "none".into(),
                shell: false,
            },
            provenance_refs: vec![],
            signature: local_test_signature(aira_object::LOCAL_TEST_DOMAIN_MSG),
        }
    }

    #[test]
    fn forged_model_ref_only_pointer_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("models")).unwrap();
        fs::write(
            root.join("models/activated.latest.json"),
            r#"{"model_ref":"aira:model:anything"}"#,
        )
        .unwrap();
        let gate = ActivatedPointerGate::from_aira_root(root);
        let err = gate.check_activated(&dummy_payload()).unwrap_err();
        assert!(
            err.contains("fail-closed") || err.contains(ACTIVATE_DENIED),
            "{err}"
        );
    }

    #[test]
    fn fixture_pointer_allows_generate() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        gate.check_activated(&dummy_payload()).unwrap();
    }

    #[test]
    fn cache_hash_mismatch_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        fs::write(
            dir.path().join("models/cache/l218/weights.bin"),
            b"tampered-bytes",
        )
        .unwrap();
        let err = gate.check_activated(&dummy_payload()).unwrap_err();
        assert!(err.contains("content_hash mismatch"), "{err}");
    }
}
