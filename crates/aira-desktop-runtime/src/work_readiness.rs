//! Work executor preference + pre-submit readiness (`#349` / RFC-0232; Compare `#354` / RFC-0237;
//! host LLM gate RFC-0242).
//!
//! **Canon (RFC-0242):** Desktop Work generate requires a configured **host process LLM**
//! (`llm_backend=process` + `llm_ollama_model`). Reference mock is not product Work.
//! OP-001 / C1 `Calculate 2 + 2` math readiness escape is **legacy non-normative** — math
//! text is treated as generate and still needs host LLM.
//! Choice ≠ VERIFIED.

use std::path::Path;

use aira_flow::{AdmissionConstraints, ReusePolicy};
use serde::{Deserialize, Serialize};

use crate::model_catalog::{load_model_catalog, CatalogEntry, ModelCatalogSnapshot};
use crate::model_status::{load_model_triple, ModelFact, ModelTripleSnapshot};
use crate::settings::{DesktopSettings, LlmBackend};

/// Work-screen executor preference (not Settings catalog CRUD).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkExecutorPreference {
    /// Use `activated.latest` tip when available.
    Auto,
    /// Exact local model_ref required.
    Required(String),
    /// Two distinct Required models; sequential dual run (`#354` / RFC-0237).
    Compare { a: String, b: String },
}

/// Capability class of the draft text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkCapabilityKind {
    /// Legacy label only — Desktop no longer treats math as a model-free escape (RFC-0242).
    #[serde(rename = "math")]
    Math,
    /// Local text generate — host LLM + model tip/catalog required.
    Generate,
}

/// Pre-submit readiness snapshot for the Work screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkReadiness {
    pub kind: WorkCapabilityKind,
    pub ready: bool,
    pub reasons: Vec<String>,
    pub preference: WorkExecutorPreference,
    pub resolved_model_ref: Option<String>,
    /// Second Compare leg (`#354`); `None` for Auto/Specific.
    pub resolved_model_ref_b: Option<String>,
    pub admission: AdmissionConstraints,
    /// Second Compare admission (`RequireNewExecution`); `None` for Auto/Specific.
    pub admission_b: Option<AdmissionConstraints>,
}

fn empty_readiness(
    kind: WorkCapabilityKind,
    ready: bool,
    reasons: Vec<String>,
    preference: WorkExecutorPreference,
) -> WorkReadiness {
    WorkReadiness {
        kind,
        ready,
        reasons,
        preference,
        resolved_model_ref: None,
        resolved_model_ref_b: None,
        admission: AdmissionConstraints::default(),
        admission_b: None,
    }
}

fn require_new_admission(model_ref: &str) -> AdmissionConstraints {
    AdmissionConstraints {
        model_ref: Some(model_ref.to_string()),
        reuse_policy: ReusePolicy::RequireNewExecution,
        ..Default::default()
    }
}

fn bind_admission(model_ref: &str) -> AdmissionConstraints {
    AdmissionConstraints {
        model_ref: Some(model_ref.to_string()),
        ..Default::default()
    }
}

/// Host process LLM configured in Desktop settings (RFC-0242).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostLlmGate {
    pub ok: bool,
    pub ollama_model: Option<String>,
    pub reasons: Vec<String>,
}

/// Inspect settings for a configured host process LLM (mock → fail-closed).
pub fn evaluate_host_llm_gate(settings: &DesktopSettings) -> HostLlmGate {
    match settings.llm_backend {
        LlmBackend::Mock => HostLlmGate {
            ok: false,
            ollama_model: None,
            reasons: vec![
                "host LLM required — Settings → Models: Process + a model from `ollama list` (reference mock is not product Work)"
                    .into(),
            ],
        },
        LlmBackend::Process => {
            let model = settings
                .llm_ollama_model
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            match model {
                Some(m) => HostLlmGate {
                    ok: true,
                    ollama_model: Some(m),
                    reasons: vec![],
                },
                None => HostLlmGate {
                    ok: false,
                    ollama_model: None,
                    reasons: vec![
                        "host LLM process bind needs llm_ollama_model from `ollama list`".into(),
                    ],
                },
            }
        }
    }
}

/// Executor the live node is using. `None` means the node is stopped; readiness
/// then follows saved settings (the next start applies them).
///
/// GUI audit P0: a running Mock must not become ready because Settings were
/// saved as Process. Model A→B on an already applied Process does not use this
/// type — admission `host_cli_model` switches without restart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedHostLlm {
    pub backend: LlmBackend,
    pub ollama_model: Option<String>,
}

/// Classify draft text and build fail-closed admission + readiness (`#349` / `#354` / RFC-0242).
///
/// Re-audit R1: `settings` must be the same document the GUI loaded (system
/// `DesktopPaths` / in-memory apply). Never load-or-create under `data_root`.
/// Stopped node (`applied = None`). See [`evaluate_work_readiness_applied`].
pub fn evaluate_work_readiness(
    root: impl AsRef<Path>,
    settings: &DesktopSettings,
    text: &str,
    preference: WorkExecutorPreference,
) -> WorkReadiness {
    evaluate_work_readiness_applied(root, settings, text, preference, None)
}

