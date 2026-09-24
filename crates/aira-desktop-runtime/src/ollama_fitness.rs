//! Unified model fitness evaluation (`#372`).
//!
//! One state feeds the catalog row, Compatible filter, Work button label,
//! summary chip, and pre-run gate. States:
//! - [`ModelFitnessState::Runnable`] — local + `completion` + bind
//! - [`ModelFitnessState::PrepareAndRun`] — local + `completion`, bind still missing
//! - [`ModelFitnessState::Checking`] — capabilities or locality still Unknown
//! - [`ModelFitnessState::NeedsAction`] — remote/cloud, absent from list, or list freshness unknown
//! - [`ModelFitnessState::Incompatible`] — no `completion` (e.g. embedding-only)
//!
//! `completion` + `vision` / `embedding` stays ready when the rest holds.
//! `vision` alone does not enable image input. Cloud / unknown locality stay
//! in the catalog but are not ready as local.

use crate::ollama::{OllamaJsonField, OllamaTagsModel};
use crate::ollama_locality::{classify_model_locality, ModelLocality, OllamaApiLocalityContract};

/// Five fitness states shared by row / filter / button / summary / preflight (`#372`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFitnessState {
    /// Can submit generate now (slot exists).
    Runnable,
    /// Fit for local Work; next control is Prepare-and-run.
    PrepareAndRun,
    /// Waiting on capabilities or locality metadata.
    Checking,
    /// Operator action needed (remote, absent, stale list) — not “ready”.
    NeedsAction,
    /// Cannot run text generate (no `completion`).
    Incompatible,
}

impl ModelFitnessState {
    /// Stable id for tests / filter keys / button mapping.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runnable => "runnable",
            Self::PrepareAndRun => "prepare_and_run",
            Self::Checking => "checking",
            Self::NeedsAction => "needs_action",
            Self::Incompatible => "incompatible",
        }
    }

    /// Rows shown by the default «Compatible with this task» filter (`#373` consumes this).
    pub fn passes_compatible_filter(self) -> bool {
        matches!(self, Self::Runnable | Self::PrepareAndRun | Self::Checking)
    }

    /// True when Work may treat the model as locally runnable (bind or prepare).
    pub fn is_locally_actionable(self) -> bool {
        matches!(self, Self::Runnable | Self::PrepareAndRun)
    }
}

/// Single fitness verdict for one Ollama model row (`#372`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFitness {
    pub state: ModelFitnessState,
    /// Stable machine reason (not localized UI copy — `#375`).
    pub reason: &'static str,
    pub locality: ModelLocality,
    pub has_completion: bool,
}

/// Inputs that must agree for row, filter, button, summary, and preflight (`#372`).
#[derive(Debug, Clone, Copy)]
pub struct ModelFitnessInput<'a> {
    pub model: &'a OllamaTagsModel,
    pub endpoint: &'a str,
    pub contract: OllamaApiLocalityContract,
    /// Present in the last **successful** tags/list snapshot for this endpoint.
    pub listed_in_fresh_snapshot: bool,
    /// Last list refresh failed — prior names may remain (`#365` Unknown freshness).
    pub list_freshness_unknown: bool,
    /// Verified host-Ollama bind / slot already exists for this CLI name.
    pub has_host_bind: bool,
}

