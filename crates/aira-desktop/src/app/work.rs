use super::AiraDesktopApp;
use crate::lexicon::{work_submit_gate, ErrorCode, UiProblem};

impl AiraDesktopApp {
    /// Queue a background submit (`#257`). Does not block the egui thread.
    ///
    /// The draft (`problem_text`) is **not** cleared on validation errors, Help,
    /// section switches, or failed runs (`#260`).
    pub(super) fn submit_work(&mut self, ctx: &egui::Context) {
        let gate = work_submit_gate(self.async_jobs.work_inflight());
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
        let ensure_started = !self.node_running;
        let ctx = ctx.clone();
        let started = self.async_jobs.try_spawn_submit(
            self.paths.clone(),
            self.settings.clone(),
            self.node_bin.clone(),
            text,
            ensure_started,
            move || ctx.request_repaint(),
        );
        if !started {
            self.last_problem = Some(UiProblem::new(
                ErrorCode::WorkSubmitInFlight,
                self.ui_lang(),
                None,
            ));
            return;
        }
        self.clear_problem();
    }
}
