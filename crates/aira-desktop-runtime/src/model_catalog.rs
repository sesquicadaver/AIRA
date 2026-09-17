//! Settings → Models catalog facade (`#348` / RFC-0231).
//!
//! Thin Desktop projection over per-model lifecycle (#344), select (#345), and
//! activate/quarantine/inventory CSUs. Does **not** submit Work or invent
//! VERIFIED from selection.

use std::fs;
use std::path::{Path, PathBuf};

use aira_csu_model_acquisition::{
    activate_verified_model, fetch_to_quarantine, list_model_lifecycle, load_policy, select_model,
    verify_quarantine, write_acquisition_policy, FetchOutcome, ModelSelectError, ModelSelection,
    QuarantinePointer, VerifyOutcome, QUARANTINE_POINTER_REL,
};
use aira_csu_model_inventory::scan_and_publish;
use aira_flow::ActivatedPointerGate;
use serde::{Deserialize, Serialize};

/// One catalog row for Settings → Models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub model_ref: String,
    pub verified: bool,
    pub available: bool,
    /// Concrete readiness reason for this row (never empty fluff).
    pub ready_reason: String,
}

/// Frozen catalog view for egui / actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelCatalogSnapshot {
    pub entries: Vec<CatalogEntry>,
    /// Tip from `activated.latest` when present.
    pub tip_model_ref: Option<String>,
    /// Whether local file add is allowed by acquisition policy.
    pub local_add_allowed: bool,
    pub last_message: Option<String>,
}

/// How Settings selects an executor preference (`#348`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatalogSelection {
    Auto,
    Required(String),
}

/// Explained catalog failures — no silent substitute.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelCatalogError {
    #[error("{0}")]
    Message(String),
}

impl From<ModelSelectError> for ModelCatalogError {
    fn from(e: ModelSelectError) -> Self {
        Self::Message(e.to_string())
    }
}

fn tip_model_ref(root: &Path) -> Option<String> {
    let obs = ActivatedPointerGate::from_aira_root(root).observe();
    if obs.pointer_present {
        obs.selected_model_ref
    } else {
        None
    }
}

fn ready_reason(verified: bool, available: bool) -> String {
    match (verified, available) {
        (true, true) => "verified and available (activated cache)".into(),
        (true, false) => "verified but not activated — use Prepare".into(),
        (false, true) => "available for process (activated/bind) — not weight-verified".into(),
        (false, false) => "not verified and not available".into(),
    }
}

fn quarantine_ready_reason() -> String {
    "quarantined — Verify (artifact) then Prepare".into()
}

fn local_add_allowed(root: &Path) -> bool {
    match load_policy(root) {
        Ok(Some(p)) => p.auto_download,
        _ => false,
    }
}

