//! Phase D activate verification for generate-local (QUEUE #218).
//!
//! [`ActivatedPointerGate`] lives on the plane (CSU ↛ CSU): it does not Cargo-dep
//! `model-acquisition`. Presence of `models/activated.latest.json` is not enough;
//! cache bytes, `content_hash`, and a signed activate Evidence artifact must match.
//! Desktop `#269` observes selected vs ready via [`ActivatedPointerGate::observe`].
//! `#277` light monitoring reuses a versioned metadata cache so status refresh does
//! not `fs::read` + sha256 full weights every tick; admission (`check_activated`)
//! always re-hashes weights.
//! `#278` verifies activate evidence against a **root-scoped** keyring
//! (`Keyring::load_node_identity`), not the process-global ring — so Desktop
//! reopen shows ready without test keyring priming.

#[cfg(test)]
use std::cell::Cell;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aira_artifact::{ArtifactStore, CasArtifactStore};
use aira_csu::support::{json_bytes, make_artifact};
use aira_csu_execution_llm::{GenerateLocalPayload, ModelActivateGate, ACTIVATE_DENIED};
use aira_object::{
    active_signature, is_cryptographic_signature, utc_now_rfc3339, AiraRef, ContentHash, Keyring,
    Signature,
};
use serde::{Deserialize, Serialize};
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

/// Versioned ready cache for light observe (`#277`).
///
/// Bound to pointer fingerprint + cache file length/mtime. Admission never trusts
/// this alone: [`ActivatedPointerGate::check_activated`] always full-hashes weights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ObserveReadyCache {
    pointer_fp: String,
    cache_path: String,
    cache_len: u64,
    /// Nanoseconds since UNIX_EPOCH (mtime).
    cache_mtime_ns: u128,
    content_hash: String,
}

