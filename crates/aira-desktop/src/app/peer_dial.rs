//! Phase S `#300` opt-in peer dial action (Technical details).

use crate::actions;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    /// Dial a trusted peer at an explicit address; persist handshake evidence.
    pub(super) fn run_opt_in_peer_dial(&mut self) {
        match actions::opt_in_peer_dial(&self.paths, &self.dial_peer_edit, &self.dial_addr_edit) {
            Ok(out) => {
                self.dial_msg = Some(format!(
                    "confirmed handshake {}",
                    out.evidence.summary_line()
                ));
                self.clear_problem();
                let _ = self.refresh_status();
            }
            Err(e) => self.set_problem(crate::lexicon::ErrorCode::Generic, format!("{e:#}")),
        }
    }
}
