use aira_desktop_runtime::{evaluate_work_readiness, ModelFact, WorkExecutorPreference};

use super::AiraDesktopApp;
use crate::lexicon::{work_submit_gate, ErrorCode, UiProblem};
use crate::work_view::WorkSubmitModelContext;

/// Work-screen executor radio (`#349` / `#354`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkExecutorUiMode {
    Auto,
    Specific,
    Compare,
}

impl AiraDesktopApp {
    /// Work-screen executor preference (`#349` / RFC-0232; Compare `#354` / RFC-0237).
    pub(super) fn work_preference(&self) -> WorkExecutorPreference {
        match self.work_executor_mode {
            WorkExecutorUiMode::Auto => WorkExecutorPreference::Auto,
            WorkExecutorUiMode::Specific => {
                WorkExecutorPreference::Required(self.work_required_ref.clone())
            }
            WorkExecutorUiMode::Compare => WorkExecutorPreference::Compare {
                a: self.work_compare_a.clone(),
                b: self.work_compare_b.clone(),
            },
        }
    }

    /// Recompute pre-submit readiness (math ≠ generate; Compare fail-closed).
    pub(super) fn refresh_work_readiness(&mut self) {
        self.work_readiness = evaluate_work_readiness(
            &self.paths.data_root,
            &self.problem_text,
            self.work_preference(),
        );
    }

    /// Submit-time requested/applied for a single admit (`#350`).
    pub(super) fn work_submit_model_context(&self) -> WorkSubmitModelContext {
        let requested = self.work_readiness.admission.model_ref.clone();
        let applied = match &self.model_triple.selected {
            ModelFact::Value(s) => Some(s.clone()),
            ModelFact::Undefined | ModelFact::None => self
                .model_catalog
                .tip_model_ref
                .clone()
                .or_else(|| requested.clone()),
        };
        WorkSubmitModelContext { requested, applied }
    }

    /// Exact-bind context for one Compare leg (requested = applied = model_ref).
    fn compare_leg_context(model_ref: &str) -> WorkSubmitModelContext {
        let r = Some(model_ref.to_string());
        WorkSubmitModelContext {
            requested: r.clone(),
            applied: r,
        }
    }

    /// Queue a background submit (`#257`). Does not block the egui thread.
    ///
    /// The draft (`problem_text`) is **not** cleared on validation errors, Help,
    /// section switches, or failed runs (`#260`).
    /// Compare (`#354`) runs A then B in one worker — no silent substitute.
    pub(super) fn submit_work(&mut self, ctx: &egui::Context) {
        let gate = work_submit_gate(
            self.async_jobs.work_inflight(),
            self.async_jobs.lifecycle_inflight(),
        );
        if !gate.available {
            if let Some(code) = gate.reason {
                self.last_problem = Some(UiProblem::new(code, self.ui_lang(), None));
            }
            return;
        }
        let text = self.problem_text.clone();
        if text.trim().is_empty() {
            self.last_problem = Some(UiProblem::new(
                ErrorCode::WorkEmptyText,
                self.ui_lang(),
                None,
            ));
            return;
        }
        self.refresh_work_readiness();
        if !self.work_readiness.ready {
            let detail = self.work_readiness.reasons.join("; ");
            self.last_problem = Some(UiProblem::new(
                ErrorCode::WorkModelUnready,
                self.ui_lang(),
                if detail.is_empty() {
                    None
                } else {
                    Some(detail)
                },
            ));
            return;
        }
        let ensure_started = !self.node_running;
        let ctx = ctx.clone();
        let on_done = move || ctx.request_repaint();

        let started = if matches!(self.work_executor_mode, WorkExecutorUiMode::Compare) {
            let admission_a = self.work_readiness.admission.clone();
            let Some(admission_b) = self.work_readiness.admission_b.clone() else {
                self.last_problem = Some(UiProblem::new(
                    ErrorCode::WorkModelUnready,
                    self.ui_lang(),
                    Some("Compare admission B missing — no silent substitute".into()),
                ));
                return;
            };
            let a_ref = self
                .work_readiness
                .resolved_model_ref
                .clone()
                .unwrap_or_default();
            let b_ref = self
                .work_readiness
                .resolved_model_ref_b
                .clone()
                .unwrap_or_default();
            let model_ctx_a = Self::compare_leg_context(&a_ref);
            let model_ctx_b = Self::compare_leg_context(&b_ref);
            // Clear prior dual surface before spawn.
            self.work_result_b = None;
            self.work_compare_b_error = None;
            self.async_jobs.try_spawn_compare_submit(
                self.paths.clone(),
                self.settings.clone(),
                self.node_bin.clone(),
                text,
                ensure_started,
                admission_a,
                model_ctx_a,
                admission_b,
                model_ctx_b,
                on_done,
            )
        } else {
            let admission = self.work_readiness.admission.clone();
            let model_ctx = self.work_submit_model_context();
            self.work_result_b = None;
            self.work_compare_b_error = None;
            self.async_jobs.try_spawn_submit(
                self.paths.clone(),
                self.settings.clone(),
                self.node_bin.clone(),
                text,
                ensure_started,
                admission,
                model_ctx,
                on_done,
            )
        };
        if !started {
            let code = if self.async_jobs.lifecycle_inflight() {
                ErrorCode::LifecycleBusy
            } else {
                ErrorCode::WorkSubmitInFlight
            };
            self.last_problem = Some(UiProblem::new(code, self.ui_lang(), None));
            return;
        }
        self.clear_problem();
    }
}
