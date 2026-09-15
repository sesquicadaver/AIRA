use aira_desktop_runtime::{evaluate_work_readiness, ModelFact, WorkExecutorPreference};

use super::AiraDesktopApp;
use crate::lexicon::{work_submit_gate, ErrorCode, UiProblem};
use crate::work_view::WorkSubmitModelContext;

impl AiraDesktopApp {
    /// Work-screen executor preference (`#349` / RFC-0232).
    pub(super) fn work_preference(&self) -> WorkExecutorPreference {
        if self.work_executor_auto {
            WorkExecutorPreference::Auto
        } else {
            WorkExecutorPreference::Required(self.work_required_ref.clone())
        }
    }

    /// Recompute pre-submit readiness (math ≠ generate).
    pub(super) fn refresh_work_readiness(&mut self) {
        self.work_readiness = evaluate_work_readiness(
            &self.paths.data_root,
            &self.problem_text,
            self.work_preference(),
        );
    }

    /// Submit-time requested/applied for the result triple (`#350`).
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

    /// Queue a background submit (`#257`). Does not block the egui thread.
    ///
    /// The draft (`problem_text`) is **not** cleared on validation errors, Help,
    /// section switches, or failed runs (`#260`).
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
        let admission = self.work_readiness.admission.clone();
        let model_ctx = self.work_submit_model_context();
        let ensure_started = !self.node_running;
        let ctx = ctx.clone();
        let started = self.async_jobs.try_spawn_submit(
            self.paths.clone(),
            self.settings.clone(),
            self.node_bin.clone(),
            text,
            ensure_started,
            admission,
            model_ctx,
            move || ctx.request_repaint(),
        );
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
