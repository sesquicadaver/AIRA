use crate::actions;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    pub(super) fn refresh_federation_detail(&mut self) {
        match actions::federation_membership(&self.paths) {
            Ok(Some(m)) => {
                self.federation_detail = format!(
                    "joined {} · {} · since {}",
                    m.federation_id, m.identity_ref, m.joined_at
                );
            }
            Ok(None) => {
                self.federation_detail = "not joined (import descriptor to pin federation)".into();
            }
            Err(e) => self.federation_detail = format!("federation read error: {e:#}"),
        }
    }

    pub(super) fn import_federation_descriptor_dialog(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Federation descriptor JSON", &["json"])
            .pick_file();
        if let Some(path) = path {
            match actions::join_federation_descriptor(&self.paths, &path) {
                Ok(out) => {
                    self.invite_msg = Some(if out.already_member {
                        format!(
                            "already joined {} ({})",
                            out.membership.federation_id,
                            path.display()
                        )
                    } else {
                        format!(
                            "joined {} · trusted {}",
                            out.membership.federation_id, out.membership.identity_ref
                        )
                    });
                    self.last_error = None;
                    self.refresh_federation_detail();
                }
                Err(e) => self.last_error = Some(format!("{e:#}")),
            }
        }
    }
}