/// Evaluate fitness once; all UI surfaces must use this result (`#372`).
pub fn evaluate_model_fitness(input: ModelFitnessInput<'_>) -> ModelFitness {
    let locality = classify_model_locality(input.model, input.endpoint, input.contract);
    let has_completion = capability_has_completion(&input.model.capabilities);

    // 1–2: capabilities
    match &input.model.capabilities {
        OllamaJsonField::Unknown => {
            return ModelFitness {
                state: ModelFitnessState::Checking,
                reason: "capabilities_unknown",
                locality,
                has_completion: false,
            };
        }
        OllamaJsonField::Known(_) if !has_completion => {
            return ModelFitness {
                state: ModelFitnessState::Incompatible,
                reason: "missing_completion",
                locality,
                has_completion: false,
            };
        }
        OllamaJsonField::Known(_) => {}
    }

    // 3: locality unknown → still checking (do not invent local).
    if matches!(locality, ModelLocality::Unknown) {
        return ModelFitness {
            state: ModelFitnessState::Checking,
            reason: "locality_unknown",
            locality,
            has_completion: true,
        };
    }

    // 4: remote/cloud relative to the Ollama server — catalog row kept, not local-ready.
    if matches!(locality, ModelLocality::RemoteFromServer) {
        return ModelFitness {
            state: ModelFitnessState::NeedsAction,
            reason: "remote_from_server",
            locality,
            has_completion: true,
        };
    }

    // 5: list freshness / presence
    if input.list_freshness_unknown {
        return ModelFitness {
            state: ModelFitnessState::NeedsAction,
            reason: "list_freshness_unknown",
            locality,
            has_completion: true,
        };
    }
    if !input.listed_in_fresh_snapshot {
        return ModelFitness {
            state: ModelFitnessState::NeedsAction,
            reason: "absent_from_list",
            locality,
            has_completion: true,
        };
    }

    // Server-local (incl. this computer) + completion + listed.
    if input.has_host_bind {
        ModelFitness {
            state: ModelFitnessState::Runnable,
            reason: "ready",
            locality,
            has_completion: true,
        }
    } else {
        ModelFitness {
            state: ModelFitnessState::PrepareAndRun,
            reason: "needs_prepare_and_run",
            locality,
            has_completion: true,
        }
    }
}

/// `completion` present (case-insensitive). `vision` alone is not enough (`#372`).
pub fn capability_has_completion(caps: &OllamaJsonField<Vec<String>>) -> bool {
    match caps {
        OllamaJsonField::Unknown => false,
        OllamaJsonField::Known(list) => list.iter().any(|c| c.eq_ignore_ascii_case("completion")),
    }
}

