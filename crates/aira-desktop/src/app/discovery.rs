use crate::actions;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    pub(super) fn run_stun_query(&mut self) {
        match actions::discovery_stun_query(&self.paths, &self.stun_server_edit) {
            Ok(out) => {
                self.discovery_msg = Some(format!(
                    "STUN reflexive {} via {}",
                    out.reflexive_addr, out.stun_server
                ));
                self.last_error = None;
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    pub(super) fn run_discv_announce(&mut self) {
        match actions::discovery_discv_announce(
            &self.paths,
            &self.discv_to_edit,
            &self.discv_addr_edit,
        ) {
            Ok(msg) => {
                self.discovery_msg = Some(msg);
                self.last_error = None;
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    pub(super) fn run_discv_find(&mut self) {
        let to = self.find_to_edit.trim();
        let to_opt = if to.is_empty() { None } else { Some(to) };
        match actions::discovery_discv_find(&self.paths, &self.find_key_edit, to_opt, 8) {
            Ok(report) => {
                self.discovery_msg = Some(format!(
                    "FIND hops={} queried={} stored={}{}",
                    report.hops,
                    report.queried,
                    report.stored,
                    report
                        .exact
                        .map(|(id, addr)| format!(" exact {id} @ {addr}"))
                        .unwrap_or_default()
                ));
                self.last_error = None;
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }
}