fn read_quarantine_pointer(root: &Path) -> Option<QuarantinePointer> {
    let path = root.join(QUARANTINE_POINTER_REL);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Load lifecycle catalog + tip + policy posture.
pub fn load_model_catalog(
    root: impl AsRef<Path>,
) -> Result<ModelCatalogSnapshot, ModelCatalogError> {
    let root = root.as_ref();
    let life = list_model_lifecycle(root).map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    let tip = tip_model_ref(root);
    let mut entries: Vec<CatalogEntry> = life
        .into_iter()
        .map(|e| CatalogEntry {
            ready_reason: ready_reason(e.verified, e.available),
            model_ref: e.model_ref,
            verified: e.verified,
            available: e.available,
        })
        .collect();
    // Pack C: surface pending quarantine so GUI can Verify → Prepare.
    if let Some(q) = read_quarantine_pointer(root) {
        if !entries.iter().any(|e| e.model_ref == q.model_ref) {
            entries.push(CatalogEntry {
                model_ref: q.model_ref,
                verified: false,
                available: false,
                ready_reason: quarantine_ready_reason(),
            });
        }
    }
    Ok(ModelCatalogSnapshot {
        entries,
        tip_model_ref: tip,
        local_add_allowed: local_add_allowed(root),
        last_message: None,
    })
}

/// Scan scoped `models/` and refresh inventory artifact, then reload catalog.
pub fn scan_model_catalog(
    root: impl AsRef<Path>,
) -> Result<(usize, ModelCatalogSnapshot), ModelCatalogError> {
    let root = root.as_ref();
    let out =
        scan_and_publish(root, None).map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    let mut snap = load_model_catalog(root)?;
    snap.last_message = Some(format!(
        "scan published {} installed path(s); inventory {}",
        out.installed_count, out.artifact_id
    ));
    Ok((out.installed_count, snap))
}

/// Enable local-file add by writing acquisition policy `auto_download=true`.
///
/// Local source only — still no remote HTTP transfer in fetch.
pub fn enable_local_model_add(
    root: impl AsRef<Path>,
) -> Result<ModelCatalogSnapshot, ModelCatalogError> {
    let root = root.as_ref();
    write_acquisition_policy(root, true, false)
        .map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    let mut snap = load_model_catalog(root)?;
    snap.last_message =
        Some("local model add enabled (auto_download=true; still no remote fetch)".into());
    Ok(snap)
}

/// Copy a local weight file into quarantine after policy ALLOW.
pub fn add_model_file(
    root: impl AsRef<Path>,
    model_ref: &str,
    source: impl AsRef<Path>,
) -> Result<ModelCatalogSnapshot, ModelCatalogError> {
    let root = root.as_ref();
    let model_ref = model_ref.trim();
    if model_ref.is_empty() {
        return Err(ModelCatalogError::Message(
            "model_ref is required before Add".into(),
        ));
    }
    let out = fetch_to_quarantine(root, model_ref, source.as_ref())
        .map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    let mut snap = load_model_catalog(root)?;
    match out {
        FetchOutcome::Denied(gate) => {
            snap.last_message = Some(format!("add denied: {} ({})", gate.reason, gate.reason_ref));
            Err(ModelCatalogError::Message(
                snap.last_message.clone().unwrap_or_default(),
            ))
        }
        FetchOutcome::Quarantined {
            quarantine_path,
            content_hash,
            ..
        } => {
            snap.last_message = Some(format!(
                "quarantined {model_ref} at {quarantine_path} ({content_hash}); Verify with ModelArtifact then Prepare"
            ));
            Ok(snap)
        }
    }
}

/// Verify quarantined weights against a ModelArtifact path (Pack C / audit #2).
pub fn verify_catalog_quarantine(
    root: impl AsRef<Path>,
    artifact_path: impl AsRef<Path>,
) -> Result<ModelCatalogSnapshot, ModelCatalogError> {
    let root = root.as_ref();
    let out = verify_quarantine(root, artifact_path.as_ref())
        .map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    let mut snap = load_model_catalog(root)?;
    match out {
        VerifyOutcome::Verified {
            model_ref,
            verified_path,
            ..
        } => {
            snap.last_message = Some(format!(
                "verified {model_ref} → {verified_path}; use Prepare to activate"
            ));
            Ok(snap)
        }
        VerifyOutcome::Rejected { reason, .. } => {
            snap.last_message = Some(format!("verify rejected: {reason}"));
            Err(ModelCatalogError::Message(
                snap.last_message.clone().unwrap_or_default(),
            ))
        }
    }
}

/// Prepare = activate a verified model into cache (sets tip + slot).
pub fn prepare_model(
    root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<ModelCatalogSnapshot, ModelCatalogError> {
    let root = root.as_ref();
    let model_ref = model_ref.trim();
    if model_ref.is_empty() {
        return Err(ModelCatalogError::Message(
            "select a model_ref before Prepare".into(),
        ));
    }
    if model_ref.starts_with("aira:model:ollama-") {
        return Err(ModelCatalogError::Message(
            "host Ollama bind uses Settings → Models process bind — not Prepare/activate_verified (RFC-0243)"
                .into(),
        ));
    }
    let out = activate_verified_model(root, model_ref)
        .map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    // Refresh inventory for the activated cache dir (CLI parity).
    let cache = PathBuf::from(&out.cache_scan_dir);
    let _ = scan_and_publish(root, Some(&cache));
    let mut snap = load_model_catalog(root)?;
    snap.last_message = Some(format!(
        "prepared/activated {} → {}",
        out.model_ref, out.cache_path
    ));
    Ok(snap)
}

/// Select Auto or Required; when the resolved model is available, activate tip.
pub fn select_catalog_model(
    root: impl AsRef<Path>,
    selection: CatalogSelection,
) -> Result<(String, ModelCatalogSnapshot), ModelCatalogError> {
    let root = root.as_ref();
    let sel = match &selection {
        CatalogSelection::Auto => ModelSelection::Auto,
        CatalogSelection::Required(r) => ModelSelection::Required(r.clone()),
    };
    let chosen = select_model(root, sel)?;
    let life = list_model_lifecycle(root).map_err(|e| ModelCatalogError::Message(e.to_string()))?;
    let entry = life.iter().find(|e| e.model_ref == chosen.model_ref);
    let available = entry.is_some_and(|e| e.available);
    let tip = load_model_catalog(root).ok().and_then(|s| s.tip_model_ref);
    let mut snap = load_model_catalog(root)?;
    if chosen.model_ref.starts_with("aira:model:ollama-") {
        // Pack C: host-ollama tip is already process-bound; never activate_verified.
        snap.last_message = Some(format!(
            "selected host Ollama {} (process bind; no weight activate)",
            chosen.model_ref
        ));
        return Ok((chosen.model_ref, snap));
    }
    if available {
        if tip.as_deref() == Some(chosen.model_ref.as_str()) {
            // Pack C: tip already this model — skip redundant materialize/copy.
            snap.last_message = Some(format!(
                "selected {} (already tip; skipped re-activate)",
                chosen.model_ref
            ));
            return Ok((chosen.model_ref, snap));
        }
        let _ = activate_verified_model(root, &chosen.model_ref)
            .map_err(|e| ModelCatalogError::Message(e.to_string()))?;
        snap = load_model_catalog(root)?;
        snap.last_message = Some(format!("selected and tip-activated {}", chosen.model_ref));
    } else {
        snap.last_message = Some(format!(
            "selected {} but not available — use Prepare when verified ({})",
            chosen.model_ref,
            match chosen.via {
                aira_csu_model_acquisition::ModelSelectionVia::Required => "required",
                aira_csu_model_acquisition::ModelSelectionVia::AutoLatestTip => "auto tip",
                aira_csu_model_acquisition::ModelSelectionVia::AutoFirstAvailable => "auto first",
            }
        ));
    }
    Ok((chosen.model_ref, snap))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_flow::ActivatedPointerGate;
    use serial_test::serial;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn isolated() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn empty_root_lists_zero_entries() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let snap = load_model_catalog(dir.path()).unwrap();
        assert!(snap.entries.is_empty());
        assert!(!snap.local_add_allowed);
        assert!(snap.tip_model_ref.is_none());
    }

    #[test]
    #[serial]
    fn fixture_lists_available_with_reason() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        aira_object::reset_primary_signer();
        ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        let snap = load_model_catalog(dir.path()).unwrap();
        assert!(
            !snap.entries.is_empty(),
            "fixture should produce lifecycle rows"
        );
        let tip = snap.tip_model_ref.as_deref();
        assert!(tip.is_some());
        let row = snap
            .entries
            .iter()
            .find(|e| Some(e.model_ref.as_str()) == tip)
            .expect("tip row");
        assert!(row.available || row.verified);
        assert!(!row.ready_reason.is_empty());
    }

    #[test]
    fn enable_local_add_sets_policy() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        // Identity needed for policy write evidence paths in some roots.
        aira_object::reset_primary_signer();
        let _ = aira_object::register_node_identity(dir.path());
        let snap = enable_local_model_add(dir.path()).unwrap();
        assert!(snap.local_add_allowed);
    }

    #[test]
    fn add_without_policy_is_explained() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        aira_object::reset_primary_signer();
        let _ = aira_object::register_node_identity(dir.path());
        let weights = dir.path().join("w.bin");
        fs::write(&weights, b"weights").unwrap();
        let err = add_model_file(dir.path(), "aira:model:x", &weights).unwrap_err();
        assert!(
            err.to_string().contains("denied") || err.to_string().contains("policy"),
            "{err}"
        );
    }

    #[test]
    fn prepare_without_verified_is_explained() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        aira_object::reset_primary_signer();
        let _ = aira_object::register_node_identity(dir.path());
        let err = prepare_model(dir.path(), "aira:model:missing").unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn prepare_rejects_ollama_ref() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let err = prepare_model(dir.path(), "aira:model:ollama-x-deadbeefcafe").unwrap_err();
        assert!(
            err.to_string().contains("Ollama") || err.to_string().contains("process"),
            "{err}"
        );
    }

    #[test]
    fn select_auto_empty_is_explained() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let err = select_catalog_model(dir.path(), CatalogSelection::Auto).unwrap_err();
        assert!(
            err.to_string().contains("no available") || err.to_string().contains("fail-closed")
        );
    }

    /// Pack C: host-ollama Select never calls activate_verified / NoVerified path.
    #[test]
    fn select_host_ollama_skips_activate_verified() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("models/cache/ollama-slot")).unwrap();
        let model_ref = "aira:model:ollama-sel_test-deadbeefcafe";
        let cache = root.join("models/cache/ollama-slot/host-ollama.bind");
        fs::write(&cache, b"aira-host-ollama-bind\nmodel=sel-test:latest\n").unwrap();
        let pointer = serde_json::json!({
            "updated_at": "2026-09-17T00:00:00Z",
            "model_ref": model_ref,
            "cache_path": cache.display().to_string(),
            "verified_path": "",
            "content_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "evidence_artifact_id": "aira:artifact:test",
            "host_ollama_model": "sel-test:latest",
        });
        let slot = root.join("models/cache/ollama-slot").join("activated.json");
        fs::write(&slot, serde_json::to_string_pretty(&pointer).unwrap()).unwrap();
        // Slot dir name must match sanitize(model_ref) for select_model/lifecycle.
        let slot_name = model_ref
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let slot_dir = root.join("models/cache").join(&slot_name);
        fs::create_dir_all(&slot_dir).unwrap();
        let slot_ptr = slot_dir.join("activated.json");
        fs::write(&slot_ptr, serde_json::to_string_pretty(&pointer).unwrap()).unwrap();
        fs::write(
            root.join("models/activated.latest.json"),
            serde_json::to_string_pretty(&pointer).unwrap(),
        )
        .unwrap();

        let (chosen, snap) =
            select_catalog_model(root, CatalogSelection::Required(model_ref.into())).unwrap();
        assert_eq!(chosen, model_ref);
        let msg = snap.last_message.as_deref().unwrap_or("");
        assert!(
            msg.contains("process bind") || msg.contains("host Ollama"),
            "expected host-ollama select message, got {msg}"
        );
        assert!(
            !msg.contains("tip-activated"),
            "must not activate_verified: {msg}"
        );
    }
}