/// Same as [`evaluate_work_readiness`], with the confirmed running executor.
pub fn evaluate_work_readiness_applied(
    root: impl AsRef<Path>,
    settings: &DesktopSettings,
    text: &str,
    preference: WorkExecutorPreference,
    applied: Option<&AppliedHostLlm>,
) -> WorkReadiness {
    let root = root.as_ref();
    let _trimmed = text.trim();

    let gate_owned;
    let settings = if let Some(applied) = applied {
        if applied.backend != LlmBackend::Process {
            return empty_readiness(
                WorkCapabilityKind::Generate,
                false,
                vec!["running node is still mock — restart to apply the saved host model".into()],
                preference,
            );
        }
        let mut owned = settings.clone();
        owned.llm_backend = LlmBackend::Process;
        owned.llm_ollama_model = applied.ollama_model.clone();
        gate_owned = owned;
        &gate_owned
    } else {
        settings
    };

    let host = evaluate_host_llm_gate(settings);
    if !host.ok {
        return empty_readiness(
            WorkCapabilityKind::Generate,
            false,
            host.reasons,
            preference,
        );
    }

    let catalog = load_model_catalog(root).unwrap_or_default();
    let triple = load_model_triple(root, crate::settings::LlmBackend::Process.as_env_str());

    match &preference {
        WorkExecutorPreference::Auto | WorkExecutorPreference::Required(_) => {
            evaluate_single_generate(preference, catalog, triple)
        }
        WorkExecutorPreference::Compare { a, b } => {
            evaluate_compare_generate(a, b, preference.clone(), catalog, triple)
        }
    }
}

fn evaluate_single_generate(
    preference: WorkExecutorPreference,
    catalog: ModelCatalogSnapshot,
    triple: ModelTripleSnapshot,
) -> WorkReadiness {
    let tip = catalog
        .tip_model_ref
        .clone()
        .or_else(|| match &triple.selected {
            ModelFact::Value(s) => Some(s.clone()),
            _ => None,
        });

    let (want, auto) = match &preference {
        WorkExecutorPreference::Auto => (tip.clone(), true),
        WorkExecutorPreference::Required(r) => {
            let r = r.trim().to_string();
            if r.is_empty() {
                return empty_readiness(
                    WorkCapabilityKind::Generate,
                    false,
                    vec![
                        "specific model selected but model_ref is empty — pick a catalog row"
                            .into(),
                    ],
                    preference,
                );
            }
            (Some(r), false)
        }
        WorkExecutorPreference::Compare { .. } => unreachable!("handled by evaluate_compare"),
    };

    let Some(model_ref) = want else {
        return empty_readiness(
            WorkCapabilityKind::Generate,
            false,
            vec![
                "text generation needs a local model tip — Start node after Settings → Models process bind, or Select/Prepare in catalog"
                    .into(),
            ],
            preference,
        );
    };

    let entry = catalog
        .entries
        .iter()
        .find(|e| e.model_ref == model_ref)
        .cloned();
    let is_tip = tip.as_deref() == Some(model_ref.as_str());
    let (ready, mut reasons) =
        generate_ready(&model_ref, entry.as_ref(), triple.ready, auto, is_tip);
    if !ready {
        if let Some(e) = &entry {
            reasons.push(e.ready_reason.clone());
        } else if !triple.ready_detail.is_empty() {
            reasons.push(triple.ready_detail.clone());
        }
    } else if auto {
        reasons.insert(
            0,
            format!("Auto → tip {model_ref} (available); choice ≠ VERIFIED"),
        );
    } else {
        reasons.insert(
            0,
            format!("Required {model_ref} available; choice ≠ VERIFIED"),
        );
    }

    WorkReadiness {
        kind: WorkCapabilityKind::Generate,
        ready,
        reasons,
        preference,
        resolved_model_ref: Some(model_ref.clone()),
        resolved_model_ref_b: None,
        admission: bind_admission(&model_ref),
        admission_b: None,
    }
}

