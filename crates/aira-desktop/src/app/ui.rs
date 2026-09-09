use aira_desktop_runtime::{NetworkProfile, UiLang, DEFAULT_RELAY_TTL_DAYS};

use crate::lexicon::HelpId;

use super::{AiraDesktopApp, MainTab};

impl eframe::App for AiraDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Data refresh is independent of this repaint timer (`#257`).
        self.pump_async_jobs(ctx);

        let mut open_help = false;
        let mut close_help = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                open_help = true;
            }
            if i.key_pressed(egui::Key::Escape) {
                close_help = true;
            }
        });
        if open_help {
            self.open_help_contextual();
        } else if close_help && self.help_open {
            // Esc closes Help only — never cancels in-flight work (`#259`).
            self.close_help();
        }

        let l = self.labels();
        egui::TopBottomPanel::top("shell-chrome").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut tab = self.tab;
                ui.selectable_value(&mut tab, MainTab::Work, l.tab_work);
                ui.selectable_value(&mut tab, MainTab::System, l.tab_system);
                ui.selectable_value(&mut tab, MainTab::Settings, l.tab_settings);
                if tab != self.tab {
                    self.set_tab(tab);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(l.help_f1).clicked() {
                        self.open_help_contextual();
                    }
                });
            });
            ui.separator();
            self.ui_status_strip(ui);
        });

        if self.help_open {
            egui::SidePanel::right("help-panel")
                .default_width(320.0)
                .show(ctx, |ui| {
                    self.ui_help_panel(ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(l.heading);
                ui.label(l.subtitle);
                ui.separator();
                match self.tab {
                    MainTab::Work => self.ui_work(ui, ctx),
                    MainTab::System => self.ui_system(ui, ctx),
                    MainTab::Settings => self.ui_settings(ui, ctx),
                }
                let mut open_problem_help: Option<HelpId> = None;
                if let Some(problem) = &self.last_problem {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(200, 60, 60), &problem.message);
                    // Phase R `#290`: human next step primary; wire ids secondary.
                    let action_view = crate::problem_action::ProblemActionView::from_problem(
                        problem,
                        self.ui_lang(),
                    );
                    if action_view.has_human_next_step() {
                        if let Some(next) = action_view.next_step {
                            ui.label(next);
                        }
                    }
                    ui.small(format!(
                        "{} · help:{}",
                        action_view.code_wire, action_view.help_wire
                    ));
                    if let Some(try_wire) = action_view.try_wire {
                        ui.small(format!(
                            "try:{} · help:{}",
                            try_wire,
                            problem
                                .corrective
                                .map(|a| a.help_id().as_str())
                                .unwrap_or(action_view.help_wire)
                        ));
                    }
                    // Keep action catalog reachable for F1 wiring (#263).
                    let _ = crate::lexicon::ActionId::catalog().len();
                    if ui.small_button(l.help_open_topic).clicked() {
                        open_problem_help = Some(problem.help_id);
                    }
                    if let Some(detail) = &problem.detail {
                        egui::CollapsingHeader::new(l.work_details)
                            .id_source("problem-detail")
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.monospace(detail);
                            });
                    }
                }
                if let Some(topic) = open_problem_help {
                    self.open_help(topic);
                }
            });
        });

        // Repaint schedule only — does not load status/mesh by itself.
        ctx.request_repaint_after(crate::async_jobs::STATUS_REFRESH_INTERVAL);
    }
}

