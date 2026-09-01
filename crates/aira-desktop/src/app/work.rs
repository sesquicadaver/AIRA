use crate::actions;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    pub(super) fn submit_work(&mut self) {
        if !self.node_running {
            if let Err(e) = self.do_start() {
                self.last_error = Some(format!("{e:#}"));
                return;
            }
        }
        match actions::submit_problem(&self.paths, &self.settings, &self.problem_text) {
            Ok(view) => {
                self.work_result = Some(view);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("{e:#}"));
            }
        }
    }
}
