//! egui application state for AIRA Desktop (QUEUE #78 / #85).

use std::path::PathBuf;

use aira_desktop_runtime::{
    load_or_create_settings, start, status, stop, sync_autostart_from_settings, DesktopPaths,
    DesktopSettings, LifecycleStatus, NetworkProfile, DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;

pub struct AiraDesktopApp {
    paths: DesktopPaths,
    node_bin: Option<PathBuf>,
    settings: DesktopSettings,
    status_label: String,
    detail: String,
    peer_detail: String,
    peer_listen_edit: String,
    relay_ttl_edit: String,
    invite_msg: Option<String>,
    last_error: Option<String>,
    qr_texture: Option<egui::TextureHandle>,
    restart_hint: bool,
}

impl AiraDesktopApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        paths: DesktopPaths,
        node_bin: Option<PathBuf>,
        auto_start: bool,
    ) -> Self {
        let mut last_error = None;
        let settings = match load_or_create_settings(&paths) {
            Ok(s) => s,
            Err(e) => {
                last_error = Some(format!("{e:#}"));
                DesktopSettings::default_p0(&paths)
            }
        };
        let peer_listen_edit = settings
            .peer_listen
            .clone()
            .unwrap_or_else(|| DEFAULT_PEER_LISTEN.to_string());
        let relay_ttl_edit = settings
            .relay_ttl_days
            .map(|d| d.to_string())
            .unwrap_or_else(|| DEFAULT_RELAY_TTL_DAYS.to_string());
        let mut app = Self {
            paths,
            node_bin,
            settings,
            status_label: "stopped".into(),
            detail: String::new(),
            peer_detail: String::new(),
            peer_listen_edit,
            relay_ttl_edit,
            invite_msg: None,
            last_error,
            qr_texture: None,
            restart_hint: false,
        };
        let _ = app.refresh_status();
        if auto_start {
            if let Err(e) = app.do_start() {
                app.last_error = Some(format!("{e:#}"));
            }
        }
        if let Err(e) = sync_autostart_from_settings(app.settings.autostart_on_login) {
            app.last_error = Some(format!("autostart sync: {e:#}"));
        }
        app
    }

    fn refresh_status(&mut self) -> anyhow::Result<()> {
        let (st, rec) = status(&self.paths)?;
        self.status_label = status_label(st).to_string();
        match rec {
            Some(r) => {
                self.detail = format!("pid {} · {} · {}", r.pid, r.listen, r.instance_id);
                self.peer_detail = match (r.peer_pid, r.peer_listen.as_ref()) {
                    (Some(pp), Some(pl)) => format_peer_running(
                        self.settings.network_profile,
                        pp,
                        pl,
                        self.settings.relay_ttl_days,
                    ),
                    _ if self.settings.network_profile.requires_peer_listen() => {
                        format_peer_not_running(self.settings.network_profile)
                    }
                    _ => "peer off (P0)".into(),
                };
            }
            None => {
                self.detail = format!("listen {}", self.settings.http_listen);
                self.peer_detail = if self.settings.network_profile.requires_peer_listen() {
                    format_peer_configured(
                        self.settings.network_profile,
                        self.settings.peer_listen.as_deref(),
                        self.settings.relay_ttl_days,
                    )
                } else {
                    "peer off (P0)".into()
                };
            }
        }
        Ok(())
    }

    fn do_start(&mut self) -> anyhow::Result<()> {
        let outcome = start(&self.paths, self.node_bin.clone())?;
        self.status_label = status_label(outcome.status).to_string();
        self.detail = format!(
            "{}pid {:?} · {} · {}",
            if outcome.attached { "attached · " } else { "" },
            outcome.pid,
            outcome.listen,
            outcome.instance_id
        );
        self.peer_detail = match (outcome.peer_pid, outcome.peer_listen.as_ref()) {
            (Some(pp), Some(pl)) => format_peer_running(
                self.settings.network_profile,
                pp,
                pl,
                self.settings.relay_ttl_days,
            ),
            _ if self.settings.network_profile.requires_peer_listen() => {
                format_peer_not_running(self.settings.network_profile)
            }
            _ => "peer off (P0)".into(),
        };
        self.restart_hint = false;
        self.last_error = None;
        Ok(())
    }

    fn do_stop(&mut self) -> anyhow::Result<()> {
        let st = stop(&self.paths)?;
        self.status_label = status_label(st).to_string();
        self.detail.clear();
        self.peer_detail.clear();
        self.last_error = None;
        Ok(())
    }

    fn persist_settings(&mut self) -> anyhow::Result<()> {
        actions::persist_settings(&self.paths, &self.settings)?;
        sync_autostart_from_settings(self.settings.autostart_on_login)?;
        self.last_error = None;
        Ok(())
    }

    fn apply_profile(&mut self, profile: NetworkProfile) {
        let relay_ttl = if profile.is_relay_profile() {
            Some(
                self.relay_ttl_edit
                    .trim()
                    .parse()
                    .unwrap_or(DEFAULT_RELAY_TTL_DAYS),
            )
        } else {
            None
        };
        match actions::apply_network_profile(
            &mut self.settings,
            profile,
            &self.peer_listen_edit,
            relay_ttl,
        ) {
            Ok(()) => {
                if profile.requires_peer_listen() {
                    self.peer_listen_edit = self
                        .settings
                        .peer_listen
                        .clone()
                        .unwrap_or_else(|| DEFAULT_PEER_LISTEN.to_string());
                }
                if profile.is_relay_profile() {
                    self.relay_ttl_edit = self
                        .settings
                        .relay_ttl_days
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| DEFAULT_RELAY_TTL_DAYS.to_string());
                }
                self.restart_hint = true;
                if let Err(e) = self.persist_settings() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    fn save_peer_listen(&mut self) {
        let profile = self.settings.network_profile;
        if !profile.requires_peer_listen() {
            return;
        }
        let relay_ttl = if profile.is_relay_profile() {
            Some(
                self.relay_ttl_edit
                    .trim()
                    .parse()
                    .unwrap_or(DEFAULT_RELAY_TTL_DAYS),
            )
        } else {
            None
        };
        match actions::apply_network_profile(
            &mut self.settings,
            profile,
            &self.peer_listen_edit,
            relay_ttl,
        ) {
            Ok(()) => {
                self.restart_hint = true;
                if let Err(e) = self.persist_settings() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    fn save_relay_ttl(&mut self) {
        if !self.settings.network_profile.is_relay_profile() {
            return;
        }
        let days = self
            .relay_ttl_edit
            .trim()
            .parse()
            .unwrap_or(DEFAULT_RELAY_TTL_DAYS);
        match actions::apply_network_profile(
            &mut self.settings,
            NetworkProfile::P3,
            &self.peer_listen_edit,
            Some(days),
        ) {
            Ok(()) => {
                self.restart_hint = true;
                if let Err(e) = self.persist_settings() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            Err(e) => self.last_error = Some(format!("{e:#}")),
        }
    }

    fn toggle_relay_profile(&mut self, enable: bool) {
        if enable {
            self.apply_profile(NetworkProfile::P3);
        } else if self.settings.network_profile == NetworkProfile::P3 {
            self.apply_profile(NetworkProfile::P2);
        }
    }

    fn load_qr_preview(&mut self, ctx: &egui::Context) {
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

    fn export_json_dialog(&mut self) {
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

    fn export_qr_dialog(&mut self, ctx: &egui::Context) {
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

    fn import_json_dialog(&mut self) {
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

    fn import_qr_dialog(&mut self) {
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
}

impl eframe::App for AiraDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("AIRA Desktop");
                ui.label("Developer Preview · P0/P1/P2 local · P3 relay in Advanced");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.strong("Status:");
                    ui.label(&self.status_label);
                });
                if !self.detail.is_empty() {
                    ui.label(&self.detail);
                }
                ui.horizontal(|ui| {
                    ui.strong("Peer:");
                    ui.label(&self.peer_detail);
                });
                ui.label(format!("data_root: {}", self.paths.data_root.display()));

                ui.separator();
                ui.heading("Controls");
                ui.horizontal(|ui| {
                    if ui.button("Start").clicked() {
                        if let Err(e) = self.do_start() {
                            self.last_error = Some(format!("{e:#}"));
                        }
                    }
                    if ui.button("Stop").clicked() {
                        if let Err(e) = self.do_stop() {
                            self.last_error = Some(format!("{e:#}"));
                        }
                    }
                    if ui.button("Refresh").clicked() {
                        if let Err(e) = self.refresh_status() {
                            self.last_error = Some(format!("{e:#}"));
                        }
                    }
                    if ui.button("Quit").clicked() {
                        let _ = self.do_stop();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                if self.restart_hint {
                    ui.colored_label(
                        egui::Color32::from_rgb(180, 120, 40),
                        "Profile/listen changed — Stop then Start to apply peer.",
                    );
                }

                ui.separator();
                ui.heading("Network profile");
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            self.settings.network_profile == NetworkProfile::P0,
                            "P0 local HTTP",
                        )
                        .clicked()
                    {
                        self.apply_profile(NetworkProfile::P0);
                    }
                    if ui
                        .selectable_label(
                            self.settings.network_profile == NetworkProfile::P1,
                            "P1 + peer listen",
                        )
                        .clicked()
                    {
                        self.apply_profile(NetworkProfile::P1);
                    }
                    if ui
                        .selectable_label(
                            self.settings.network_profile == NetworkProfile::P2,
                            "P2 + DHT book",
                        )
                        .clicked()
                    {
                        self.apply_profile(NetworkProfile::P2);
                    }
                });
                if self.settings.network_profile.requires_peer_listen() {
                    ui.horizontal(|ui| {
                        ui.label("peer_listen:");
                        let resp = ui.text_edit_singleline(&mut self.peer_listen_edit);
                        if ui.button("Save listen").clicked()
                            || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            self.save_peer_listen();
                        }
                    });
                }

                ui.separator();
                ui.heading("Advanced");
                ui.label(
                    "P3 relay hub (--relay) and P4 gossip (--gossip) are mutually exclusive on one peer listen.",
                );
                let mut relay_on = self.settings.network_profile.is_relay_profile();
                if ui.checkbox(&mut relay_on, "P3 relay hub").changed() {
                    self.toggle_relay_profile(relay_on);
                }
                if self.settings.network_profile.is_relay_profile() {
                    ui.horizontal(|ui| {
                        ui.label("relay_ttl_days:");
                        let resp = ui.text_edit_singleline(&mut self.relay_ttl_edit);
                        if ui.button("Save TTL").clicked()
                            || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            self.save_relay_ttl();
                        }
                    });
                    let ttl = self
                        .settings
                        .relay_ttl_days
                        .unwrap_or(DEFAULT_RELAY_TTL_DAYS);
                    ui.label(format!("relay status: enabled · TTL {ttl} days"));
                }

                ui.separator();
                ui.heading("Settings");
                let mut dirty = false;
                dirty |= ui
                    .checkbox(&mut self.settings.open_ui_on_start, "Open UI on start")
                    .changed();
                dirty |= ui
                    .checkbox(
                        &mut self.settings.autostart_on_login,
                        "Autostart on login (XDG)",
                    )
                    .changed();
                ui.label(format!("HTTP: {}", self.settings.http_listen));
                ui.label(format!("instance: {}", self.settings.instance_id));
                if dirty {
                    if let Err(e) = self.persist_settings() {
                        self.last_error = Some(format!("{e:#}"));
                    }
                }

                ui.separator();
                ui.heading("Friend invite");
                ui.label("File or QR PNG (no camera). Import → trust + address book.");
                ui.horizontal(|ui| {
                    if ui.button("Export JSON…").clicked() {
                        self.export_json_dialog();
                    }
                    if ui.button("Import JSON…").clicked() {
                        self.import_json_dialog();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Show QR").clicked() {
                        self.load_qr_preview(ctx);
                    }
                    if ui.button("Export QR…").clicked() {
                        self.export_qr_dialog(ctx);
                    }
                    if ui.button("Import QR…").clicked() {
                        self.import_qr_dialog();
                    }
                });
                if let Some(msg) = &self.invite_msg {
                    ui.label(msg);
                }
                if let Some(tex) = &self.qr_texture {
                    ui.add(egui::Image::new(tex).max_width(220.0));
                }

                if let Some(err) = &self.last_error {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(200, 60, 60), err);
                }
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_secs(2));
    }
}

fn status_label(st: LifecycleStatus) -> &'static str {
    match st {
        LifecycleStatus::Stopped => "stopped",
        LifecycleStatus::Starting => "starting",
        LifecycleStatus::Running => "running",
        LifecycleStatus::Unhealthy => "unhealthy",
        LifecycleStatus::Stopping => "stopping",
        LifecycleStatus::Failed => "failed",
    }
}

fn format_peer_running(
    profile: NetworkProfile,
    pid: u32,
    listen: &str,
    relay_ttl_days: Option<u32>,
) -> String {
    match profile {
        NetworkProfile::P3 => {
            let ttl = relay_ttl_days.unwrap_or(DEFAULT_RELAY_TTL_DAYS);
            format!("peer running (relay · TTL {ttl}d) · pid {pid} @ {listen}")
        }
        NetworkProfile::P2 => format!("peer running (dht+apply-book) · pid {pid} @ {listen}"),
        NetworkProfile::P1 => format!("peer running · pid {pid} @ {listen}"),
        _ => format!("peer running · pid {pid} @ {listen}"),
    }
}

fn format_peer_not_running(profile: NetworkProfile) -> String {
    match profile {
        NetworkProfile::P3 => "peer not running (Start with P3 relay)".into(),
        NetworkProfile::P2 => "peer not running (Start with P2)".into(),
        NetworkProfile::P1 => "peer not running (Start with P1)".into(),
        _ => "peer not running".into(),
    }
}

fn format_peer_configured(
    profile: NetworkProfile,
    listen: Option<&str>,
    relay_ttl_days: Option<u32>,
) -> String {
    let addr = listen.unwrap_or(DEFAULT_PEER_LISTEN);
    match profile {
        NetworkProfile::P3 => {
            let ttl = relay_ttl_days.unwrap_or(DEFAULT_RELAY_TTL_DAYS);
            format!("peer configured (relay · TTL {ttl}d) · {addr}")
        }
        NetworkProfile::P2 => format!("peer configured (dht+apply-book) · {addr}"),
        NetworkProfile::P1 => format!("peer configured · {addr}"),
        _ => format!("peer configured · {addr}"),
    }
}