impl AiraDesktopApp {
    /// Compact status strip on every main screen (`desktop-ux` §2.2 / `#259`).
    /// Network cell: human phrase via `#289` `mesh_language` (agrees with Connection).
    fn ui_status_strip(&self, ui: &mut egui::Ui) {
        let l = self.labels();
        let work = if self.async_jobs.work_inflight() {
            l.strip_work_busy
        } else if let Some(problem) = &self.last_problem {
            // Phase R `#290`: actionable strip hint when corrective is known.
            crate::problem_action::strip_work_from_problem(
                problem,
                self.ui_lang(),
                l.strip_work_action,
            )
        } else {
            l.strip_work_ready
        };
        let quality = self.system_snapshot.network_quality;
        let network = match crate::mesh_language::strip_network_from_top_level(
            quality,
            &self.mesh_snapshot.top_level,
        ) {
            crate::mesh_language::StripNetworkPhrase::Connected => l.strip_net_connected,
            crate::mesh_language::StripNetworkPhrase::LocalOnly => l.strip_net_local,
            crate::mesh_language::StripNetworkPhrase::Stale => l.strip_net_stale,
            crate::mesh_language::StripNetworkPhrase::NotChecked => l.strip_net_unknown,
            crate::mesh_language::StripNetworkPhrase::Offline => l.strip_net_offline,
        };
        let model =
            match aira_desktop_runtime::ModelTripleConclusion::from_triple(&self.model_triple) {
                aira_desktop_runtime::ModelTripleConclusion::NoneSelected => l.strip_model_none,
                aira_desktop_runtime::ModelTripleConclusion::SelectedNotReady => {
                    l.strip_model_not_ready
                }
                aira_desktop_runtime::ModelTripleConclusion::Ready => l.strip_model_ready,
                aira_desktop_runtime::ModelTripleConclusion::UsedInResult => l.strip_model_used,
            };
        ui.horizontal(|ui| {
            ui.small(format!("{} {}", l.strip_work, work));
            ui.separator();
            ui.small(format!("{} {}", l.strip_model, model));
            ui.separator();
            ui.small(format!("{} {}", l.strip_network, network));
        });
    }

