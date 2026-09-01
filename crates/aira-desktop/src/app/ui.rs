use aira_desktop_runtime::{NetworkProfile, UiLang, DEFAULT_RELAY_TTL_DAYS};

use super::{AiraDesktopApp, MainTab};

impl eframe::App for AiraDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let l = self.labels();
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, MainTab::Work, l.tab_work);
                ui.selectable_value(&mut self.tab, MainTab::Node, l.tab_node);
                ui.selectable_value(&mut self.tab, MainTab::Network, l.tab_network);
                ui.selectable_value(&mut self.tab, MainTab::Settings, l.tab_settings);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(l.heading);
                ui.label(l.subtitle);
                ui.separator();
                match self.tab {
                    MainTab::Work => self.ui_work(ui),
                    MainTab::Node => self.ui_node(ui, ctx),
                    MainTab::Network => self.ui_network(ui, ctx),
                    MainTab::Settings => self.ui_settings(ui, ctx),
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

impl AiraDesktopApp {
    fn ui_work(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
        ui.heading(l.work_heading);
        ui.label(l.work_hint);
        ui.add(
            egui::TextEdit::multiline(&mut self.problem_text)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );
        if ui.button(l.work_submit).clicked() {
            self.submit_work();
        }
        ui.label(l.work_not_llm);
        if let Some(view) = &self.work_result {
            ui.separator();
            ui.strong(l.work_answer);
            let answer = if view.answer.is_empty() {
                l.work_no_answer
            } else {
                view.answer.as_str()
            };
            ui.heading(answer);
            ui.horizontal(|ui| {
                ui.strong(l.status);
                ui.label(&view.status);
            });
            if let Some(vs) = &view.verification_status {
                ui.horizontal(|ui| {
                    ui.strong(l.work_verification);
                    let color = if vs.eq_ignore_ascii_case("VERIFIED") {
                        egui::Color32::from_rgb(40, 140, 70)
                    } else {
                        egui::Color32::from_rgb(180, 120, 40)
                    };
                    ui.colored_label(color, vs);
                });
            }
            let has_ids = view.problem_id.is_some()
                || view.verified_artifact_id.is_some()
                || view.field_artifact_id.is_some();
            if has_ids {
                egui::CollapsingHeader::new(l.work_ids)
                    .id_source("work-ids")
                    .default_open(false)
                    .show(ui, |ui| {
                        if let Some(id) = &view.problem_id {
                            ui.horizontal(|ui| {
                                ui.strong(l.work_problem_id);
                                ui.monospace(id);
                            });
                        }
                        if let Some(id) = &view.verified_artifact_id {
                            ui.horizontal(|ui| {
                                ui.strong(l.work_artifact_id);
                                ui.monospace(id);
                            });
                        }
                        if let Some(id) = &view.field_artifact_id {
                            ui.horizontal(|ui| {
                                ui.strong(l.work_field_id);
                                ui.monospace(id);
                            });
                        }
                    });
            }
            egui::CollapsingHeader::new(l.work_details)
                .id_source("work-details")
                .default_open(false)
                .show(ui, |ui| {
                    ui.monospace(&view.details_json);
                });
        }
    }

    fn ui_node(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.horizontal(|ui| {
            ui.strong(l.status);
            ui.label(&self.status_label);
        });
        if !self.detail.is_empty() {
            ui.label(&self.detail);
        }
        ui.horizontal(|ui| {
            ui.strong(l.peer);
            ui.label(&self.peer_detail);
        });
        ui.label(format!("data_root: {}", self.paths.data_root.display()));

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(l.start).clicked() {
                if let Err(e) = self.do_start() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            if ui.button(l.stop).clicked() {
                if let Err(e) = self.do_stop() {
                    self.last_error = Some(format!("{e:#}"));
                }
            }
            if ui.button(l.refresh).clicked() {
                if let Err(e) = self.refresh_status() {
                    self.last_error = Some(format!("{e:#}"));
                }
                self.refresh_federation_detail();
            }
            if ui.button(l.quit).clicked() {
                let _ = self.do_stop();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        if self.restart_hint {
            ui.colored_label(egui::Color32::from_rgb(180, 120, 40), l.restart_hint);
        }
    }

    fn ui_network(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.heading(l.network_profile);
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.settings.network_profile == NetworkProfile::P0, l.p0)
                .clicked()
            {
                self.apply_profile(NetworkProfile::P0);
            }
            if ui
                .selectable_label(self.settings.network_profile == NetworkProfile::P1, l.p1)
                .clicked()
            {
                self.apply_profile(NetworkProfile::P1);
            }
            if ui
                .selectable_label(self.settings.network_profile == NetworkProfile::P2, l.p2)
                .clicked()
            {
                self.apply_profile(NetworkProfile::P2);
            }
        });
        if self.settings.network_profile.requires_peer_listen() {
            ui.horizontal(|ui| {
                ui.label(l.peer_listen);
                let resp = ui.text_edit_singleline(&mut self.peer_listen_edit);
                if ui.button(l.save_listen).clicked()
                    || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.save_peer_listen();
                }
            });
        }

        ui.separator();
        ui.heading(l.advanced);
        ui.label(l.advanced_hint);
        let mut relay_on = self.settings.network_profile.is_relay_profile();
        if ui.checkbox(&mut relay_on, l.p3_relay).changed() {
            self.toggle_relay_profile(relay_on);
        }
        if self.settings.network_profile.is_relay_profile() {
            ui.horizontal(|ui| {
                ui.label(l.relay_ttl);
                let resp = ui.text_edit_singleline(&mut self.relay_ttl_edit);
                if ui.button(l.save_ttl).clicked()
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
        if ui.checkbox(&mut gossip_on, l.p4_gossip).changed() {
            self.toggle_gossip_profile(gossip_on);
        }
        if self.settings.network_profile.is_gossip_profile() {
            ui.label("gossip status: enabled (dht+apply-book+apply-trust)");
        }

        ui.separator();
        ui.heading(l.federation);
        ui.label(l.federation_hint);
        ui.label(&self.federation_detail);
        if ui.button(l.import_federation).clicked() {
            self.import_federation_descriptor_dialog();
        }

        ui.separator();
        ui.heading(l.discovery);
        ui.label(l.discovery_hint);
        ui.horizontal(|ui| {
            ui.label(l.stun_server);
            ui.text_edit_singleline(&mut self.stun_server_edit);
            if ui.button(l.stun_query).clicked() {
                self.run_stun_query();
            }
        });
        ui.horizontal(|ui| {
            ui.label(l.discv_to);
            ui.text_edit_singleline(&mut self.discv_to_edit);
            ui.label(l.discv_addr);
            ui.text_edit_singleline(&mut self.discv_addr_edit);
            if ui.button(l.discv_announce).clicked() {
                self.run_discv_announce();
            }
        });
        ui.horizontal(|ui| {
            ui.label(l.find_key);
            ui.text_edit_singleline(&mut self.find_key_edit);
            ui.label(l.find_to);
            ui.text_edit_singleline(&mut self.find_to_edit);
            if ui.button(l.discv_find).clicked() {
                self.run_discv_find();
            }
        });
        if let Some(msg) = &self.discovery_msg {
            ui.label(msg);
        }

        ui.separator();
        ui.heading(l.friend_invite);
        ui.label(l.invite_hint);
        if self.qr_camera.is_some() {
            if let Some(msg) = &self.qr_camera_status {
                ui.colored_label(egui::Color32::from_rgb(80, 140, 200), msg);
            }
            if ui.button(l.stop_camera).clicked() {
                self.stop_qr_camera_scan();
            }
        }
        ui.horizontal(|ui| {
            if ui.button(l.export_json).clicked() {
                self.export_json_dialog();
            }
            if ui.button(l.import_json).clicked() {
                self.import_json_dialog();
            }
        });
        ui.horizontal(|ui| {
            if ui.button(l.show_qr).clicked() {
                self.load_qr_preview(ctx);
            }
            if ui.button(l.export_qr).clicked() {
                self.export_qr_dialog(ctx);
            }
            if ui.button(l.import_qr).clicked() {
                self.import_qr_dialog();
            }
            if self.qr_camera.is_none() && ui.button(l.scan_qr).clicked() {
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
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.heading(l.settings_heading);
        ui.horizontal(|ui| {
            ui.strong(l.language);
            if ui
                .selectable_label(self.ui_prefs.ui_lang == UiLang::Uk, l.lang_uk)
                .clicked()
            {
                self.set_ui_lang(UiLang::Uk, ctx);
            }
            if ui
                .selectable_label(self.ui_prefs.ui_lang == UiLang::En, l.lang_en)
                .clicked()
            {
                self.set_ui_lang(UiLang::En, ctx);
            }
        });
        ui.label(l.not_llm);
        ui.separator();
        let mut dirty = false;
        dirty |= ui
            .checkbox(&mut self.settings.open_ui_on_start, l.open_window_on_login)
            .changed();
        ui.label(l.open_window_hint);
        dirty |= ui
            .checkbox(&mut self.settings.autostart_on_login, l.autostart)
            .changed();
        ui.label(format!("HTTP: {}", self.settings.http_listen));
        ui.label(format!("instance: {}", self.settings.instance_id));
        if dirty {
            if let Err(e) = self.persist_settings() {
                self.last_error = Some(format!("{e:#}"));
            }
        }
    }
}
