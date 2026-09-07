use super::AiraDesktopApp;

impl AiraDesktopApp {
    /// Queue a background submit (`#257`). Does not block the egui thread.
    pub(super) fn submit_work(&mut self, ctx: &egui::Context) {
        if self.async_jobs.work_inflight() {
            return;
        }
        let text = self.problem_text.clone();
        if text.trim().is_empty() {
            self.last_error = Some("problem text must be non-empty".into());
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
            return;
        }
        self.last_error = None;
    }
}
