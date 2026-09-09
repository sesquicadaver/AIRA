//! Model selected ≠ ready ≠ used (`#269`).

use std::path::Path;

use aira_flow::{ActivatedPointerGate, ActivationObservation};

/// One slot of the model triple (never invent a name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelFact {
    /// No observation yet / Explicitly Undefined.
    Undefined,
    /// Confirmed absence (e.g. no activated pointer).
    None,
    /// Observed model identity (model_ref / artifact ref / content hash — never a backend id; `#283`).
    Value(String),
}

impl ModelFact {
    pub fn as_display(&self) -> &str {
        match self {
            Self::Undefined => "undefined",
            Self::None => "none",
            Self::Value(s) => s.as_str(),
        }
    }

    pub fn from_optional_ref(v: Option<String>) -> Self {
        match v {
            Some(s) if !s.trim().is_empty() => Self::Value(s),
            Some(_) => Self::None,
            None => Self::None,
        }
    }
}

/// Desktop projection of model monitoring facts (`#269`).
///
/// `used` is filled by the GUI from the last Work result — never copied from selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelTripleSnapshot {
    pub selected: ModelFact,
    pub ready: bool,
    pub ready_detail: String,
    pub used: ModelFact,
    pub activation: ActivationObservation,
}

impl ModelTripleSnapshot {
    /// Empty / unobserved triple.
    pub fn undefined() -> Self {
        Self {
            selected: ModelFact::Undefined,
            ready: false,
            ready_detail: "not observed".into(),
            used: ModelFact::Undefined,
            activation: ActivationObservation {
                pointer_present: false,
                selected_model_ref: None,
                ready: false,
                detail: "not observed".into(),
            },
        }
    }

    /// Load selected/ready from Phase D activation under node root; `used` stays Undefined.
    ///
    /// Uses UI-safe [`ActivatedPointerGate::observe`] (`#303` / `#309`): cache miss
    /// defers streaming weight hash; durable fail for a version does not rehash-storm.
    pub fn load(root: impl AsRef<Path>) -> Self {
        let obs = ActivatedPointerGate::from_aira_root(root).observe();
        let selected = if obs.pointer_present {
            ModelFact::from_optional_ref(obs.selected_model_ref.clone())
        } else {
            ModelFact::None
        };
        Self {
            selected,
            ready: obs.ready,
            ready_detail: obs.detail.clone(),
            used: ModelFact::Undefined,
            activation: obs,
        }
    }

    /// Attach used-in-result without mutating selected/ready.
    pub fn with_used(mut self, used: ModelFact) -> Self {
        self.used = used;
        self
    }
}

/// Load [`ModelTripleSnapshot`] (selected/ready; used left Undefined).
pub fn load_model_triple(root: impl AsRef<Path>) -> ModelTripleSnapshot {
    ModelTripleSnapshot::load(root)
}

/// Strip / System coarse conclusion from the triple (`#269`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTripleConclusion {
    /// No selected model and not ready.
    NoneSelected,
    /// Selected but activation evidence not confirmed.
    SelectedNotReady,
    /// Selected and ready (used may still be undefined).
    Ready,
    /// Last result named a model (may differ from selected).
    UsedInResult,
}

impl ModelTripleConclusion {
    pub fn from_triple(t: &ModelTripleSnapshot) -> Self {
        if matches!(t.used, ModelFact::Value(_)) {
            return Self::UsedInResult;
        }
        if t.ready {
            return Self::Ready;
        }
        if matches!(t.selected, ModelFact::Value(_)) {
            return Self::SelectedNotReady;
        }
        Self::NoneSelected
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoneSelected => "none_selected",
            Self::SelectedNotReady => "selected_not_ready",
            Self::Ready => "ready",
            Self::UsedInResult => "used_in_result",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    #[test]
    fn load_without_pointer_is_none_not_ready() {
        let dir = tempdir().unwrap();
        let snap = ModelTripleSnapshot::load(dir.path());
        assert_eq!(snap.selected, ModelFact::None);
        assert!(!snap.ready);
        assert_eq!(snap.used, ModelFact::Undefined);
        assert_eq!(
            ModelTripleConclusion::from_triple(&snap),
            ModelTripleConclusion::NoneSelected
        );
    }

    #[test]
    #[serial]
    fn fixture_ready_does_not_fill_used() {
        let dir = tempdir().unwrap();
        aira_object::reset_primary_signer();
        let gate = ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        // `#303`: UI load defers hash on miss; warm observe-ready before asserting ready.
        assert!(gate.observe_verify_now().ready);
        let snap = ModelTripleSnapshot::load(dir.path());
        assert!(matches!(snap.selected, ModelFact::Value(_)));
        assert!(snap.ready);
        assert_eq!(snap.used, ModelFact::Undefined);
        assert_eq!(
            ModelTripleConclusion::from_triple(&snap),
            ModelTripleConclusion::Ready
        );
        let with_used = snap.with_used(ModelFact::Value("aira:model:other".into()));
        assert_eq!(
            ModelTripleConclusion::from_triple(&with_used),
            ModelTripleConclusion::UsedInResult
        );
        assert!(
            matches!(with_used.selected, ModelFact::Value(ref s) if s == "aira:model:test-activated")
        );
    }

    #[test]
    #[serial]
    fn load_on_miss_is_pending_not_blocking_ready() {
        let dir = tempdir().unwrap();
        aira_object::reset_primary_signer();
        ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        let snap = ModelTripleSnapshot::load(dir.path());
        assert!(matches!(snap.selected, ModelFact::Value(_)));
        assert!(!snap.ready);
        assert!(
            snap.ready_detail.contains("pending"),
            "{}",
            snap.ready_detail
        );
    }

    #[test]
    #[serial]
    fn selected_survives_when_not_ready() {
        let dir = tempdir().unwrap();
        aira_object::reset_primary_signer();
        ActivatedPointerGate::install_fixture(dir.path()).unwrap();
        std::fs::remove_file(dir.path().join("models/cache/l218/weights.bin")).unwrap();
        let snap = ModelTripleSnapshot::load(dir.path());
        assert!(matches!(snap.selected, ModelFact::Value(_)));
        assert!(!snap.ready);
        assert_eq!(
            ModelTripleConclusion::from_triple(&snap),
            ModelTripleConclusion::SelectedNotReady
        );
    }
}
