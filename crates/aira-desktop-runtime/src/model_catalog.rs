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

/// Resolve host CLI name (`ollama list` token) for an `aira:model:ollama-*` row.
///
/// `#379`: only from verified bind data (`host_ollama_model_ref(cli) == model_ref`).
/// Pointer files locate the bind; a tampered `host_ollama_model` does not become a
/// CLI name for Make default.
pub fn host_cli_name_for_ollama_ref(root: impl AsRef<Path>, model_ref: &str) -> Option<String> {
    if !model_ref.starts_with("aira:model:ollama-") {
        return None;
    }
    ActivatedPointerGate::from_aira_root(root.as_ref()).verified_host_cli_for_model_ref(model_ref)
}

/// Display name for a catalog row. `model_ref` and hashes stay out of this string.
pub fn catalog_display_name(model_ref: &str) -> String {
    let rest = model_ref.strip_prefix("aira:model:").unwrap_or(model_ref);
    if rest.is_empty() {
        model_ref.to_string()
    } else {
        rest.to_string()
    }
}

/// Host-ollama catalog identity (`aira:model:ollama-…`). Not a local weight file.
pub fn is_host_ollama_catalog_ref(model_ref: &str) -> bool {
    model_ref.starts_with("aira:model:ollama-")
}

/// Where a shared catalog row came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogSource {
    HostOllama,
    LocalFile,
}

/// One row shared by Settings and Work: name, source, availability.
///
/// `model_ref` is for details and admission, not the primary label.
/// `cli_name` is the exact host token for bind when known; a display label alone
/// is never a CLI name (`#363`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProjectionRow {
    pub name: String,
    /// Exact `ollama list` / `host_ollama_model` token. `None` → row visible, bind refused.
    pub cli_name: Option<String>,
    pub source: CatalogSource,
    pub available: bool,
    pub verified: bool,
    pub model_ref: String,
    pub ready_reason: String,
}

/// Settings and Work share this projection.
///
/// Host names come from `ollama list`. Local-file rows never include
/// `aira:model:ollama-` refs, so Prepare is not offered for a bind that cannot activate.
/// Lifecycle-only host rows stay visible; without an exact CLI token they are not bindable.
///
/// `#380`: verified CLI names are indexed in one tip/cache pass (O(slots)), then
/// each saved host row is a map lookup — not a fresh disk walk per entry.
pub fn project_shared_catalog(
    snap: &ModelCatalogSnapshot,
    ollama_names: &[String],
    root: impl AsRef<Path>,
) -> Vec<CatalogProjectionRow> {
    let root = root.as_ref();
    let mut rows = Vec::new();
    let mut seen_refs = std::collections::HashSet::new();
    let cli_by_ref = ActivatedPointerGate::from_aira_root(root).verified_host_cli_index();
    for name in ollama_names {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let model_ref = aira_flow::host_ollama_model_ref(name);
        if !seen_refs.insert(model_ref.clone()) {
            continue;
        }
        let cat = snap.entries.iter().find(|e| e.model_ref == model_ref);
        rows.push(CatalogProjectionRow {
            name: name.to_string(),
            cli_name: Some(name.to_string()),
            source: CatalogSource::HostOllama,
            available: true,
            verified: cat.is_some_and(|e| e.verified),
            model_ref,
            ready_reason: cat
                .map(|e| e.ready_reason.clone())
                .unwrap_or_else(|| "listed by host ollama — not weight-verified".into()),
        });
    }
    for entry in &snap.entries {
        if !is_host_ollama_catalog_ref(&entry.model_ref)
            || !seen_refs.insert(entry.model_ref.clone())
        {
            continue;
        }
        let cli = cli_by_ref.get(&entry.model_ref).cloned();
        let name = cli
            .clone()
            .unwrap_or_else(|| catalog_display_name(&entry.model_ref));
        rows.push(CatalogProjectionRow {
            name,
            cli_name: cli,
            source: CatalogSource::HostOllama,
            available: entry.available,
            verified: entry.verified,
            model_ref: entry.model_ref.clone(),
            ready_reason: entry.ready_reason.clone(),
        });
    }
    for entry in &snap.entries {
        if is_host_ollama_catalog_ref(&entry.model_ref) {
            continue;
        }
        rows.push(CatalogProjectionRow {
            name: catalog_display_name(&entry.model_ref),
            cli_name: None,
            source: CatalogSource::LocalFile,
            available: entry.available,
            verified: entry.verified,
            model_ref: entry.model_ref.clone(),
            ready_reason: entry.ready_reason.clone(),
        });
    }
    rows
}

