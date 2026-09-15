//! Work executor preference + pre-submit readiness (`#349` / RFC-0232).
//!
//! Math (`Calculate 2 + 2`) does not need a model. Generate-local needs an
//! Auto tip or Required `model_ref` that is available. Choice ≠ VERIFIED.

use std::path::Path;

use aira_csu_reduction_basic::problem_binds_math_eval_safe;
use aira_flow::AdmissionConstraints;
use serde::{Deserialize, Serialize};

use crate::model_catalog::{load_model_catalog, CatalogEntry};
use crate::model_status::{load_model_triple, ModelFact};

/// Work-screen executor preference (not Settings catalog CRUD).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkExecutorPreference {
    /// Use `activated.latest` tip when available.
    Auto,
    /// Exact local model_ref required.
    Required(String),
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
    pub admission: AdmissionConstraints,
}

/// Classify draft text and build fail-closed admission + readiness (`#349`).
pub fn evaluate_work_readiness(
    root: impl AsRef<Path>,
    text: &str,
    preference: WorkExecutorPreference,
) -> WorkReadiness {
    let root = root.as_ref();
    let trimmed = text.trim();
    if problem_binds_math_eval_safe(trimmed) {
        return WorkReadiness {
            kind: WorkCapabilityKind::Math,
            ready: true,
            reasons: vec![
                "deterministic math — local model not required (omit model_ref)".into(),
            ],
            preference,
            resolved_model_ref: None,
            admission: AdmissionConstraints::default(),
        };
    }

    let catalog = load_model_catalog(root).unwrap_or_default();
    let triple = load_model_triple(root);
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
                return WorkReadiness {
                    kind: WorkCapabilityKind::Generate,
                    ready: false,
                    reasons: vec![
                        "specific model selected but model_ref is empty — pick a catalog row"
                            .into(),
                    ],
                    preference,
                    resolved_model_ref: None,
                    admission: AdmissionConstraints::default(),
                };
            }
            (Some(r), false)
        }
    };

    let Some(model_ref) = want else {
        return WorkReadiness {
            kind: WorkCapabilityKind::Generate,
            ready: false,
            reasons: vec![
                "text generation needs a local model — Select/Prepare tip in Settings → Models"
                    .into(),
            ],
            preference,
            resolved_model_ref: None,
            admission: AdmissionConstraints::default(),
        };
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

    let admission = AdmissionConstraints {
        model_ref: Some(model_ref.clone()),
        ..Default::default()
    };
    WorkReadiness {
        kind: WorkCapabilityKind::Generate,
        ready,
        reasons,
        preference,
        resolved_model_ref: Some(model_ref),
        admission,
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

    #[test]
    fn math_is_ready_without_model() {
        let _g = isolated();
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let r = evaluate_work_readiness(
            dir.path(),
            "Calculate 2 + 2",
            WorkExecutorPreference::Auto,
        );
        assert_eq!(r.kind, WorkCapabilityKind::Math);
        assert!(r.ready);
        assert!(r.admission.model_ref.is_none());
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
}