/// Same verdict whether called for a row, filter, button, summary, or preflight.
pub fn fitness_surfaces_agree(a: &ModelFitness, b: &ModelFitness) -> bool {
    a.state == b.state && a.reason == b.reason
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(
        name: &str,
        caps: OllamaJsonField<Vec<String>>,
        remote_host: OllamaJsonField<String>,
    ) -> OllamaTagsModel {
        OllamaTagsModel {
            name: name.into(),
            digest: OllamaJsonField::Known("d".into()),
            size: OllamaJsonField::Known(1),
            remote_host: remote_host.clone(),
            remote_model: match &remote_host {
                OllamaJsonField::Known(h) if !h.is_empty() => {
                    OllamaJsonField::Known("remote-model".into())
                }
                OllamaJsonField::Known(_) => OllamaJsonField::Known(String::new()),
                OllamaJsonField::Unknown => OllamaJsonField::Unknown,
            },
            capabilities: caps,
        }
    }

    fn base_input<'a>(m: &'a OllamaTagsModel) -> ModelFitnessInput<'a> {
        ModelFitnessInput {
            model: m,
            endpoint: "http://127.0.0.1:11434",
            contract: OllamaApiLocalityContract::EmptyMeansServerLocal,
            listed_in_fresh_snapshot: true,
            list_freshness_unknown: false,
            has_host_bind: true,
        }
    }

    /// `#372`: five states covered — runnable, prepare, checking, needs_action, incompatible.
    #[test]
    fn five_fitness_states() {
        let empty_remote = OllamaJsonField::Known(String::new());

        // Runnable: completion + local + listed + bind
        let text = model(
            "phi:latest",
            OllamaJsonField::Known(vec!["completion".into()]),
            empty_remote.clone(),
        );
        let r = evaluate_model_fitness(base_input(&text));
        assert_eq!(r.state, ModelFitnessState::Runnable);
        assert_eq!(r.state.as_str(), "runnable");

        // PrepareAndRun: same but no bind
        let mut prep_in = base_input(&text);
        prep_in.has_host_bind = false;
        let p = evaluate_model_fitness(prep_in);
        assert_eq!(p.state, ModelFitnessState::PrepareAndRun);

        // Checking: capabilities unknown
        let checking = model("x:latest", OllamaJsonField::Unknown, empty_remote.clone());
        let c = evaluate_model_fitness(base_input(&checking));
        assert_eq!(c.state, ModelFitnessState::Checking);
        assert_eq!(c.reason, "capabilities_unknown");

        // NeedsAction: remote cloud host
        let cloud = model(
            "kimi-k2.7-code:cloud",
            OllamaJsonField::Known(vec!["completion".into()]),
            OllamaJsonField::Known("https://ollama.com".into()),
        );
        let n = evaluate_model_fitness(base_input(&cloud));
        assert_eq!(n.state, ModelFitnessState::NeedsAction);
        assert_eq!(n.reason, "remote_from_server");
        assert!(!n.state.is_locally_actionable());

        // Incompatible: embedding only
        let emb = model(
            "embed:latest",
            OllamaJsonField::Known(vec!["embedding".into()]),
            empty_remote,
        );
        let i = evaluate_model_fitness(base_input(&emb));
        assert_eq!(i.state, ModelFitnessState::Incompatible);
        assert_eq!(i.reason, "missing_completion");
    }

    /// `#372`: completion+vision is ready when the rest holds; vision ≠ image input.
    #[test]
    fn completion_plus_vision_is_ready_when_local() {
        let m = model(
            "vl:latest",
            OllamaJsonField::Known(vec!["completion".into(), "vision".into()]),
            OllamaJsonField::Known(String::new()),
        );
        let f = evaluate_model_fitness(base_input(&m));
        assert_eq!(f.state, ModelFitnessState::Runnable);
        assert!(f.has_completion);

        // Same surface agreement for filter / button / preflight projections.
        let filter = evaluate_model_fitness(base_input(&m));
        let button = evaluate_model_fitness(base_input(&m));
        let preflight = evaluate_model_fitness(base_input(&m));
        assert!(fitness_surfaces_agree(&filter, &button));
        assert!(fitness_surfaces_agree(&button, &preflight));
        assert!(filter.state.passes_compatible_filter());
    }

    /// `#372`: completion+embedding ready; embedding-only incompatible.
    #[test]
    fn completion_plus_embedding_ready_embedding_only_not() {
        let both = model(
            "mix:latest",
            OllamaJsonField::Known(vec!["embedding".into(), "completion".into()]),
            OllamaJsonField::Known(String::new()),
        );
        assert_eq!(
            evaluate_model_fitness(base_input(&both)).state,
            ModelFitnessState::Runnable
        );

        let only = model(
            "emb:latest",
            OllamaJsonField::Known(vec!["embedding".into()]),
            OllamaJsonField::Known(String::new()),
        );
        assert_eq!(
            evaluate_model_fitness(base_input(&only)).state,
            ModelFitnessState::Incompatible
        );
    }

    /// `#372`: unknown locality keeps Checking (not ready as local).
    #[test]
    fn unknown_locality_is_checking_not_ready() {
        let m = model(
            "phi:latest",
            OllamaJsonField::Known(vec!["completion".into()]),
            OllamaJsonField::Unknown,
        );
        let f = evaluate_model_fitness(base_input(&m));
        assert_eq!(f.state, ModelFitnessState::Checking);
        assert_eq!(f.reason, "locality_unknown");
    }

    /// `#372`: absent from fresh list → NeedsAction (still a catalog concern later).
    #[test]
    fn absent_from_list_needs_action() {
        let m = model(
            "gone:latest",
            OllamaJsonField::Known(vec!["completion".into()]),
            OllamaJsonField::Known(String::new()),
        );
        let mut input = base_input(&m);
        input.listed_in_fresh_snapshot = false;
        let f = evaluate_model_fitness(input);
        assert_eq!(f.state, ModelFitnessState::NeedsAction);
        assert_eq!(f.reason, "absent_from_list");
    }
}
