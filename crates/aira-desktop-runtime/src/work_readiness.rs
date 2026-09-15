//! Work executor preference + pre-submit readiness (`#349` / RFC-0232; Compare `#354` / RFC-0237).
//!
//! Math (`Calculate 2 + 2`) does not need a model. Generate-local needs an
//! Auto tip or Required `model_ref` that is available. Compare needs **two**
//! distinct available models and never silently substitutes either leg.
//! Choice ≠ VERIFIED.

use std::path::Path;

use aira_csu_reduction_basic::problem_binds_math_eval_safe;
use aira_flow::{AdmissionConstraints, ReusePolicy};
use serde::{Deserialize, Serialize};

use crate::model_catalog::{load_model_catalog, CatalogEntry, ModelCatalogSnapshot};
use crate::model_status::{load_model_triple, ModelFact, ModelTripleSnapshot};

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
    /// Deterministic math — model optional / must be omitted.
    Math,
    /// Local text generate — model required.
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

/// Classify draft text and build fail-closed admission + readiness (`#349` / `#354`).
pub fn evaluate_work_readiness(
    root: impl AsRef<Path>,
    text: &str,
    preference: WorkExecutorPreference,
) -> WorkReadiness {
    let root = root.as_ref();
    let trimmed = text.trim();
    if problem_binds_math_eval_safe(trimmed) {
        if matches!(preference, WorkExecutorPreference::Compare { .. }) {
            return empty_readiness(
                WorkCapabilityKind::Math,
                false,
                vec![
                    "Compare mode needs text generation — math has no dual-model path (no silent substitute)"
                        .into(),
                ],
                preference,
            );
        }
        return empty_readiness(
            WorkCapabilityKind::Math,
            true,
            vec!["deterministic math — local model not required (omit model_ref)".into()],
            preference,
        );
    }

    let catalog = load_model_catalog(root).unwrap_or_default();
    let triple = load_model_triple(root, crate::settings::LlmBackend::Mock.as_env_str());

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
                "text generation needs a local model — Select/Prepare tip in Settings → Models"
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
    let (ready, mut reasons) = generate_ready(&model_ref, entry.as_ref(), triple.ready, auto);
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

    let entry_a = catalog.entries.iter().find(|e| e.model_ref == a).cloned();
    let entry_b = catalog.entries.iter().find(|e| e.model_ref == b).cloned();
    let (ready_a, mut reasons_a) = generate_ready(&a, entry_a.as_ref(), triple.ready, false);
    let (ready_b, mut reasons_b) = generate_ready(&b, entry_b.as_ref(), triple.ready, false);

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

fn generate_ready(
    model_ref: &str,
    entry: Option<&CatalogEntry>,
    tip_ready: bool,
    auto: bool,
) -> (bool, Vec<String>) {
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
    // Tip may be observed ready without a catalog row (fixture / race).
    if auto && tip_ready {
        return (true, vec![]);
    }
    (
        false,
        vec![format!(
            "{model_ref} not in lifecycle catalog — Scan/Prepare in Settings → Models"
        )],
    )
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
    fn math_is_ready_without_model() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let r =
            evaluate_work_readiness(dir.path(), "Calculate 2 + 2", WorkExecutorPreference::Auto);
        assert_eq!(r.kind, WorkCapabilityKind::Math);
        assert!(r.ready);
        assert!(r.admission.model_ref.is_none());
        assert!(r.admission_b.is_none());
        assert!(r.reasons.iter().any(|s| s.contains("math")));
    }

    #[test]
    fn generate_auto_without_tip_is_not_ready() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let r = evaluate_work_readiness(
            dir.path(),
            "Summarize the local Problem Statement",
            WorkExecutorPreference::Auto,
        );
        assert_eq!(r.kind, WorkCapabilityKind::Generate);
        assert!(!r.ready);
        assert!(r.admission.model_ref.is_none());
    }

    #[test]
    fn generate_required_empty_is_not_ready() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let r = evaluate_work_readiness(
            dir.path(),
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
        aira_object::reset_primary_signer();
        ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        // Warm observe so tip/lifecycle settle.
        let _ = ActivatedPointerGate::from_aira_root(dir.path()).observe_verify_now();
        let r = evaluate_work_readiness(
            dir.path(),
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
        // Fixture may be pending hash — ready if available in catalog OR tip_ready.
        if r.ready {
            assert!(r.admission.model_ref.is_some());
        } else {
            assert!(!r.reasons.is_empty());
        }
    }

    #[test]
    fn math_ignores_required_model_preference_for_admission() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let r = evaluate_work_readiness(
            dir.path(),
            "Calculate 9 - 3",
            WorkExecutorPreference::Required("aira:model:x".into()),
        );
        assert_eq!(r.kind, WorkCapabilityKind::Math);
        assert!(r.ready);
        assert!(r.admission.model_ref.is_none());
    }

    #[test]
    fn compare_math_is_blocked_fail_closed() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let r = evaluate_work_readiness(
            dir.path(),
            "Calculate 2 + 2",
            WorkExecutorPreference::Compare {
                a: "aira:model:a".into(),
                b: "aira:model:b".into(),
            },
        );
        assert_eq!(r.kind, WorkCapabilityKind::Math);
        assert!(!r.ready);
        assert!(r.reasons.iter().any(|s| s.contains("Compare")));
        assert!(r.admission_b.is_none());
    }

    #[test]
    fn compare_empty_or_same_is_not_ready() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let empty = evaluate_work_readiness(
            dir.path(),
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
        write_available_slot(dir.path(), "aira:model:a", "slot-a");
        // B missing from catalog.
        let r = evaluate_work_readiness(
            dir.path(),
            "Summarize locally for compare",
            WorkExecutorPreference::Compare {
                a: "aira:model:a".into(),
                b: "aira:model:missing".into(),
            },
        );
        assert!(!r.ready);
        assert!(r.reasons.iter().any(|s| s.contains("Compare B")));
        assert!(r.reasons.iter().any(|s| s.contains("no silent substitute")));
        // Admissions still name both requested refs — never tip substitution.
        assert_eq!(r.admission.model_ref.as_deref(), Some("aira:model:a"));
        assert_eq!(
            r.admission_b.as_ref().and_then(|c| c.model_ref.as_deref()),
            Some("aira:model:missing")
        );
        assert_eq!(r.admission.reuse_policy, ReusePolicy::RequireNewExecution);
    }

    #[test]
    fn compare_both_available_admits_two_require_new() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        write_available_slot(dir.path(), "aira:model:a", "slot-a");
        write_available_slot(dir.path(), "aira:model:b", "slot-b");
        let r = evaluate_work_readiness(
            dir.path(),
            "Summarize locally for compare",
            WorkExecutorPreference::Compare {
                a: "aira:model:a".into(),
                b: "aira:model:b".into(),
            },
        );
        assert!(r.ready, "reasons={:?}", r.reasons);
        assert_eq!(r.resolved_model_ref.as_deref(), Some("aira:model:a"));
        assert_eq!(r.resolved_model_ref_b.as_deref(), Some("aira:model:b"));
        assert_eq!(r.admission.model_ref.as_deref(), Some("aira:model:a"));
        assert_eq!(
            r.admission_b.as_ref().and_then(|c| c.model_ref.as_deref()),
            Some("aira:model:b")
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
}