    /// Offline F1 panel: search, topics, related links, embedded Markdown (`#263`/`#273`).
    fn ui_help_panel(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
        let lang = self.ui_lang();
        ui.horizontal(|ui| {
            ui.heading(l.help_panel_title);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(l.help_close).clicked() {
                    self.close_help();
                }
            });
        });
        ui.small(l.help_offline_note);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(l.help_search);
            ui.add(
                egui::TextEdit::singleline(&mut self.help_search)
                    .desired_width(180.0)
                    .hint_text(l.help_search),
            );
        });
        let hits = crate::help::search_help_ids(lang, &self.help_search);
        ui.strong(l.help_topics);
        ui.horizontal_wrapped(|ui| {
            for id in &hits {
                let title = crate::help::load_article(lang, *id).title;
                let selected = self.help_topic == *id;
                if ui.selectable_label(selected, title).clicked() {
                    self.help_topic = *id;
                }
            }
        });
        if hits.is_empty() {
            ui.label(l.help_placeholder);
        } else if !hits.contains(&self.help_topic) {
            // Keep pinned topic; search filters the list only (`#273`).
            ui.small(l.help_search_miss);
        }
        ui.separator();
        let article = crate::help::load_article(lang, self.help_topic);
        ui.heading(&article.title);
        ui.horizontal(|ui| {
            ui.strong(l.help_topic_label);
            ui.monospace(article.id.as_str());
        });
        let related = crate::help::extract_help_links(article.markdown);
        if !related.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.strong(l.help_related);
                for id in &related {
                    let title = crate::help::load_article(lang, *id).title;
                    if ui.selectable_label(false, title).clicked() {
                        self.help_topic = *id;
                        self.help_search.clear();
                    }
                }
            });
        }
        ui.separator();
        let body = crate::help::render_markdown_plain(article.markdown);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(body);
        });
    }

    fn ui_mesh_status(&self, ui: &mut egui::Ui) {
        let l = self.labels();
        let snap = &self.mesh_snapshot;
        ui.heading(l.mesh_heading);
        // Phase R `#289`: human conclusion primary; raw top-level secondary/tech.
        let conclusion = crate::system_view::ConnectionConclusion::from_top_level(&snap.top_level);
        let human = match conclusion {
            crate::system_view::ConnectionConclusion::Direct => l.sys_conn_direct,
            crate::system_view::ConnectionConclusion::Relayed => l.sys_conn_relayed,
            crate::system_view::ConnectionConclusion::OutboundOnly => l.sys_conn_outbound,
            crate::system_view::ConnectionConclusion::LocalOnly => l.sys_conn_local,
            crate::system_view::ConnectionConclusion::Unknown => l.sys_conn_unknown,
            crate::system_view::ConnectionConclusion::Offline => l.sys_conn_offline,
        };
        let banner_color = match conclusion {
            crate::system_view::ConnectionConclusion::Direct => {
                egui::Color32::from_rgb(40, 140, 70)
            }
            crate::system_view::ConnectionConclusion::Relayed => {
                egui::Color32::from_rgb(40, 100, 180)
            }
            crate::system_view::ConnectionConclusion::OutboundOnly => {
                egui::Color32::from_rgb(180, 120, 40)
            }
            crate::system_view::ConnectionConclusion::LocalOnly => {
                egui::Color32::from_rgb(100, 100, 140)
            }
            crate::system_view::ConnectionConclusion::Unknown => {
                egui::Color32::from_rgb(120, 120, 120)
            }
            crate::system_view::ConnectionConclusion::Offline => {
                egui::Color32::from_rgb(140, 60, 60)
            }
        };
        ui.colored_label(banner_color, human);
        ui.horizontal(|ui| {
            ui.small(l.mesh_banner);
            ui.monospace(&snap.top_level);
        });
        let na = l.mesh_na;
        let yes = l.mesh_yes;
        let no = l.mesh_no;
        let identity = if snap.identity.is_empty() {
            na
        } else {
            snap.identity.as_str()
        };
        ui.horizontal(|ui| {
            ui.strong(l.mesh_identity);
            ui.monospace(identity);
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_preferred_port);
            ui.label(
                snap.preferred_port
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| na.to_string()),
            );
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_local_bind);
            ui.monospace(snap.local_bind.as_deref().unwrap_or(na));
            if snap.local_bind.is_some() {
                ui.small(format!(
                    "({} {})",
                    l.mesh_bind_provenance,
                    snap.local_bind_provenance.as_str()
                ));
            }
            if !snap.local_listener_proven {
                ui.small(l.mesh_listener_unproven);
            }
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_external);
            ui.monospace(snap.external_observed.as_deref().unwrap_or(na));
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_reachability);
            ui.monospace(&snap.reachability_status);
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_direct);
            ui.label(if snap.direct_reachability == "yes" {
                yes
            } else {
                no
            });
            ui.strong(l.mesh_relay);
            ui.label(if snap.relay_reachability == "yes" {
                yes
            } else {
                no
            });
        });
        let rv = if snap.rendezvous_provider.is_empty() {
            na.to_string()
        } else {
            format!(
                "{} · seq {} · {}",
                snap.rendezvous_provider,
                snap.rendezvous_sequence,
                if snap.rendezvous_connected {
                    l.mesh_rendezvous_local_meta
                } else {
                    no
                }
            )
        };
        ui.horizontal(|ui| {
            ui.strong(l.mesh_rendezvous);
            ui.label(rv);
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_address_book_count);
            ui.label(snap.address_book_count.to_string());
        });
        ui.horizontal(|ui| {
            ui.strong(l.mesh_live_sessions);
            ui.label(match snap.live_session_count {
                Some(n) => n.to_string(),
                None => na.to_string(),
            });
        });
    }

    fn ui_work(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.heading(l.work_heading);
        ui.label(l.work_hint);
        ui.small(l.work_shortcut_hint);
        let editor = ui.add(
            egui::TextEdit::multiline(&mut self.problem_text)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );
        if editor.gained_focus() || editor.changed() {
            self.note_help_focus(HelpId::WorkSubmit);
        }

        // Ctrl+Enter / ⌘+Enter — Enter alone stays newline (`#260`).
        let shortcut_run = ui.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Enter,
            )) || i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Enter,
            ))
        });

        let submitting = self.async_jobs.work_inflight();
        let mut do_submit = false;
        ui.add_enabled_ui(!submitting, |ui| {
            let btn = ui.button(l.work_submit);
            if btn.hovered() {
                self.note_help_focus(HelpId::WorkSubmit);
            }
            if btn.clicked() {
                do_submit = true;
            }
        });
        if !submitting && shortcut_run {
            do_submit = true;
        }
        if do_submit {
            // Draft (`problem_text`) is never cleared here — only cloned for submit.
            self.submit_work(ctx);
        }
        if submitting {
            ui.label(l.work_submitting);
        }
        ui.label(l.work_user_note);
        egui::CollapsingHeader::new(l.work_how_it_works)
            .id_source("work-tech-note")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(l.work_tech_details);
            });
        if self.work_result.is_some() {
            ui.separator();
            let ans = ui.strong(l.work_answer);
            if ans.hovered() {
                self.note_help_focus(HelpId::WorkResult);
            }
        }
        if let Some(view) = &self.work_result {
            let answer = if view.answer.is_empty() {
                l.work_no_answer
            } else {
                view.answer.as_str()
            };
            ui.heading(answer);
            let status_human = if view.status.eq_ignore_ascii_case("completed") {
                l.work_status_completed
            } else if view.status.eq_ignore_ascii_case("executed") {
                l.work_status_executed
            } else if view.status.eq_ignore_ascii_case("needs_human_collapse") {
                l.work_status_needs_human
            } else {
                view.status.as_str()
            };
            ui.horizontal(|ui| {
                ui.strong(l.work_run_status);
                ui.label(status_human);
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
            ui.horizontal(|ui| {
                ui.strong(l.work_provenance);
                ui.label(self.work_provenance_label(view.provenance));
            });
            let has_ids = view.problem_id.is_some()
                || view.verified_artifact_id.is_some()
                || view.execution_artifact_id.is_some()
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
                        if let Some(id) = &view.execution_artifact_id {
                            ui.horizontal(|ui| {
                                ui.strong(l.work_execution_id);
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

    fn work_provenance_label(&self, kind: crate::work_view::ProvenanceKind) -> &'static str {
        let l = self.labels();
        match kind {
            crate::work_view::ProvenanceKind::VerifiedLocalCompute => l.work_prov_verified,
            crate::work_view::ProvenanceKind::MockGenerate => l.work_prov_mock,
            crate::work_view::ProvenanceKind::LocalGenerateExecuted => l.work_prov_generate,
            crate::work_view::ProvenanceKind::ModelUndefined => l.work_prov_model_unknown,
            crate::work_view::ProvenanceKind::NeedsAttention => l.work_prov_attention,
        }
    }

    /// System status: Program / Model / Connection / Events (`#261`).
    fn ui_system(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.heading(l.system_heading);
        let view = crate::system_view::SystemStatusView::from_parts(
            self.lifecycle,
            &self.system_snapshot,
            &self.model_triple,
        );

        self.ui_sys_program(ui, ctx, &view);
        ui.separator();
        self.ui_sys_model(ui, &view);
        ui.separator();
        self.ui_sys_connection(ui, ctx, &view);
        ui.separator();
        self.ui_sys_events(ui);
    }

    fn ui_sys_program(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        view: &crate::system_view::SystemStatusView,
    ) {
        let l = self.labels();
        ui.strong(l.sys_program);
        ui.label(match view.program {
            crate::system_view::ProgramConclusion::Running => l.sys_prog_running,
            crate::system_view::ProgramConclusion::Stopped => l.sys_prog_stopped,
            crate::system_view::ProgramConclusion::Starting => l.sys_prog_starting,
            crate::system_view::ProgramConclusion::Stopping => l.sys_prog_stopping,
            crate::system_view::ProgramConclusion::Unhealthy => l.sys_prog_unhealthy,
            crate::system_view::ProgramConclusion::Failed => l.sys_prog_failed,
        });
        ui.horizontal(|ui| {
            match view.program {
                crate::system_view::ProgramConclusion::Stopped
                | crate::system_view::ProgramConclusion::Failed => {
                    let btn = ui.button(l.start);
                    if btn.hovered() {
                        self.note_help_focus(HelpId::NodeLifecycle);
                    }
                    if btn.clicked() {
                        self.note_help_focus(HelpId::NodeLifecycle);
                        self.request_lifecycle(crate::async_jobs::LifecycleJobKind::Start, ctx);
                    }
                }
                crate::system_view::ProgramConclusion::Running
                | crate::system_view::ProgramConclusion::Unhealthy => {
                    let btn = ui.button(l.stop);
                    if btn.hovered() {
                        self.note_help_focus(HelpId::NodeLifecycle);
                    }
                    if btn.clicked() {
                        self.note_help_focus(HelpId::NodeLifecycle);
                        self.request_lifecycle(crate::async_jobs::LifecycleJobKind::Stop, ctx);
                    }
                }
                crate::system_view::ProgramConclusion::Starting
                | crate::system_view::ProgramConclusion::Stopping => {}
            }
            if ui.button(l.refresh).clicked() {
                self.request_status_refresh(ctx);
                self.refresh_federation_detail();
            }
            if ui.button(l.quit).clicked() {
                self.request_quit(ctx);
            }
        });
        if self.restart_hint || self.settings_need_restart() {
            ui.colored_label(egui::Color32::from_rgb(180, 120, 40), l.restart_hint);
        }
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("sys-program-tech")
            .default_open(false)
            .show(ui, |ui| {
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
            });
    }

    fn ui_sys_model(&mut self, ui: &mut egui::Ui, view: &crate::system_view::SystemStatusView) {
        let l = self.labels();
        let heading = ui.strong(l.sys_model);
        if heading.hovered() {
            self.note_help_focus(HelpId::ModelSelect);
        }
        ui.horizontal(|ui| {
            ui.label(l.sys_model_selected);
            ui.monospace(view.model.selected.as_display());
        });
        ui.horizontal(|ui| {
            ui.label(l.sys_model_ready);
            ui.label(if view.model.ready {
                l.mesh_yes
            } else {
                l.mesh_no
            });
        });
        ui.horizontal(|ui| {
            ui.label(l.sys_model_used);
            ui.monospace(view.model.used.as_display());
        });
        ui.small(l.sys_model_triple_hint);
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("sys-model-tech")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(l.not_llm);
                ui.label(format!("ready_detail: {}", view.model.ready_detail));
                ui.label(format!("summary: {}", view.model.summary.as_str()));
            });
    }

    fn ui_sys_connection(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        view: &crate::system_view::SystemStatusView,
    ) {
        let l = self.labels();
        let heading = ui.strong(l.sys_connection);
        if heading.hovered() {
            self.note_help_focus(HelpId::NetworkReachability);
        }
        ui.label(match view.connection {
            crate::system_view::ConnectionConclusion::Direct => l.sys_conn_direct,
            crate::system_view::ConnectionConclusion::Relayed => l.sys_conn_relayed,
            crate::system_view::ConnectionConclusion::OutboundOnly => l.sys_conn_outbound,
            crate::system_view::ConnectionConclusion::LocalOnly => l.sys_conn_local,
            crate::system_view::ConnectionConclusion::Unknown => l.sys_conn_unknown,
            crate::system_view::ConnectionConclusion::Offline => l.sys_conn_offline,
        });
        ui.horizontal(|ui| {
            ui.small(format!("{} {}", l.sys_observed, view.observed_at));
            ui.small(format!("{} {}", l.sys_loaded, view.loaded_at));
            ui.small(format!(
                "{} {}",
                l.sys_quality,
                view.network_quality.as_str()
            ));
        });
        ui.horizontal(|ui| {
            ui.label(l.sys_saved_participants);
            ui.label(view.address_book_count.to_string());
        });
        ui.horizontal(|ui| {
            ui.label(l.sys_live_sessions);
            ui.label(match view.live_session_count {
                Some(n) => n.to_string(),
                None => l.sys_live_unobserved.to_string(),
            });
        });
        // Phase S `#299`: always-visible Connection boundary (setup ≠ remote; loopback ≠ dial).
        let boundary = ui.label(
            egui::RichText::new(l.conn_boundary_guidance)
                .size(12.0)
                .color(ui.visuals().weak_text_color()),
        );
        if boundary.hovered() {
            self.note_help_focus(HelpId::NetworkConnect);
        }
        // Phase R `#293`: one guidance line for P0 + empty book (no fake Applied/CONNECTED).
        if crate::cold_start::is_cold_start_empty_profile(
            self.settings.network_profile,
            view.address_book_count,
        ) {
            ui.label(l.conn_cold_start_guidance);
        }
        // Phase R `#287`: exactly one primary Connection CTA (not Refresh-only).
        let cta = crate::connection_cta::primary_connection_cta(
            crate::connection_cta::ConnectionCtaInput {
                profile: self.settings.network_profile,
                connection: view.connection,
                address_book_count: view.address_book_count,
                apply_phase: self.settings_apply_phase(),
                program: view.program,
                restart_hint: self.restart_hint,
            },
        );
        if cta.is_button() {
            let label = match cta {
                crate::connection_cta::ConnectionPrimaryCta::EnablePrivateNetwork => {
                    l.cta_enable_private_network
                }
                crate::connection_cta::ConnectionPrimaryCta::ImportInvite => l.cta_import_invite,
                crate::connection_cta::ConnectionPrimaryCta::StopToApply => l.cta_stop_to_apply,
                crate::connection_cta::ConnectionPrimaryCta::StartToApply => l.cta_start_to_apply,
                crate::connection_cta::ConnectionPrimaryCta::RefreshStatus => l.refresh,
                crate::connection_cta::ConnectionPrimaryCta::NoneOk => l.refresh,
            };
            let help = cta.help_id();
            let btn = ui.button(label);
            if btn.hovered() {
                self.note_help_focus(help);
            }
            if btn.clicked() {
                self.note_help_focus(help);
                match cta {
                    crate::connection_cta::ConnectionPrimaryCta::EnablePrivateNetwork => {
                        self.apply_profile(NetworkProfile::P1);
                    }
                    crate::connection_cta::ConnectionPrimaryCta::ImportInvite => {
                        self.import_json_dialog();
                    }
                    crate::connection_cta::ConnectionPrimaryCta::StopToApply => {
                        self.request_lifecycle(crate::async_jobs::LifecycleJobKind::Stop, ctx);
                    }
                    crate::connection_cta::ConnectionPrimaryCta::StartToApply => {
                        self.request_lifecycle(crate::async_jobs::LifecycleJobKind::Start, ctx);
                    }
                    crate::connection_cta::ConnectionPrimaryCta::RefreshStatus => {
                        self.request_status_refresh(ctx);
                        self.refresh_federation_detail();
                    }
                    crate::connection_cta::ConnectionPrimaryCta::NoneOk => {}
                }
            }
        }
        // Phase R `#288`: invite + P0–P2 path visible without opening Technical details.
        ui.separator();
        self.ui_connect_primary(ui, ctx);
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("sys-connection-tech")
            .default_open(false)
            .show(ui, |ui| {
                // Raw enum secondary (`#289`); human conclusion already above.
                ui.small(format!(
                    "top_level:{} · live_q:{}",
                    view.top_level,
                    view.live_sessions_quality.as_str()
                ));
                self.ui_mesh_status(ui);
                ui.separator();
                self.ui_network_advanced(ui);
            });
    }

    fn ui_sys_events(&self, ui: &mut egui::Ui) {
        let l = self.labels();
        ui.strong(l.sys_events);
        let mut any = false;
        if let Some(problem) = &self.last_problem {
            any = true;
            ui.colored_label(egui::Color32::from_rgb(200, 60, 60), &problem.message);
            let action_view =
                crate::problem_action::ProblemActionView::from_problem(problem, self.ui_lang());
            if let Some(next) = action_view.next_step {
                ui.label(next);
            }
            ui.small(format!(
                "{} · {}",
                action_view.code_wire, action_view.help_wire
            ));
        }
        if let Some(msg) = &self.discovery_msg {
            any = true;
            ui.label(msg);
        }
        if let Some(msg) = &self.invite_msg {
            any = true;
            ui.label(msg);
        }
        if self.restart_hint || self.settings_need_restart() {
            any = true;
            ui.colored_label(egui::Color32::from_rgb(180, 120, 40), l.restart_hint);
        }
        if !any {
            ui.label(l.sys_events_empty);
        }
    }

    /// Primary connect path: P0–P2 profile + invite/QR (`#288`).
    ///
    /// Rendered on Connection outside Technical details.
    fn ui_connect_primary(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        let profile_h = ui.heading(l.network_profile);
        if profile_h.hovered() {
            self.note_help_focus(HelpId::NetworkConnect);
        }
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
        let invite_h = ui.heading(l.friend_invite);
        if invite_h.hovered() {
            self.note_help_focus(HelpId::NetworkTrust);
        }
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

    /// Advanced Connection ops kept under Technical details (`#288`).
    fn ui_network_advanced(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
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
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.heading(l.settings_heading);
        let phase = self.settings_apply_phase();
        match phase {
            crate::settings_apply::SettingsApplyPhase::Undefined => {
                ui.colored_label(
                    egui::Color32::from_rgb(120, 120, 120),
                    l.settings_phase_undefined,
                );
            }
            crate::settings_apply::SettingsApplyPhase::Applied => {
                ui.colored_label(
                    egui::Color32::from_rgb(40, 140, 70),
                    l.settings_phase_applied,
                );
            }
            crate::settings_apply::SettingsApplyPhase::RestartNeeded => {
                ui.colored_label(
                    egui::Color32::from_rgb(180, 120, 40),
                    l.settings_phase_restart,
                );
                ui.small(l.settings_restart_action);
            }
        }
        ui.label(l.settings_close_not_stop);

        ui.separator();
        let g = ui.strong(l.settings_group_general);
        if g.hovered() {
            self.note_help_focus(HelpId::SettingsApply);
        }
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
        let mut dirty = false;
        dirty |= ui
            .checkbox(&mut self.settings.open_ui_on_start, l.open_window_on_login)
            .changed();
        ui.label(l.open_window_hint);
        dirty |= ui
            .checkbox(&mut self.settings.autostart_on_login, l.autostart)
            .changed();
        if dirty {
            if let Err(e) = self.persist_settings() {
                self.set_problem(
                    crate::lexicon::ErrorCode::SettingsPersistFailed,
                    format!("{e:#}"),
                );
            }
        }

        ui.separator();
        let g = ui.strong(l.settings_group_models);
        if g.hovered() {
            self.note_help_focus(HelpId::ModelSelect);
        }
        ui.horizontal(|ui| {
            ui.label(l.sys_model_selected);
            ui.monospace(self.model_triple.selected.as_display());
        });
        ui.horizontal(|ui| {
            ui.label(l.sys_model_ready);
            ui.label(if self.model_triple.ready {
                l.mesh_yes
            } else {
                l.mesh_no
            });
        });
        ui.horizontal(|ui| {
            ui.label(l.sys_model_used);
            ui.monospace(self.model_triple.used.as_display());
        });
        ui.small(l.settings_models_observe_only);
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("settings-models-tech")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(l.not_llm);
                ui.label(format!("ready_detail: {}", self.model_triple.ready_detail));
            });

        ui.separator();
        let g = ui.strong(l.settings_group_connection);
        if g.hovered() {
            self.note_help_focus(HelpId::SettingsApply);
        }
        let saved_profile = format!("{:?}", self.settings.network_profile);
        let applied_profile = self
            .applied_runtime
            .as_ref()
            .map(|a| format!("{:?}", a.network_profile))
            .unwrap_or_else(|| l.settings_applied_undefined.to_string());
        ui.horizontal(|ui| {
            ui.strong(l.settings_saved);
            ui.label(&saved_profile);
        });
        ui.horizontal(|ui| {
            ui.strong(l.settings_applied);
            ui.label(&applied_profile);
        });
        let saved_listen = self
            .settings
            .peer_listen
            .as_deref()
            .unwrap_or(l.peer_off_p0);
        let applied_listen = self
            .applied_runtime
            .as_ref()
            .map(|a| a.peer_listen.as_deref().unwrap_or(l.peer_off_p0))
            .unwrap_or(l.settings_applied_undefined);
        ui.horizontal(|ui| {
            ui.label(l.peer_listen);
            ui.strong(l.settings_saved);
            ui.monospace(saved_listen);
        });
        ui.horizontal(|ui| {
            ui.label(l.peer_listen);
            ui.strong(l.settings_applied);
            ui.monospace(applied_listen);
        });
        ui.small(l.network_profile);

        ui.separator();
        let g = ui.strong(l.settings_group_advanced);
        if g.hovered() {
            self.note_help_focus(HelpId::SettingsApply);
        }
        ui.horizontal(|ui| {
            ui.strong(l.settings_saved);
            ui.label(format!("HTTP {}", self.settings.http_listen));
        });
        ui.horizontal(|ui| {
            ui.strong(l.settings_applied);
            ui.label(match self.applied_runtime.as_ref() {
                Some(a) => format!("HTTP {}", a.http_listen),
                None => l.settings_applied_undefined.to_string(),
            });
        });
        ui.label(format!("instance: {}", self.settings.instance_id));
    }
}