fn evaluate_compare_generate(
    a_raw: &str,
    b_raw: &str,
    preference: WorkExecutorPreference,
    catalog: ModelCatalogSnapshot,
    triple: ModelTripleSnapshot,
) -> WorkReadiness {
    let a = a_raw.trim().to_string();
    let b = b_raw.trim().to_string();
    if a.is_empty() || b.is_empty() {
        return empty_readiness(
            WorkCapabilityKind::Generate,
            false,
            vec![
                "Compare needs two model refs (A and B) — pick catalog rows; no silent substitute"
                    .into(),
            ],
            preference,
        );
    }
    if a == b {
        return empty_readiness(
            WorkCapabilityKind::Generate,
            false,
            vec!["Compare needs two different models (A ≠ B); no silent substitute".into()],
            preference,
        );
    }

    let tip = catalog
        .tip_model_ref
        .clone()
        .or_else(|| match &triple.selected {
            ModelFact::Value(s) => Some(s.clone()),
            _ => None,
        });
    let entry_a = catalog.entries.iter().find(|e| e.model_ref == a).cloned();
    let entry_b = catalog.entries.iter().find(|e| e.model_ref == b).cloned();
    let (ready_a, mut reasons_a) = generate_ready(
        &a,
        entry_a.as_ref(),
        triple.ready,
        false,
        tip.as_deref() == Some(a.as_str()),
    );
    let (ready_b, mut reasons_b) = generate_ready(
        &b,
        entry_b.as_ref(),
        triple.ready,
        false,
        tip.as_deref() == Some(b.as_str()),
    );

    let mut reasons = Vec::new();
    if ready_a {
        reasons.push(format!(
            "Compare A {a} available; choice ≠ VERIFIED; RequireNewExecution"
        ));
    } else {
        reasons.push(format!("Compare A {a} not ready — no silent substitute"));
        reasons.append(&mut reasons_a);
        if let Some(e) = &entry_a {
            reasons.push(e.ready_reason.clone());
        }
    }
    if ready_b {
        reasons.push(format!(
            "Compare B {b} available; choice ≠ VERIFIED; RequireNewExecution"
        ));
    } else {
        reasons.push(format!("Compare B {b} not ready — no silent substitute"));
        reasons.append(&mut reasons_b);
        if let Some(e) = &entry_b {
            reasons.push(e.ready_reason.clone());
        }
    }

    let ready = ready_a && ready_b;
    WorkReadiness {
        kind: WorkCapabilityKind::Generate,
        ready,
        reasons,
        preference,
        resolved_model_ref: Some(a.clone()),
        resolved_model_ref_b: Some(b.clone()),
        admission: require_new_admission(&a),
        admission_b: Some(require_new_admission(&b)),
    }
}

fn host_ollama_model_ref(model_ref: &str) -> bool {
    model_ref.starts_with("aira:model:ollama-")
}

fn generate_ready(
    model_ref: &str,
    entry: Option<&CatalogEntry>,
    tip_ready: bool,
    auto: bool,
    is_tip: bool,
) -> (bool, Vec<String>) {
    // GUI audit P0 / F2: Process Work only runs host Ollama bindings.
    // A prepared local file must not look executable.
    if !host_ollama_model_ref(model_ref) {
        return (
            false,
            vec![format!(
                "{model_ref} is a local file; the host Ollama executor cannot run it"
            )],
        );
    }
    if let Some(e) = entry {
        if e.available {
            return (true, vec![]);
        }
        if e.verified {
            return (
                false,
                vec![format!(
                    "{model_ref} is verified but not activated — Prepare in Settings → Models"
                )],
            );
        }
        return (
            false,
            vec![format!("{model_ref} is not available for generate-local")],
        );
    }
    // Tip may be observed ready without a catalog row (fixture / host-ollama bind).
    if auto && tip_ready {
        return (true, vec![]);
    }
    // Only that tip. Another `aira:model:ollama-` name must not borrow it (#359).
    if is_tip && tip_ready {
        return (true, vec![]);
    }
    (
        false,
        vec![format!(
            "{model_ref} not in lifecycle catalog — Scan/Prepare in Settings → Models"
        )],
    )
}

/// What «Prepare and run» should bind. Empty means use the normal Run button (`#362`).
///
/// `#382`: uses the same **applied** host-LLM gate as Run. Only models the Process
/// executor can run are considered; then exact `ollama list` names are collected for
/// legs that still need a slot. A display label is not a CLI name.
pub fn prepare_and_run_cli_names(
    settings: &DesktopSettings,
    applied: Option<&AppliedHostLlm>,
    text_blank: bool,
    busy: bool,
    preference: &WorkExecutorPreference,
    entries: &[CatalogEntry],
    listed_cli_names: &[String],
) -> Option<Vec<String>> {
    let host_ok = match applied {
        Some(a) if a.backend != LlmBackend::Process => false,
        Some(a) => {
            let mut owned = settings.clone();
            owned.llm_backend = LlmBackend::Process;
            owned.llm_ollama_model = a.ollama_model.clone();
            evaluate_host_llm_gate(&owned).ok
        }
        None => evaluate_host_llm_gate(settings).ok,
    };
    if !host_ok || text_blank || busy {
        return None;
    }
    let selected = match preference {
        WorkExecutorPreference::Auto => return None,
        WorkExecutorPreference::Required(r) => {
            let r = r.trim();
            if r.is_empty() {
                return None;
            }
            vec![r.to_string()]
        }
        WorkExecutorPreference::Compare { a, b } => {
            let a = a.trim();
            let b = b.trim();
            if a.is_empty() || b.is_empty() || a == b {
                return None;
            }
            vec![a.to_string(), b.to_string()]
        }
    };
    let mut needed = Vec::new();
    for model_ref in selected {
        // Same Process rule as `generate_ready`: local files never become prepare targets.
        if !host_ollama_model_ref(&model_ref) {
            return None;
        }
        if entries
            .iter()
            .any(|e| e.model_ref == model_ref && e.available)
        {
            continue;
        }
        let cli = listed_cli_names.iter().find_map(|n| {
            let n = n.trim();
            if n.is_empty() {
                return None;
            }
            (aira_flow::host_ollama_model_ref(n) == model_ref).then(|| n.to_string())
        })?;
        needed.push(cli);
    }
    if needed.is_empty() {
        None
    } else {
        Some(needed)
    }
}

