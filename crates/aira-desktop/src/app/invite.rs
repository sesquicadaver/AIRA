use crate::actions;
use crate::camera;

use super::AiraDesktopApp;

impl AiraDesktopApp {
    pub(super) fn load_qr_preview(&mut self, ctx: &egui::Context) {
        match actions::preview_invite_qr(&self.paths, &mut self.settings) {
            Ok((invite, w, h, rgba)) => {
                let image = egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba);
                self.qr_texture =
                    Some(ctx.load_texture("invite-qr", image, egui::TextureOptions::NEAREST));
                self.invite_msg = Some(format!(
                    "QR for {}{}",
                    invite.identity_ref,
                    invite
                        .addr
                        .as_ref()
                        .map(|a| format!(" · {a}"))
                        .unwrap_or_default()
                ));
                self.last_error = None;
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    pub(super) fn export_json_dialog(&mut self) {
        let path = rfd::FileDialog::new()
            .set_file_name("aira.invite.json")
            .add_filter("PeerInvite JSON", &["json"])
            .save_file();
        if let Some(path) = path {
            match actions::export_json(&self.paths, &path, None) {
                Ok(inv) => {
                    self.invite_msg = Some(format!("exported JSON {}", path.display()));
                    self.last_error = None;
                    let _ = inv;
                }
                Err(e) => self.last_error = Some(format!("{e:#}")),
            }
        }
    }

    pub(super) fn export_qr_dialog(&mut self, ctx: &egui::Context) {
        let path = rfd::FileDialog::new()
            .set_file_name("aira.invite.png")
            .add_filter("PNG", &["png"])
            .save_file();
        if let Some(path) = path {
            match actions::export_qr(&self.paths, &path, None) {
                Ok(_) => {
                    self.invite_msg = Some(format!("exported QR {}", path.display()));
                    self.last_error = None;
                    self.load_qr_preview(ctx);
                }
                Err(e) => self.last_error = Some(format!("{e:#}")),
            }
        }
    }

    pub(super) fn import_json_dialog(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("PeerInvite JSON", &["json"])
            .pick_file();
        if let Some(path) = path {
            match actions::import_json(&self.paths, &path) {
                Ok(out) => {
                    self.invite_msg = Some(format!(
                        "imported {} (book={})",
                        out.identity_ref, out.book_updated
                    ));
                    self.last_error = None;
                }
                Err(e) => self.last_error = Some(format!("{e:#}")),
            }
        }
    }

    pub(super) fn import_qr_dialog(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("PNG / image", &["png", "jpg", "jpeg", "webp"])
            .pick_file();
        if let Some(path) = path {
            match actions::import_qr(&self.paths, &path) {
                Ok(out) => {
                    self.invite_msg = Some(format!(
                        "imported QR {} (book={})",
                        out.identity_ref, out.book_updated
                    ));
                    self.last_error = None;
                }
                Err(e) => self.last_error = Some(format!("{e:#}")),
            }
        }
    }

    pub(super) fn start_qr_camera_scan(&mut self) {
        self.stop_qr_camera_scan();
        match camera::InviteQrCamera::open_default() {
            Ok(cam) => {
                self.qr_camera = Some(cam);
                self.qr_camera_status = Some(self.labels().scan_camera.to_string());
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("camera: {e:#}"));
            }
        }
    }

    pub(super) fn stop_qr_camera_scan(&mut self) {
        self.qr_camera = None;
        self.qr_camera_status = None;
    }

    pub(super) fn poll_qr_camera_scan(&mut self, ctx: &egui::Context) {
        if self.qr_camera.is_none() {
            return;
        }
        let luma = match self
            .qr_camera
            .as_mut()
            .and_then(|c| c.grab_luma_frame().ok())
        {
            Some(l) => l,
            None => {
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
                return;
            }
        };
        match actions::import_qr_luma(&self.paths, luma) {
            Ok(out) => {
                self.invite_msg = Some(format!(
                    "imported via camera {} (book={})",
                    out.identity_ref, out.book_updated
                ));
                self.last_error = None;
                self.stop_qr_camera_scan();
            }
            Err(_) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
        }
    }
}
