use aira_desktop_runtime::{
    evaluate_work_readiness_applied, AppliedHostLlm, LlmBackend, ModelFact, WorkExecutorPreference,
};

use super::AiraDesktopApp;
use crate::lexicon::{work_run_available, work_submit_gate, ErrorCode, UiProblem};
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

    /// Recompute pre-submit readiness (host LLM required; RFC-0242).
    ///
    /// A running node is judged by the applied executor, not by settings that
    /// were only saved. A stopped node uses saved settings (next start).
    pub(super) fn refresh_work_readiness(&mut self) {
        let applied = self.applied_host_for_readiness();
        self.work_readiness = evaluate_work_readiness_applied(
            &self.paths.data_root,
            &self.settings,
            &self.problem_text,
            self.work_preference(),
            applied.as_ref(),
        );
    }

    /// `Some` only while the node is running. Unconfirmed runtime is treated as mock.
    fn applied_host_for_readiness(&self) -> Option<AppliedHostLlm> {
        if !self.node_running {
            return None;
        }
        Some(match &self.applied_runtime {
            Some(a) => AppliedHostLlm {
                backend: a.llm_backend,
                ollama_model: a.llm_ollama_model.clone(),
            },
            None => AppliedHostLlm {
                backend: LlmBackend::Mock,
                ollama_model: None,
            },
        })
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
        WorkSubmitModelContext {
            requested,
            applied,
            prompt: Some(self.problem_text.clone()),
        }
    }

    /// Exact-bind context for one Compare leg (requested = applied = model_ref).
    fn compare_leg_context(&self, model_ref: &str) -> WorkSubmitModelContext {
        let r = Some(model_ref.to_string());
        WorkSubmitModelContext {
            requested: r.clone(),
            applied: r,
            prompt: Some(self.problem_text.clone()),
        }
    }

    /// Run button, shortcut, and [`Self::submit_work`] share this check.
    pub(super) fn work_can_run(&self) -> bool {
        work_run_available(
            self.async_jobs.work_inflight(),
            self.async_jobs.lifecycle_inflight(),
            self.async_jobs.catalog_mutate_inflight(),
            self.work_readiness.ready,
            self.problem_text.trim().is_empty(),
        )
    }

    /// Queue a background submit (`#257`). Does not block the egui thread.
    ///
    /// The draft (`problem_text`) is **not** cleared on validation errors, Help,
    /// section switches, or failed runs (`#260`).
    /// Compare (`#354`) runs A then B in one worker — no silent substitute.
    /// Availability is [`Self::work_can_run`] — the same check as the button and shortcut.
    pub(super) fn submit_work(&mut self, ctx: &egui::Context) {
        self.refresh_work_readiness();
        if !self.work_can_run() {
            let gate = work_submit_gate(
                self.async_jobs.work_inflight(),
                self.async_jobs.lifecycle_inflight(),
                self.async_jobs.catalog_mutate_inflight(),
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
        let text = self.problem_text.clone();
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
            let model_ctx_a = self.compare_leg_context(&a_ref);
            let model_ctx_b = self.compare_leg_context(&b_ref);
            // Pack F: clear prior result surface before spawn so a failed run
            // cannot keep showing an old answer under a new prompt.
            self.work_result = None;
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
            self.work_result = None;
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
            } else if self.async_jobs.catalog_mutate_inflight() {
                ErrorCode::CatalogBusy
            } else {
                ErrorCode::WorkSubmitInFlight
            };
            self.last_problem = Some(UiProblem::new(code, self.ui_lang(), None));
            return;
        }
        self.clear_problem();
    }

    /// Exact CLI names for «Prepare and run», or `None` when Run is the right control (`#362`).
    pub(super) fn prepare_and_run_names(&self) -> Option<Vec<String>> {
        let host_ok = aira_desktop_runtime::evaluate_host_llm_gate(&self.settings).ok;
        let busy = self.prepare_and_run_busy
            || self.async_jobs.work_inflight()
            || self.async_jobs.lifecycle_inflight()
            || self.async_jobs.catalog_mutate_inflight();
        aira_desktop_runtime::prepare_and_run_cli_names(
            host_ok,
            self.problem_text.trim().is_empty(),
            busy,
            &self.work_preference(),
            &self.model_catalog.entries,
            &self.ollama_models,
        )
    }

    /// Slot the selected host model, then the existing admit-and-run path.
    ///
    /// A repeat while this click or a run is in flight does not start a second run.
    /// A slot error does not change the tip and does not submit.
    pub(super) fn prepare_and_run(&mut self, ctx: &egui::Context) {
        if self.prepare_and_run_busy || self.async_jobs.work_inflight() {
            return;
        }
        let Some(names) = self.prepare_and_run_names() else {
            return;
        };
        self.prepare_and_run_busy = true;
        let root = self.paths.data_root.clone();
        let prepared = aira_desktop_runtime::execute_prepare_and_run(
            false,
            || aira_desktop_runtime::prepare_host_slots(&root, &names),
            || Ok(()),
        );
        self.prepare_and_run_busy = false;
        match prepared {
            aira_desktop_runtime::PrepareAndRunOutcome::Failed(e) => {
                self.refresh_model_catalog();
                self.last_problem = Some(UiProblem::new(
                    ErrorCode::WorkModelUnready,
                    self.ui_lang(),
                    Some(e),
                ));
            }
            aira_desktop_runtime::PrepareAndRunOutcome::IgnoredRepeat => {}
            aira_desktop_runtime::PrepareAndRunOutcome::Started => {
                self.refresh_model_catalog();
                self.submit_work(ctx);
            }
        }
    }
}
