use aira_desktop_runtime::{NetworkProfile, DEFAULT_RELAY_TTL_DAYS};

use super::AiraDesktopApp;

impl eframe::App for AiraDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("AIRA Desktop");
                ui.label("Developer Preview · P0–P4 network · P5 federation · P6 discovery (Dev)");
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
                        self.refresh_federation_detail();
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
                let mut gossip_on = self.settings.network_profile.is_gossip_profile();
                if ui.checkbox(&mut gossip_on, "P4 gossip trust").changed() {
                    self.toggle_gossip_profile(gossip_on);
                }
                if self.settings.network_profile.is_gossip_profile() {
                    ui.label("gossip status: enabled (dht+apply-book+apply-trust)");
                }

                ui.separator();
                ui.heading("Federation (P5)");
                ui.label("Local pin: import signed federation descriptor JSON (no remote handshake).");
                ui.label(&self.federation_detail);
                if ui.button("Import federation descriptor…").clicked() {
                    self.import_federation_descriptor_dialog();
                }

                ui.separator();
                ui.heading("Discovery (P6 Dev)");
                ui.label("Operator shortcuts only — explicit STUN server; no public STUN default; no auto-trust.");
                ui.horizontal(|ui| {
                    ui.label("stun_server:");
                    ui.text_edit_singleline(&mut self.stun_server_edit);
                    if ui.button("STUN query").clicked() {
                        self.run_stun_query();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("discv to:");
                    ui.text_edit_singleline(&mut self.discv_to_edit);
                    ui.label("addr:");
                    ui.text_edit_singleline(&mut self.discv_addr_edit);
                    if ui.button("discv announce").clicked() {
                        self.run_discv_announce();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("find key:");
                    ui.text_edit_singleline(&mut self.find_key_edit);
                    ui.label("seed to:");
                    ui.text_edit_singleline(&mut self.find_to_edit);
                    if ui.button("discv FIND").clicked() {
                        self.run_discv_find();
                    }
                });
                if let Some(msg) = &self.discovery_msg {
                    ui.label(msg);
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
                ui.label("File, QR PNG, or camera scan. Import → trust + address book.");
                if self.qr_camera.is_some() {
                    if let Some(msg) = &self.qr_camera_status {
                        ui.colored_label(egui::Color32::from_rgb(80, 140, 200), msg);
                    }
                    if ui.button("Stop camera scan").clicked() {
                        self.stop_qr_camera_scan();
                    }
                }
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
                    if self.qr_camera.is_none() && ui.button("Scan QR (camera)").clicked() {
                        self.start_qr_camera_scan();
                    }
                });
                self.poll_qr_camera_scan(ctx);
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