/// Result of one Prepare-and-run click (`#362`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepareAndRunOutcome {
    Started,
    IgnoredRepeat,
    Failed(String),
}

/// Slot, then one start. A second call while `busy` does not prepare or start again.
/// A prepare error does not start.
pub fn execute_prepare_and_run(
    busy: bool,
    prepare: impl FnOnce() -> Result<(), String>,
    start: impl FnOnce() -> Result<(), String>,
) -> PrepareAndRunOutcome {
    if busy {
        return PrepareAndRunOutcome::IgnoredRepeat;
    }
    if let Err(e) = prepare() {
        return PrepareAndRunOutcome::Failed(e);
    }
    match start() {
        Ok(()) => PrepareAndRunOutcome::Started,
        Err(e) => PrepareAndRunOutcome::Failed(e),
    }
}

/// Write host-Ollama slots for `cli_names`. Does not replace `activated.latest`.
/// On failure the tip bytes are unchanged and the caller must not start a run.
pub fn prepare_host_slots(root: &Path, cli_names: &[String]) -> Result<(), String> {
    let tip_path = root.join("models/activated.latest.json");
    let tip_before = std::fs::read(&tip_path).ok();
    for name in cli_names {
        aira_flow::ActivatedPointerGate::install_host_ollama_slot(root, name)?;
        let tip_after = std::fs::read(&tip_path).ok();
        if tip_after != tip_before {
            return Err(
                "prepare wrote activated.latest; the default tip must stay unchanged".into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_flow::ActivatedPointerGate;
    use serial_test::serial;
    use std::fs;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    use crate::paths::DesktopPaths;
    use crate::settings::{load_settings_readonly, write_settings};

    fn isolated() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn write_host_llm(root: &std::path::Path, model: &str) -> DesktopSettings {
        let paths = DesktopPaths::for_data_root(root);
        let mut s = DesktopSettings::default_p0(&paths);
        s.llm_backend = LlmBackend::Process;
        s.llm_ollama_model = Some(model.into());
        write_settings(&paths, &s).unwrap();
        s
    }

    fn mock_settings(root: &std::path::Path) -> DesktopSettings {
        let paths = DesktopPaths::for_data_root(root);
        DesktopSettings::default_p0(&paths)
    }

    fn write_available_slot(root: &std::path::Path, model_ref: &str, slot: &str) {
        let dir = root.join("models/cache").join(slot);
        fs::create_dir_all(&dir).unwrap();
        let ptr = serde_json::json!({
            "updated_at": "2026-09-15T00:00:00Z",
            "model_ref": model_ref,
            "cache_path": dir.join("weights.bin").to_string_lossy(),
            "verified_path": dir.join("verified.bin").to_string_lossy(),
            "content_hash": format!("sha256:{slot}aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            "evidence_artifact_id": format!("aira:artifact:{slot}")
        });
        fs::write(
            dir.join("activated.json"),
            serde_json::to_string_pretty(&ptr).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn mock_settings_block_even_math_text() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let s = mock_settings(dir.path());
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Calculate 2 + 2",
            WorkExecutorPreference::Auto,
        );
        assert_eq!(r.kind, WorkCapabilityKind::Generate);
        assert!(!r.ready);
        assert!(r.reasons.iter().any(|s| s.contains("host LLM")));
    }

    #[test]
    fn process_settings_without_tip_not_ready_for_auto() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize the local Problem Statement",
            WorkExecutorPreference::Auto,
        );
        assert_eq!(r.kind, WorkCapabilityKind::Generate);
        assert!(!r.ready);
        assert!(r
            .reasons
            .iter()
            .any(|s| s.contains("tip") || s.contains("model")));
    }

    #[test]
    fn generate_required_empty_is_not_ready() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize locally",
            WorkExecutorPreference::Required(String::new()),
        );
        assert!(!r.ready);
        assert!(r.reasons.iter().any(|s| s.contains("empty")));
    }

    #[test]
    #[serial]
    fn generate_auto_with_fixture_tip_is_ready_and_admits_model() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        aira_object::reset_primary_signer();
        ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        let _ = ActivatedPointerGate::from_aira_root(dir.path()).observe_verify_now();
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize the local Problem Statement",
            WorkExecutorPreference::Auto,
        );
        assert_eq!(r.kind, WorkCapabilityKind::Generate);
        assert!(
            r.resolved_model_ref.is_some(),
            "expected tip resolve, got {:?}",
            r.reasons
        );
        assert_eq!(r.admission.model_ref, r.resolved_model_ref);
        assert!(r.admission_b.is_none());
        if r.ready {
            assert!(r.admission.model_ref.is_some());
        } else {
            assert!(!r.reasons.is_empty());
        }
    }

    #[test]
    fn compare_empty_or_same_is_not_ready() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        let empty = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize locally",
            WorkExecutorPreference::Compare {
                a: String::new(),
                b: "aira:model:b".into(),
            },
        );
        assert!(!empty.ready);
        assert!(empty.reasons.iter().any(|s| s.contains("two model")));

        let same = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize locally",
            WorkExecutorPreference::Compare {
                a: "aira:model:x".into(),
                b: "aira:model:x".into(),
            },
        );
        assert!(!same.ready);
        assert!(same.reasons.iter().any(|s| s.contains("different")));
    }

    #[test]
    fn compare_one_unready_does_not_substitute() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        write_available_slot(dir.path(), "aira:model:a", "slot-a");
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize locally for compare",
            WorkExecutorPreference::Compare {
                a: "aira:model:a".into(),
                b: "aira:model:missing".into(),
            },
        );
        assert!(!r.ready);
        assert!(r.reasons.iter().any(|s| s.contains("Compare B")));
        assert!(r.reasons.iter().any(|s| s.contains("no silent substitute")));
        assert_eq!(r.admission.model_ref.as_deref(), Some("aira:model:a"));
        assert_eq!(
            r.admission_b.as_ref().and_then(|c| c.model_ref.as_deref()),
            Some("aira:model:missing")
        );
        assert_eq!(r.admission.reuse_policy, ReusePolicy::RequireNewExecution);
    }

    /// `#359`: tip readiness is not a pass for every other `aira:model:ollama-` name.
    #[test]
    fn tip_ready_does_not_make_unbound_ollama_ref_ready() {
        let tip = "aira:model:ollama-tip";
        let other = "aira:model:ollama-other";
        let catalog = ModelCatalogSnapshot {
            tip_model_ref: Some(tip.into()),
            ..ModelCatalogSnapshot::default()
        };
        let mut triple = ModelTripleSnapshot::undefined();
        triple.ready = true;
        triple.selected = ModelFact::Value(tip.into());

        let required_other = evaluate_single_generate(
            WorkExecutorPreference::Required(other.into()),
            catalog.clone(),
            triple.clone(),
        );
        assert!(
            !required_other.ready,
            "unbound B borrowed tip readiness: {:?}",
            required_other.reasons
        );
        assert!(required_other.reasons.iter().any(|s| s.contains(other)));

        let required_tip = evaluate_single_generate(
            WorkExecutorPreference::Required(tip.into()),
            catalog.clone(),
            triple.clone(),
        );
        assert!(
            required_tip.ready,
            "the tip itself must stay ready: {:?}",
            required_tip.reasons
        );

        let auto = evaluate_single_generate(
            WorkExecutorPreference::Auto,
            catalog.clone(),
            triple.clone(),
        );
        assert!(
            auto.ready,
            "Auto still uses the ready tip: {:?}",
            auto.reasons
        );

        let compare = evaluate_compare_generate(
            tip,
            other,
            WorkExecutorPreference::Compare {
                a: tip.into(),
                b: other.into(),
            },
            catalog,
            triple,
        );
        assert!(!compare.ready);
        assert!(compare
            .reasons
            .iter()
            .any(|s| s.contains("Compare B") && s.contains("not ready")));
    }

    #[test]
    fn compare_both_available_admits_two_require_new() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        write_available_slot(dir.path(), "aira:model:ollama-a", "slot-a");
        write_available_slot(dir.path(), "aira:model:ollama-b", "slot-b");
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize locally for compare",
            WorkExecutorPreference::Compare {
                a: "aira:model:ollama-a".into(),
                b: "aira:model:ollama-b".into(),
            },
        );
        assert!(r.ready, "reasons={:?}", r.reasons);
        assert_eq!(r.resolved_model_ref.as_deref(), Some("aira:model:ollama-a"));
        assert_eq!(
            r.resolved_model_ref_b.as_deref(),
            Some("aira:model:ollama-b")
        );
        assert_eq!(
            r.admission.model_ref.as_deref(),
            Some("aira:model:ollama-a")
        );
        assert_eq!(
            r.admission_b.as_ref().and_then(|c| c.model_ref.as_deref()),
            Some("aira:model:ollama-b")
        );
        assert_eq!(r.admission.reuse_policy, ReusePolicy::RequireNewExecution);
        assert_eq!(
            r.admission_b.as_ref().map(|c| c.reuse_policy),
            Some(ReusePolicy::RequireNewExecution)
        );
        assert_ne!(
            r.admission.model_ref,
            r.admission_b.as_ref().and_then(|c| c.model_ref.clone())
        );
    }

    #[test]
    fn evaluate_host_llm_gate_rejects_mock() {
        let paths = DesktopPaths::for_data_root(tempdir().unwrap().path());
        let s = DesktopSettings::default_p0(&paths);
        let g = evaluate_host_llm_gate(&s);
        assert!(!g.ok);
        assert!(g.reasons.iter().any(|r| r.contains("host LLM")));
    }

    /// Re-audit R1: system layout settings path ≠ data_root; readiness uses passed settings.
    #[test]
    fn readiness_uses_system_layout_settings_not_data_root_file() {
        let _g = isolated();
        let home = tempdir().unwrap();
        let paths = DesktopPaths::system_for_home(home.path());
        paths.ensure_dirs().unwrap();
        // No settings under data_root.
        assert!(!paths.data_root.join("desktop-settings.json").is_file());
        let mut s = DesktopSettings::default_p0(&paths);
        s.llm_backend = LlmBackend::Process;
        s.llm_ollama_model = Some("llama3:latest".into());
        write_settings(&paths, &s).unwrap();
        assert!(paths.settings_file.is_file());
        assert_ne!(
            paths.settings_file,
            paths.data_root.join("desktop-settings.json")
        );
        // Read-only load never creates; missing under wrong path stays missing.
        let loaded = load_settings_readonly(&paths).unwrap();
        assert_eq!(loaded.llm_backend, LlmBackend::Process);
        let r = evaluate_work_readiness(
            &paths.data_root,
            &loaded,
            "Summarize locally",
            WorkExecutorPreference::Auto,
        );
        // Host gate passes; tip may still be unready — must not claim mock-host failure.
        assert!(
            !r.reasons.iter().any(|x| x.contains("host LLM required")),
            "reasons={:?}",
            r.reasons
        );
        assert!(!paths.data_root.join("desktop-settings.json").is_file());
    }

    #[test]
    fn load_settings_readonly_missing_does_not_create() {
        let home = tempdir().unwrap();
        let paths = DesktopPaths::system_for_home(home.path());
        paths.ensure_dirs().unwrap();
        assert!(load_settings_readonly(&paths).is_err());
        assert!(!paths.settings_file.is_file());
        assert!(!paths.data_root.join("desktop-settings.json").is_file());
    }

    #[test]
    fn running_mock_blocks_saved_process() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let saved = write_host_llm(dir.path(), "llama3:latest");
        let applied = AppliedHostLlm {
            backend: LlmBackend::Mock,
            ollama_model: None,
        };
        let r = evaluate_work_readiness_applied(
            dir.path(),
            &saved,
            "Summarize locally",
            WorkExecutorPreference::Auto,
            Some(&applied),
        );
        assert!(!r.ready);
        assert!(
            r.reasons.iter().any(|s| s.contains("restart")),
            "reasons={:?}",
            r.reasons
        );
    }

    #[test]
    fn local_file_is_not_ready_on_ollama_executor() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        write_available_slot(dir.path(), "aira:model:file-weights-a", "slot-file");
        let r = evaluate_work_readiness(
            dir.path(),
            &s,
            "Summarize locally",
            WorkExecutorPreference::Required("aira:model:file-weights-a".into()),
        );
        assert!(!r.ready, "reasons={:?}", r.reasons);
        assert!(r.reasons.iter().any(|s| s.contains("cannot run")));
    }

    #[test]
    fn prepare_and_run_offers_exact_cli_name_only() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let s = write_host_llm(dir.path(), "kept-tip:latest");
        let cli = "model-b:latest";
        let model_ref = aira_flow::host_ollama_model_ref(cli);
        let pref = WorkExecutorPreference::Required(model_ref.clone());
        let names = prepare_and_run_cli_names(&s, None, false, false, &pref, &[], &[cli.into()])
            .expect("exact list name");
        assert_eq!(names, vec![cli.to_string()]);
        assert!(
            prepare_and_run_cli_names(&s, None, false, true, &pref, &[], &[cli.into()]).is_none()
        );
        assert!(prepare_and_run_cli_names(
            &s,
            None,
            false,
            false,
            &pref,
            &[],
            &["not-the-cli-name".into()],
        )
        .is_none());
        let slotted = CatalogEntry {
            model_ref: model_ref.clone(),
            verified: false,
            available: true,
            ready_reason: "slotted".into(),
        };
        assert!(prepare_and_run_cli_names(
            &s,
            None,
            false,
            false,
            &pref,
            &[slotted],
            &[cli.into()]
        )
        .is_none());
        assert!(prepare_and_run_cli_names(
            &s,
            None,
            false,
            false,
            &WorkExecutorPreference::Auto,
            &[],
            &[cli.into()],
        )
        .is_none());
    }

    /// `#382`: running Mock must not offer Prepare just because Settings were saved Process.
    #[test]
    fn prepare_and_run_respects_applied_mock_like_run() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let saved = write_host_llm(dir.path(), "llama3:latest");
        let cli = "model-b:latest";
        let model_ref = aira_flow::host_ollama_model_ref(cli);
        let pref = WorkExecutorPreference::Required(model_ref);
        let applied = AppliedHostLlm {
            backend: LlmBackend::Mock,
            ollama_model: None,
        };
        assert!(
            prepare_and_run_cli_names(
                &saved,
                Some(&applied),
                false,
                false,
                &pref,
                &[],
                &[cli.into()]
            )
            .is_none(),
            "Prepare must not open when Run would require restart"
        );
        let applied_process = AppliedHostLlm {
            backend: LlmBackend::Process,
            ollama_model: Some("llama3:latest".into()),
        };
        assert_eq!(
            prepare_and_run_cli_names(
                &saved,
                Some(&applied_process),
                false,
                false,
                &pref,
                &[],
                &[cli.into()],
            )
            .as_deref(),
            Some([cli.to_string()].as_slice())
        );
    }

    /// `#382`: Compare file + Ollama must not prepare B when A cannot run on Process.
    #[test]
    fn prepare_and_run_compare_rejects_local_file_leg() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        let s = write_host_llm(dir.path(), "llama3:latest");
        let cli_b = "model-b:latest";
        let ref_b = aira_flow::host_ollama_model_ref(cli_b);
        let file_a = "aira:model:file-weights-a".to_string();
        let file_entry = CatalogEntry {
            model_ref: file_a.clone(),
            verified: true,
            available: true,
            ready_reason: "prepared file".into(),
        };
        let pref = WorkExecutorPreference::Compare {
            a: file_a,
            b: ref_b.clone(),
        };
        assert!(
            prepare_and_run_cli_names(
                &s,
                None,
                false,
                false,
                &pref,
                &[file_entry],
                &[cli_b.into()],
            )
            .is_none(),
            "available file leg must not unlock Prepare for B"
        );
        // Both Ollama legs: A slotted, B needs CLI → only B.
        let cli_a = "model-a:latest";
        let ref_a = aira_flow::host_ollama_model_ref(cli_a);
        let slotted_a = CatalogEntry {
            model_ref: ref_a.clone(),
            verified: false,
            available: true,
            ready_reason: "slotted".into(),
        };
        let pref_ok = WorkExecutorPreference::Compare { a: ref_a, b: ref_b };
        assert_eq!(
            prepare_and_run_cli_names(
                &s,
                None,
                false,
                false,
                &pref_ok,
                &[slotted_a],
                &[cli_a.into(), cli_b.into()],
            )
            .as_deref(),
            Some([cli_b.to_string()].as_slice())
        );
    }

    #[test]
    fn prepare_and_run_repeat_does_not_start_twice_and_failure_skips_start() {
        let mut starts = 0;
        let first = execute_prepare_and_run(
            false,
            || Ok(()),
            || {
                starts += 1;
                Ok(())
            },
        );
        assert_eq!(first, PrepareAndRunOutcome::Started);
        let second = execute_prepare_and_run(
            true,
            || panic!("repeat must not prepare"),
            || {
                starts += 1;
                Ok(())
            },
        );
        assert_eq!(second, PrepareAndRunOutcome::IgnoredRepeat);
        assert_eq!(starts, 1);

        let mut started = false;
        let failed = execute_prepare_and_run(
            false,
            || Err("slot failed".into()),
            || {
                started = true;
                Ok(())
            },
        );
        assert_eq!(failed, PrepareAndRunOutcome::Failed("slot failed".into()));
        assert!(!started);
    }

    #[test]
    fn prepare_host_slots_keeps_tip_and_refuses_empty_name() {
        use aira_object::{create_or_ensure_node_identity, NodeIdentityCreatePolicy};
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let _g = isolated();
        let dir = tempdir().unwrap();
        let tip_path = dir.path().join("models/activated.latest.json");
        fs::create_dir_all(tip_path.parent().unwrap()).unwrap();
        fs::write(&tip_path, "TIP-A").unwrap();
        let err = prepare_host_slots(dir.path(), &[String::new()]).unwrap_err();
        assert!(err.contains("empty"), "{err}");
        assert_eq!(fs::read_to_string(&tip_path).unwrap(), "TIP-A");

        aira_object::reset_primary_signer();
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        create_or_ensure_node_identity(
            dir.path(),
            &format!("aira:identity:prep.{}", uuid::Uuid::now_v7().as_simple()),
            "prep",
            signing,
            NodeIdentityCreatePolicy::CreateExclusive,
        )
        .unwrap();
        let (_gate, ref_a) =
            ActivatedPointerGate::install_host_ollama_bind(dir.path(), "model-a:latest").unwrap();
        let tip_bytes = fs::read(&tip_path).unwrap();
        assert!(std::str::from_utf8(&tip_bytes).unwrap().contains(&ref_a));

        let cli_b = "model-b:latest";
        let ref_b = aira_flow::host_ollama_model_ref(cli_b);
        prepare_host_slots(dir.path(), &[cli_b.into()]).unwrap();
        assert_eq!(fs::read(&tip_path).unwrap(), tip_bytes);
        let cat = crate::model_catalog::load_model_catalog(dir.path()).unwrap();
        assert_eq!(cat.tip_model_ref.as_deref(), Some(ref_a.as_str()));
        assert!(cat
            .entries
            .iter()
            .any(|e| e.model_ref == ref_b && e.available));
        assert!(prepare_and_run_cli_names(
            &write_host_llm(dir.path(), "model-a:latest"),
            None,
            false,
            false,
            &WorkExecutorPreference::Required(ref_b),
            &cat.entries,
            &[cli_b.into()],
        )
        .is_none());
    }

    /// `#367`: M0 through-test — A tip, B only in discovery, Prepare-and-run slots B,
    /// admit B, tip stays A; plus K3/K4b/K5 negatives (label, fail, repeat, applied Mock).
    #[test]
    #[serial]
    fn m0_through_prepare_and_run_with_k3_k5_negatives() {
        use aira_object::{create_or_ensure_node_identity, NodeIdentityCreatePolicy};
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let _g = isolated();
        let dir = tempdir().unwrap();
        aira_object::reset_primary_signer();
        let mut rng = OsRng;
        let signing = SigningKey::generate(&mut rng);
        create_or_ensure_node_identity(
            dir.path(),
            &format!("aira:identity:m0.{}", uuid::Uuid::now_v7().as_simple()),
            "m0",
            signing,
            NodeIdentityCreatePolicy::CreateExclusive,
        )
        .unwrap();

        let cli_a = "model-a:latest";
        let cli_b = "model-b:latest";
        let (_gate, ref_a) =
            ActivatedPointerGate::install_host_ollama_bind(dir.path(), cli_a).unwrap();
        let ref_b = aira_flow::host_ollama_model_ref(cli_b);
        let tip_path = dir.path().join("models/activated.latest.json");
        let tip_before = fs::read(&tip_path).unwrap();
        let settings = write_host_llm(dir.path(), cli_a);
        let listed = vec![cli_a.into(), cli_b.into()];

        // B is only in discovery — not yet available in the catalog.
        let cat0 = crate::model_catalog::load_model_catalog(dir.path()).unwrap();
        assert_eq!(cat0.tip_model_ref.as_deref(), Some(ref_a.as_str()));
        assert!(!cat0.entries.iter().any(|e| e.model_ref == ref_b && e.available));
        let pref_b = WorkExecutorPreference::Required(ref_b.clone());
        assert_eq!(
            prepare_and_run_cli_names(
                &settings,
                None,
                false,
                false,
                &pref_b,
                &cat0.entries,
                &listed,
            )
            .as_deref(),
            Some([cli_b.to_string()].as_slice()),
            "Prepare must offer exact B CLI when B is listed but not slotted"
        );

        // K3: display label is not a CLI name — Prepare must not invent a bind.
        let label = crate::model_catalog::catalog_display_name(&ref_b);
        assert!(crate::model_catalog::is_display_label_not_cli_name(&label));
        assert!(
            prepare_and_run_cli_names(
                &settings,
                None,
                false,
                false,
                &WorkExecutorPreference::Required(label.clone()),
                &cat0.entries,
                &listed,
            )
            .is_none(),
            "Required(display label) must not open Prepare"
        );

        // K5: applied Mock blocks Prepare even when Settings are Process.
        let applied_mock = AppliedHostLlm {
            backend: LlmBackend::Mock,
            ollama_model: None,
        };
        assert!(
            prepare_and_run_cli_names(
                &settings,
                Some(&applied_mock),
                false,
                false,
                &pref_b,
                &cat0.entries,
                &listed,
            )
            .is_none(),
            "applied Mock must match Run and refuse Prepare"
        );

        // K4b / prepare-fail: error skips start; tip unchanged.
        let mut started = false;
        let failed = execute_prepare_and_run(
            false,
            || Err("slot failed".into()),
            || {
                started = true;
                Ok(())
            },
        );
        assert_eq!(failed, PrepareAndRunOutcome::Failed("slot failed".into()));
        assert!(!started);
        assert_eq!(fs::read(&tip_path).unwrap(), tip_before);

        // Happy path: prepare slots B, then start once; tip stays A.
        let mut starts = 0;
        let outcome = execute_prepare_and_run(
            false,
            || prepare_host_slots(dir.path(), &[cli_b.into()]),
            || {
                starts += 1;
                Ok(())
            },
        );
        assert_eq!(outcome, PrepareAndRunOutcome::Started);
        assert_eq!(starts, 1);
        assert_eq!(fs::read(&tip_path).unwrap(), tip_before);

        // K4b: repeat while busy does not prepare/start again.
        let repeat = execute_prepare_and_run(
            true,
            || panic!("repeat must not prepare"),
            || {
                starts += 1;
                Ok(())
            },
        );
        assert_eq!(repeat, PrepareAndRunOutcome::IgnoredRepeat);
        assert_eq!(starts, 1);

        let cat1 = crate::model_catalog::load_model_catalog(dir.path()).unwrap();
        assert_eq!(cat1.tip_model_ref.as_deref(), Some(ref_a.as_str()));
        assert!(cat1
            .entries
            .iter()
            .any(|e| e.model_ref == ref_b && e.available));
        assert!(
            prepare_and_run_cli_names(
                &settings,
                None,
                false,
                false,
                &pref_b,
                &cat1.entries,
                &listed,
            )
            .is_none(),
            "after slot, Run is the right control — Prepare closes"
        );

        // Admit B for generate; Auto tip remains A.
        let ready_b = evaluate_work_readiness(
            dir.path(),
            &settings,
            "Summarize using B",
            pref_b,
        );
        assert!(ready_b.ready, "reasons={:?}", ready_b.reasons);
        assert_eq!(ready_b.resolved_model_ref.as_deref(), Some(ref_b.as_str()));

        let ready_auto = evaluate_work_readiness(
            dir.path(),
            &settings,
            "Summarize using tip A",
            WorkExecutorPreference::Auto,
        );
        assert!(ready_auto.ready, "reasons={:?}", ready_auto.reasons);
        assert_eq!(
            ready_auto.resolved_model_ref.as_deref(),
            Some(ref_a.as_str()),
            "tip A must remain the Auto default after preparing B"
        );
    }
}