/// Exact host CLI token for bind. A screen label is never invented into a CLI name (`#363`).
pub fn exact_cli_name_for_bind(
    pick: Option<&str>,
    listed: &[String],
    rows: &[CatalogProjectionRow],
) -> Option<String> {
    let pick = pick.map(str::trim).filter(|s| !s.is_empty())?;
    if let Some(row) = rows.iter().find(|r| {
        r.source == CatalogSource::HostOllama
            && (r.name == pick || r.cli_name.as_deref() == Some(pick) || r.model_ref == pick)
    }) {
        return row
            .cli_name
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
    listed
        .iter()
        .find(|n| n.trim() == pick)
        .map(|n| n.trim().to_string())
}

/// Bind target for “Use Ollama” when a name is already known.
///
/// `None` means wait for the host list, or refuse — never invent a CLI name from
/// the first `ollama list` row (`#364`). A pick that is only a display label is
/// never returned as a CLI name (`#363`).
pub fn resolve_use_ollama_bind(
    pick: Option<&str>,
    bound: Option<&str>,
    listed: &[String],
) -> Option<String> {
    fn nonempty(s: &str) -> Option<String> {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }
    fn exact_in_list(candidate: &str, listed: &[String]) -> Option<String> {
        listed
            .iter()
            .find(|n| n.trim() == candidate.trim())
            .map(|n| n.trim().to_string())
    }
    if let Some(p) = pick.and_then(nonempty) {
        if let Some(exact) = exact_in_list(&p, listed) {
            return Some(exact);
        }
        if !is_display_label_not_cli_name(&p) && listed.is_empty() {
            // List not loaded yet; pick still looks like a host token.
            return Some(p);
        }
        // Display label or non-exact pick — never bind the label itself.
    }
    if let Some(b) = bound.and_then(nonempty) {
        if is_display_label_not_cli_name(&b) {
            // Fall through; do not treat a screen label as a saved CLI name.
        } else if listed.is_empty() {
            return Some(b);
        } else if let Some(exact) = exact_in_list(&b, listed) {
            return Some(exact);
        }
    }
    // `#364`: no first-row fallback — require an exact pick or exact saved bind.
    None
}

/// True when `s` is a catalog display label (or full model_ref), not an `ollama list` token.
pub fn is_display_label_not_cli_name(s: &str) -> bool {
    let t = s.trim();
    if t.starts_with("aira:model:") {
        return true;
    }
    // `catalog_display_name("aira:model:ollama-…-xxxxxxxxxxxx")` → `ollama-…-xxxxxxxxxxxx`
    let Some(rest) = t.strip_prefix("ollama-") else {
        return false;
    };
    if rest.len() < 13 {
        return false;
    }
    let hash = &rest[rest.len() - 13..];
    hash.starts_with('-') && hash.len() == 13 && hash[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// After a successful `ollama list`, finish a pending Use Ollama bind.
///
/// Pick binds only when it is an exact list token. Display labels are refused (`#363`).
/// Absent pick does **not** bind the first list row (`#364`).
pub fn resolve_bind_after_ollama_list(pick: Option<&str>, listed: &[String]) -> Option<String> {
    let pick = pick.map(str::trim).filter(|s| !s.is_empty());
    if let Some(p) = pick {
        if is_display_label_not_cli_name(p) {
            return None;
        }
        if listed.iter().any(|n| n.trim() == p) {
            return Some(p.to_string());
        }
        // Pick present but not exact — do not invent a CLI name from the first row.
        return None;
    }
    // `#364`: empty pick after list — wait for an explicit row, never first-row.
    None
}

/// Select Auto or Required; when the resolved model is available, activate tip.
///
/// Host-ollama rows never call `activate_verified`. Tip change for host-ollama is
/// Desktop's make-default bind (`install_host_ollama_bind`); this path reports tip
/// honesty so the UI never pretends Auto tip changed when it did not.
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
        // Pack C / P3: never activate_verified for host-ollama.
        if tip.as_deref() == Some(chosen.model_ref.as_str()) {
            snap.last_message = Some(format!(
                "selected host Ollama {} — already the default tip (Work Auto unchanged)",
                chosen.model_ref
            ));
        } else {
            snap.last_message = Some(format!(
                "selected host Ollama {} — tip not changed yet (make default in Settings → Models)",
                chosen.model_ref
            ));
        }
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
    #[serial]
    fn select_host_ollama_skips_activate_verified() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        use serde_json::json;

        let _g = isolated();
        let dir = tempdir().unwrap();
        let root = dir.path();
        aira_object::reset_primary_signer();
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let verifying = signing.verifying_key();
        let secret_hex = hex::encode(signing.to_bytes());
        let public_hex = hex::encode(verifying.to_bytes());
        let identity_id = format!("aira:identity:desktop.{}", uuid::Uuid::now_v7().as_simple());
        let id_ref = aira_object::AiraRef::parse(&identity_id).unwrap();
        let id_dir = root.join("identity");
        fs::create_dir_all(&id_dir).unwrap();
        fs::write(id_dir.join("local.ed25519"), format!("{secret_hex}\n")).unwrap();
        let id_sig = aira_object::sign_with_key(id_ref, &signing, identity_id.as_bytes());
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

        let (_gate, model_ref) =
            ActivatedPointerGate::install_host_ollama_bind(root, "sel-test:latest").unwrap();

        let (chosen, snap) =
            select_catalog_model(root, CatalogSelection::Required(model_ref.clone())).unwrap();
        assert_eq!(chosen, model_ref);
        let msg = snap.last_message.as_deref().unwrap_or("");
        assert!(
            msg.contains("host Ollama") && (msg.contains("tip") || msg.contains("default")),
            "expected host-ollama tip-honesty message, got {msg}"
        );
        assert!(
            !msg.contains("tip-activated"),
            "must not activate_verified: {msg}"
        );
        assert_eq!(
            host_cli_name_for_ollama_ref(root, &model_ref).as_deref(),
            Some("sel-test:latest")
        );
    }

    #[test]
    fn projection_file_source_omits_ollama_rows() {
        let dir = tempfile::tempdir().unwrap();
        let snap = ModelCatalogSnapshot {
            entries: vec![
                CatalogEntry {
                    model_ref: "aira:model:ollama-phi-aaaaaaaaaaaa".into(),
                    verified: false,
                    available: true,
                    ready_reason: "available for process (activated/bind) — not weight-verified"
                        .into(),
                },
                CatalogEntry {
                    model_ref: "aira:model:weights-a".into(),
                    verified: true,
                    available: false,
                    ready_reason: "verified but not activated — use Prepare".into(),
                },
            ],
            ..ModelCatalogSnapshot::default()
        };
        let rows = project_shared_catalog(&snap, &["phi:latest".into()], dir.path());
        let files: Vec<_> = rows
            .iter()
            .filter(|r| r.source == CatalogSource::LocalFile)
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "weights-a");
        assert!(files[0].cli_name.is_none());
        assert!(!is_host_ollama_catalog_ref(&files[0].model_ref));
        let host: Vec<_> = rows
            .iter()
            .filter(|r| r.source == CatalogSource::HostOllama)
            .collect();
        assert!(host.iter().any(|r| r.name == "phi:latest" && r.available));
        assert!(host
            .iter()
            .all(|r| is_host_ollama_catalog_ref(&r.model_ref)));
        assert!(host.iter().all(|r| r.name != r.model_ref));
        assert!(host
            .iter()
            .any(|r| r.cli_name.as_deref() == Some("phi:latest")));
    }

    #[test]
    fn display_label_is_not_accepted_as_cli_bind() {
        let dir = tempfile::tempdir().unwrap();
        let model_ref = aira_flow::host_ollama_model_ref("kept-tip:latest");
        let label = catalog_display_name(&model_ref);
        assert!(is_display_label_not_cli_name(&label));
        assert!(is_display_label_not_cli_name(&model_ref));
        assert!(!is_display_label_not_cli_name("kept-tip:latest"));

        let snap = ModelCatalogSnapshot {
            entries: vec![CatalogEntry {
                model_ref: model_ref.clone(),
                verified: false,
                available: true,
                ready_reason: "available".into(),
            }],
            ..ModelCatalogSnapshot::default()
        };
        // Lifecycle row without host_ollama_model in slot → visible, not bindable.
        let rows = project_shared_catalog(&snap, &[], dir.path());
        let host = rows
            .iter()
            .find(|r| r.model_ref == model_ref)
            .expect("row stays visible");
        assert_eq!(host.name, label);
        assert!(host.cli_name.is_none());
        assert!(exact_cli_name_for_bind(Some(&label), &[], &rows).is_none());
        assert!(exact_cli_name_for_bind(Some(&label), &["other:latest".into()], &rows).is_none());
        assert_eq!(
            exact_cli_name_for_bind(Some("other:latest"), &["other:latest".into()], &rows)
                .as_deref(),
            Some("other:latest")
        );

        assert!(resolve_use_ollama_bind(Some(&label), None, &[]).is_none());
        // Label pick must not become the CLI name — and must not fall through to first row (#364).
        let with_list = resolve_use_ollama_bind(Some(&label), None, &["phi:latest".into()]);
        assert!(with_list.is_none());
        assert!(resolve_bind_after_ollama_list(Some(&label), &["phi:latest".into()]).is_none());
    }

    /// `#379`: tampered pointer host must not become a Make-default CLI bind.
    #[test]
    #[serial]
    fn tampered_pointer_host_is_not_make_default_cli() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        use serde_json::json;

        let _g = isolated();
        let dir = tempdir().unwrap();
        aira_object::reset_primary_signer();
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        let verifying = signing.verifying_key();
        let secret_hex = hex::encode(signing.to_bytes());
        let public_hex = hex::encode(verifying.to_bytes());
        let identity_id = format!("aira:identity:desktop.{}", uuid::Uuid::now_v7().as_simple());
        let id_ref = aira_object::AiraRef::parse(&identity_id).unwrap();
        let id_dir = dir.path().join("identity");
        fs::create_dir_all(&id_dir).unwrap();
        fs::write(id_dir.join("local.ed25519"), format!("{secret_hex}\n")).unwrap();
        let id_sig = aira_object::sign_with_key(id_ref, &signing, identity_id.as_bytes());
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

        let (_gate, model_ref) =
            ActivatedPointerGate::install_host_ollama_bind(dir.path(), "trusted:latest").unwrap();
        assert_eq!(
            host_cli_name_for_ollama_ref(dir.path(), &model_ref).as_deref(),
            Some("trusted:latest")
        );

        let tip_path = dir.path().join("models/activated.latest.json");
        let mut tip: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&tip_path).unwrap()).unwrap();
        tip["host_ollama_model"] = json!("evil-injected:latest");
        fs::write(&tip_path, serde_json::to_string_pretty(&tip).unwrap()).unwrap();
        // Slot pointer is also a locator source for lifecycle rows.
        let cache = dir.path().join("models/cache");
        if let Ok(entries) = fs::read_dir(&cache) {
            for ent in entries.flatten() {
                let slot = ent.path().join("activated.json");
                if slot.is_file() {
                    let mut slot_tip: serde_json::Value =
                        serde_json::from_str(&fs::read_to_string(&slot).unwrap()).unwrap();
                    if slot_tip.get("model_ref").and_then(|x| x.as_str())
                        == Some(model_ref.as_str())
                    {
                        slot_tip["host_ollama_model"] = json!("evil-injected:latest");
                        fs::write(&slot, serde_json::to_string_pretty(&slot_tip).unwrap()).unwrap();
                    }
                }
            }
        }

        assert!(
            host_cli_name_for_ollama_ref(dir.path(), &model_ref).is_none(),
            "forged pointer host must not become CLI"
        );

        let snap = ModelCatalogSnapshot {
            entries: vec![CatalogEntry {
                model_ref: model_ref.clone(),
                verified: false,
                available: true,
                ready_reason: "available".into(),
            }],
            tip_model_ref: Some(model_ref.clone()),
            ..ModelCatalogSnapshot::default()
        };
        let rows = project_shared_catalog(&snap, &[], dir.path());
        let host = rows
            .iter()
            .find(|r| r.model_ref == model_ref)
            .expect("corrupted bind stays visible");
        assert!(host.cli_name.is_none());
        assert_eq!(host.name, catalog_display_name(&model_ref));
        assert!(exact_cli_name_for_bind(Some(&host.name), &[], &rows).is_none());
        assert!(exact_cli_name_for_bind(Some("evil-injected:latest"), &[], &rows).is_none());
    }

    #[test]
    fn use_ollama_waits_for_empty_list() {
        assert!(resolve_use_ollama_bind(None, None, &[]).is_none());
        assert_eq!(
            resolve_use_ollama_bind(Some("b:latest"), Some("a:latest"), &[]).as_deref(),
            Some("b:latest"),
            "a request pick is not the saved default"
        );
    }

    /// `#364`: Use Ollama never invents a CLI name from the first `ollama list` row.
    #[test]
    fn use_ollama_does_not_bind_first_list_row() {
        let listed = vec!["phi:latest".into(), "llama3:latest".into()];
        assert!(
            resolve_use_ollama_bind(None, None, &listed).is_none(),
            "no pick and no saved bind must not take listed[0]"
        );
        assert!(
            resolve_bind_after_ollama_list(None, &listed).is_none(),
            "pending finish without pick must not take listed[0]"
        );
        assert_eq!(
            resolve_use_ollama_bind(Some("llama3:latest"), None, &listed).as_deref(),
            Some("llama3:latest"),
            "exact pick still binds"
        );
        assert_eq!(
            resolve_bind_after_ollama_list(Some("llama3:latest"), &listed).as_deref(),
            Some("llama3:latest")
        );
        assert_eq!(
            resolve_use_ollama_bind(None, Some("llama3:latest"), &listed).as_deref(),
            Some("llama3:latest"),
            "exact saved bind still works"
        );
        assert!(
            resolve_use_ollama_bind(None, Some("gone:latest"), &listed).is_none(),
            "stale saved bind must not fall through to first row"
        );
    }
}
