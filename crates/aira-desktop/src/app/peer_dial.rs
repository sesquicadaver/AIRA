//! Phase S `#300` / Phase T `#308`: opt-in peer dial action (Technical details).
//!
//! Dial runs on an `async_jobs` worker — never `block_on` on the egui thread.

use egui::Context;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    /// Queue opt-in dial off the UI thread; F1/nav remain available (`#308`).
    pub(super) fn request_opt_in_peer_dial(&mut self, ctx: &Context) {
        if self.async_jobs.dial_inflight() {
            return;
        }
        let l = self.labels();
        self.dial_msg = Some(l.dial_waiting.to_string());
        self.clear_problem();
        let ctx = ctx.clone();
        let started = self.async_jobs.try_spawn_dial(
            self.paths.clone(),
            self.dial_peer_edit.clone(),
            self.dial_addr_edit.clone(),
            move || ctx.request_repaint(),
        );
        if !started {
            self.dial_msg = None;
        }
    }
}