/// Read-only activation observation for Desktop model triple (`#269`).
///
/// `selected_model_ref` comes from the pointer file; `ready` requires Phase D
/// evidence/hash confirmation (pointer presence alone is never enough). Light
/// observe (`#277`) may reuse [`ObserveReadyCache`] for the weights hash step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationObservation {
    pub pointer_present: bool,
    pub selected_model_ref: Option<String>,
    pub ready: bool,
    /// Stable English detail for tests / technical UI (not localized).
    pub detail: String,
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

    fn observe_cache_path(&self) -> PathBuf {
        self.aira_root.join("models/activated.observe-ready.json")
    }

    /// Observe selected vs ready without inventing a used-in-result model (`#269`).
    ///
    /// Uses light verification (`#277`): full weights hash only on cache miss or
    /// metadata/pointer change. Evidence artifact is always checked.
    pub fn observe(&self) -> ActivationObservation {
        if !self.pointer_path.is_file() {
            return ActivationObservation {
                pointer_present: false,
                selected_model_ref: None,
                ready: false,
                detail: "no activated pointer".into(),
            };
        }
        let raw = match fs::read_to_string(&self.pointer_path) {
            Ok(s) => s,
            Err(_) => {
                return ActivationObservation {
                    pointer_present: true,
                    selected_model_ref: None,
                    ready: false,
                    detail: "activated pointer unreadable".into(),
                };
            }
        };
        let pointer: ActivatedPointer = match serde_json::from_str(&raw) {
            Ok(p) => p,
            Err(_) => {
                return ActivationObservation {
                    pointer_present: true,
                    selected_model_ref: None,
                    ready: false,
                    detail: "activated pointer is not a Phase D activation record".into(),
                };
            }
        };
        if pointer.model_ref.is_empty() {
            return ActivationObservation {
                pointer_present: true,
                selected_model_ref: None,
                ready: false,
                detail: "activated pointer missing model_ref".into(),
            };
        }
        let selected = pointer.model_ref.clone();
        match self.verify_pointer_ready(&pointer, VerifyMode::ObserveLight) {
            Ok(()) => ActivationObservation {
                pointer_present: true,
                selected_model_ref: Some(selected),
                ready: true,
                detail: "activated evidence confirmed".into(),
            },
            Err(e) => ActivationObservation {
                pointer_present: true,
                selected_model_ref: Some(selected),
                ready: false,
                detail: e,
            },
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

    fn verify_pointer_ready(
        &self,
        pointer: &ActivatedPointer,
        mode: VerifyMode,
    ) -> Result<(), String> {
        if pointer.model_ref.is_empty()
            || pointer.cache_path.is_empty()
            || pointer.verified_path.is_empty()
            || pointer.content_hash.is_empty()
            || pointer.evidence_artifact_id.is_empty()
            || pointer.updated_at.is_empty()
        {
            return Err(ACTIVATE_DENIED.into());
        }
        let claimed = ContentHash::parse(&pointer.content_hash).map_err(|_| {
            "activated content_hash is not a valid hash (fail-closed; not VERIFIED)".to_string()
        })?;
        let cache = resolve_cache_path(&self.aira_root, &pointer.cache_path)?;
        if !cache.is_file() {
            return Err("activated cache file missing (fail-closed; not VERIFIED)".into());
        }
        let meta = fs::metadata(&cache).map_err(|_| ACTIVATE_DENIED.to_string())?;
        let cache_len = meta.len();
        let cache_mtime_ns = mtime_ns(&meta).map_err(|_| ACTIVATE_DENIED.to_string())?;
        let pointer_fp = pointer_fingerprint(pointer);

        let skip_weight_hash = matches!(mode, VerifyMode::ObserveLight)
            && observe_cache_hit(
                &self.observe_cache_path(),
                &pointer_fp,
                &pointer.cache_path,
                cache_len,
                cache_mtime_ns,
                claimed.as_str(),
            );

        if !skip_weight_hash {
            let cache_bytes = fs::read(&cache).map_err(|_| ACTIVATE_DENIED.to_string())?;
            #[cfg(test)]
            note_full_weight_hash();
            let observed = ContentHash::sha256_bytes(&cache_bytes);
            if observed != claimed {
                let _ = fs::remove_file(self.observe_cache_path());
                return Err(
                    "activated cache content_hash mismatch (fail-closed; not VERIFIED)".into(),
                );
            }
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
        verify_activate_evidence(
            &self.aira_root,
            &ev_bytes,
            &pointer.model_ref,
            claimed.as_str(),
        )?;

        // Persist light-observe binding after a successful full hash (or refresh after admit).
        if !skip_weight_hash {
            let rec = ObserveReadyCache {
                pointer_fp,
                cache_path: pointer.cache_path.clone(),
                cache_len,
                cache_mtime_ns,
                content_hash: claimed.as_str().to_string(),
            };
            let _ = write_observe_cache(&self.observe_cache_path(), &rec);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum VerifyMode {
    /// Status / UI monitoring: may skip weights sha256 when observe-ready cache hits.
    ObserveLight,
    /// Generate-local admission: always full-hash weights.
    AdmitFull,
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
        if let Some(want) = &payload.model_artifact_ref {
            if want.as_str() != pointer.model_ref {
                return Err(format!(
                    "model {} is not Phase D activated (activated {}; fail-closed; not VERIFIED)",
                    want.as_str(),
                    pointer.model_ref
                ));
            }
        }
        self.verify_pointer_ready(&pointer, VerifyMode::AdmitFull)
    }
}

fn pointer_fingerprint(pointer: &ActivatedPointer) -> String {
    let raw = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        pointer.updated_at,
        pointer.model_ref,
        pointer.cache_path,
        pointer.verified_path,
        pointer.content_hash,
        pointer.evidence_artifact_id
    );
    ContentHash::sha256_bytes(raw.as_bytes())
        .as_str()
        .to_string()
}

fn mtime_ns(meta: &fs::Metadata) -> Result<u128, ()> {
    let modified = meta.modified().map_err(|_| ())?;
    Ok(system_time_ns(modified))
}

fn system_time_ns(t: SystemTime) -> u128 {
    t.duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
}

fn observe_cache_hit(
    path: &Path,
    pointer_fp: &str,
    cache_path: &str,
    cache_len: u64,
    cache_mtime_ns: u128,
    content_hash: &str,
) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(rec) = serde_json::from_str::<ObserveReadyCache>(&raw) else {
        return false;
    };
    rec.pointer_fp == pointer_fp
        && rec.cache_path == cache_path
        && rec.cache_len == cache_len
        && rec.cache_mtime_ns == cache_mtime_ns
        && rec.content_hash == content_hash
}

fn write_observe_cache(path: &Path, rec: &ObserveReadyCache) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(rec).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
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

/// Keyring used to verify activate evidence for this `.aira` root (`#278`).
///
/// Prefer on-disk node identity; fall back to local-test-only ring for fixtures
/// that never wrote `identity/`. Does **not** mutate the process-global keyring.
fn verification_keyring(aira_root: &Path) -> Keyring {
    match Keyring::load_node_identity(aira_root) {
        Ok((_, ring)) => ring,
        Err(_) => Keyring::with_local_test(),
    }
}

fn verify_activate_evidence(
    aira_root: &Path,
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
    let ring = verification_keyring(aira_root);
    ring.verify(&sig, &msg).map_err(|_| {
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
    let raw_body = activate_evidence_body(model_ref, verified_path, cache_path, content_hash);
    let for_sign = Value::Object(raw_body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| e.to_string())?;
    let sig: Signature = active_signature(&raw).map_err(|e| e.to_string())?;
    publish_activate_evidence_signed(root, raw_body, sig)
}

fn activate_evidence_body(
    model_ref: &str,
    verified_path: &str,
    cache_path: &str,
    content_hash: &str,
) -> Map<String, Value> {
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
    body
}

fn publish_activate_evidence_signed(
    root: &Path,
    mut body: Map<String, Value>,
    sig: Signature,
) -> Result<String, String> {
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
thread_local! {
    static FULL_WEIGHT_HASHES: Cell<u64> = const { Cell::new(0) };
}

#[cfg(test)]
fn note_full_weight_hash() {
    FULL_WEIGHT_HASHES.with(|c| c.set(c.get().saturating_add(1)));
}

#[cfg(test)]
fn take_full_weight_hash_count() -> u64 {
    FULL_WEIGHT_HASHES.with(|c| {
        let n = c.get();
        c.set(0);
        n
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_csu_execution_llm::{
        GenerateLocalConstraints, ACTION_GENERATE_LOCAL, PAYLOAD_SCHEMA_ID,
    };
    use aira_object::local_test_signature;
    use std::thread;
    use std::time::Duration;

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

    #[test]
    fn observe_without_pointer_is_not_ready() {
        let dir = tempfile::tempdir().unwrap();
        let obs = ActivatedPointerGate::from_aira_root(dir.path()).observe();
        assert!(!obs.pointer_present);
        assert!(obs.selected_model_ref.is_none());
        assert!(!obs.ready);
        assert_eq!(obs.detail, "no activated pointer");
    }

    #[test]
    fn observe_fixture_selected_and_ready_differ_from_used() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        let obs = gate.observe();
        assert!(obs.pointer_present);
        assert_eq!(
            obs.selected_model_ref.as_deref(),
            Some("aira:model:test-activated")
        );
        assert!(obs.ready);
        // Observation never invents a used-in-result identity.
        assert!(obs.detail.contains("confirmed"));
    }

    #[test]
    fn observe_keeps_selected_when_not_ready() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        fs::remove_file(dir.path().join("models/cache/l218/weights.bin")).unwrap();
        let obs = gate.observe();
        assert_eq!(
            obs.selected_model_ref.as_deref(),
            Some("aira:model:test-activated")
        );
        assert!(!obs.ready);
    }

    #[test]
    fn observe_second_pass_skips_full_weight_hash() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        let _ = take_full_weight_hash_count();
        assert!(gate.observe().ready);
        assert_eq!(take_full_weight_hash_count(), 1);
        assert!(gate.observe().ready);
        assert_eq!(
            take_full_weight_hash_count(),
            0,
            "second observe must not re-hash full weights"
        );
        assert!(
            gate.observe_cache_path().is_file(),
            "observe-ready cache should be written"
        );
    }

    #[test]
    fn admit_always_full_hashes_even_after_observe_cache() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        assert!(gate.observe().ready);
        let _ = take_full_weight_hash_count();
        gate.check_activated(&dummy_payload()).unwrap();
        assert_eq!(
            take_full_weight_hash_count(),
            1,
            "admission must not trust observe-ready cache for weights"
        );
    }

    #[test]
    fn observe_rehashes_when_cache_len_changes() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        assert!(gate.observe().ready);
        let _ = take_full_weight_hash_count();
        let weights = dir.path().join("models/cache/l218/weights.bin");
        // Same prefix + extra byte → len change → light cache miss → rehash → mismatch.
        fs::write(&weights, b"aira-l218-activate-fixtureX").unwrap();
        let obs = gate.observe();
        assert!(!obs.ready);
        assert!(
            obs.detail.contains("content_hash mismatch"),
            "{}",
            obs.detail
        );
        assert_eq!(take_full_weight_hash_count(), 1);
    }

    #[test]
    fn observe_rehashes_when_mtime_changes_same_bytes() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        assert!(gate.observe().ready);
        let _ = take_full_weight_hash_count();
        let weights = dir.path().join("models/cache/l218/weights.bin");
        let bytes = fs::read(&weights).unwrap();
        thread::sleep(Duration::from_millis(20));
        fs::write(&weights, &bytes).unwrap();
        assert!(gate.observe().ready);
        assert_eq!(
            take_full_weight_hash_count(),
            1,
            "mtime change must invalidate observe-ready cache"
        );
        // Third pass hits the refreshed cache again.
        assert!(gate.observe().ready);
        assert_eq!(take_full_weight_hash_count(), 0);
    }

    /// Write install-scoped identity on disk and activate fixture signed by that
    /// key **without** registering it into the process keyring (`#278`).
    fn install_disk_identity_activate_fixture(root: &Path) -> (ActivatedPointerGate, String) {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let verifying = signing.verifying_key();
        let secret_hex = hex::encode(signing.to_bytes());
        let public_hex = hex::encode(verifying.to_bytes());
        let identity_id = format!("aira:identity:desktop.{}", uuid::Uuid::now_v7().as_simple());
        let id_ref = AiraRef::parse(&identity_id).unwrap();
        let id_dir = root.join("identity");
        fs::create_dir_all(&id_dir).unwrap();
        fs::write(id_dir.join("local.ed25519"), format!("{secret_hex}\n")).unwrap();
        let id_sig = aira_object::sign_with_key(id_ref.clone(), &signing, identity_id.as_bytes());
        let desc = json!({
            "identity_id": identity_id,
            "identity_type": "local",
            "display_name": "desktop",
            "public_key": { "algorithm": "ed25519", "key_hex": public_hex },
            "created_at": "2026-09-08T00:00:00Z",
            "key_path": "identity/local.ed25519",
            "signature": id_sig,
        });
        fs::write(
            id_dir.join("local.identity.json"),
            serde_json::to_string_pretty(&desc).unwrap(),
        )
        .unwrap();

        let cache_rel = PathBuf::from("models/cache/l218/weights.bin");
        let cache_abs = root.join(&cache_rel);
        fs::create_dir_all(cache_abs.parent().unwrap()).unwrap();
        let bytes = b"aira-l278-disk-identity-fixture";
        fs::write(&cache_abs, bytes).unwrap();
        let content_hash = ContentHash::sha256_bytes(bytes);
        let model_ref = "aira:model:test-activated";
        let verified_rel = "models/verified/l218/weights.bin";
        let body = activate_evidence_body(
            model_ref,
            verified_rel,
            &cache_abs.display().to_string(),
            content_hash.as_str(),
        );
        let raw = serde_json::to_vec(&Value::Object(body.clone())).unwrap();
        // Sign from disk ring only — never register_keyring / set_primary_signer.
        let (loaded_id, ring) = Keyring::load_node_identity(root).unwrap();
        assert_eq!(loaded_id.as_str(), identity_id.as_str());
        let sig = ring.sign(&loaded_id, &raw).unwrap();
        let evidence_id = publish_activate_evidence_signed(root, body, sig).unwrap();
        let pointer = json!({
            "updated_at": "2026-09-08T00:00:00Z",
            "model_ref": model_ref,
            "cache_path": cache_rel.to_string_lossy(),
            "verified_path": verified_rel,
            "content_hash": content_hash.as_str(),
            "evidence_artifact_id": evidence_id,
        });
        let apath = root.join("models/activated.latest.json");
        fs::write(apath, serde_json::to_string_pretty(&pointer).unwrap()).unwrap();
        (ActivatedPointerGate::from_aira_root(root), identity_id)
    }

    #[test]
    fn observe_ready_with_disk_identity_without_process_keyring_priming() {
        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let (gate, identity_id) = install_disk_identity_activate_fixture(dir.path());
        // Process ring must not hold the install identity (no priming).
        assert!(
            aira_object::process_keyring_snapshot()
                .verifying_keys(&identity_id)
                .is_empty(),
            "process keyring must not contain {identity_id}"
        );
        assert_eq!(
            aira_object::primary_signer().as_str(),
            aira_object::LOCAL_TEST_KEY_REF
        );
        let obs = gate.observe();
        assert!(obs.ready, "detail={}", obs.detail);
        assert!(obs.detail.contains("confirmed"));
        gate.check_activated(&dummy_payload()).unwrap();
    }

    #[test]
    fn observe_ready_after_reopen_without_process_keyring_priming() {
        // Child process: fresh process keyring (local-test only) + on-disk identity.
        if std::env::var_os("AIRA_278_REOPEN_CHILD").is_some() {
            let root = std::env::var("AIRA_278_REOPEN_ROOT").expect("AIRA_278_REOPEN_ROOT");
            // Deliberately do not call reset_primary_signer / register_node_identity.
            assert_eq!(
                aira_object::primary_signer().as_str(),
                aira_object::LOCAL_TEST_KEY_REF
            );
            let obs = ActivatedPointerGate::from_aira_root(&root).observe();
            assert!(obs.ready, "child observe detail={}", obs.detail);
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        aira_object::reset_primary_signer();
        let (gate, identity_id) = install_disk_identity_activate_fixture(dir.path());
        assert!(gate.observe().ready);
        let root = dir.path().to_path_buf();
        // Keep tempdir alive across child by leaking path under owned dir — child
        // reads before parent drops `dir`.
        let exe = std::env::current_exe().expect("current_exe");
        let status = std::process::Command::new(&exe)
            .env("AIRA_278_REOPEN_CHILD", "1")
            .env("AIRA_278_REOPEN_ROOT", &root)
            .env("RUST_TEST_THREADS", "1")
            .args([
                "--exact",
                "activate_gate::tests::observe_ready_after_reopen_without_process_keyring_priming",
                "--nocapture",
            ])
            .status()
            .expect("spawn reopen child");
        assert!(
            status.success(),
            "reopen child failed for identity {identity_id}: {status}"
        );
    }
}
