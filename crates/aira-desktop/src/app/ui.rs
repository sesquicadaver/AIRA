use aira_desktop_runtime::{
    CatalogProjectionRow, CatalogSelection, CatalogSource, NetworkProfile, UiLang,
    DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;
use crate::lexicon::{ErrorCode, HelpId};

use super::{work, AiraDesktopApp, MainTab};

/// Which Work field a shared catalog row writes. Never the default tip.
#[derive(Clone, Copy)]
enum WorkPick {
    Required,
    CompareA,
    CompareB,
}

/// Closed Work combo. Empty Compare B is a prompt, not the tip (`#360`).
fn work_selector_closed_label(
    pick: WorkPick,
    selected_name: Option<&str>,
    default_word: &str,
    default_name: &str,
    empty_compare_b: &str,
) -> String {
    if let Some(name) = selected_name {
        return name.to_string();
    }
    if matches!(pick, WorkPick::CompareB) {
        return empty_compare_b.to_string();
    }
    format!("{default_word}: {default_name}")
}

/// Lines under «Not ready». Compare keeps B visible when A already succeeded (`#360`).
fn work_readiness_lines(comparing: bool, reasons: &[String]) -> Vec<&str> {
    if comparing {
        reasons.iter().map(String::as_str).collect()
    } else {
        reasons.first().map(String::as_str).into_iter().collect()
    }
}

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
            ui.horizontal_wrapped(|ui| {
                let mut tab = self.tab;
                ui.selectable_value(&mut tab, MainTab::Work, l.tab_work);
                ui.selectable_value(&mut tab, MainTab::System, l.tab_system);
                ui.selectable_value(&mut tab, MainTab::Settings, l.tab_settings);
                if tab != self.tab {
                    self.set_tab(tab);
                }
                if ui.button(l.help_f1).clicked() {
                    self.open_help_contextual();
                }
            });
            ui.separator();
            self.ui_status_strip(ui);
        });

        if self.help_open {
            let screen_w = ctx.screen_rect().width();
            if crate::window_check::central_width_with_f1(screen_w) < screen_w {
                egui::SidePanel::right("help-panel")
                    .exact_width(crate::window_check::HELP_DOCK_WIDTH)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.ui_help_panel(ui, true);
                    });
            } else {
                let w = (screen_w - 16.0).clamp(240.0, crate::window_check::HELP_DOCK_WIDTH);
                let mut open = true;
                egui::Window::new(l.help_panel_title)
                    .id(egui::Id::new("help-overlay"))
                    .collapsible(false)
                    .resizable(true)
                    .default_width(w)
                    .default_pos(egui::pos2((screen_w - w - 8.0).max(0.0), 36.0))
                    .open(&mut open)
                    .show(ctx, |ui| {
                        self.ui_help_panel(ui, false);
                    });
                if !open {
                    self.close_help();
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
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
                    if ui.small_button(l.help_open_topic).clicked() {
                        open_problem_help = Some(problem.help_id);
                    }
                    egui::CollapsingHeader::new(l.work_details)
                        .id_source("problem-detail")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.small(format!(
                                "{} · help:{}",
                                action_view.code_wire, action_view.help_wire
                            ));
                            if let Some(try_wire) = action_view.try_wire {
                                ui.small(format!("try:{try_wire}"));
                            }
                            if let Some(detail) = &problem.detail {
                                ui.monospace(detail);
                            }
                        });
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
    /// Paint-safe catalog rows (`#380`): memory lookup only — no tip/cache I/O.
    fn catalog_rows(&self) -> &[CatalogProjectionRow] {
        &self.catalog_projection
    }

    fn projection_label(l: &crate::app::i18n::Labels, row: &CatalogProjectionRow) -> String {
        let source = match row.source {
            CatalogSource::HostOllama => l.settings_models_source_ollama,
            CatalogSource::LocalFile => l.settings_models_source_file,
        };
        let avail = if row.available {
            l.settings_models_available
        } else {
            l.settings_models_unavailable
        };
        if row.verified {
            format!(
                "{} · {} · {} · {}",
                row.name, source, l.settings_models_verified, avail
            )
        } else {
            format!("{} · {} · {}", row.name, source, avail)
        }
    }

    fn row_name_for_ref(&self, model_ref: &str) -> String {
        self.catalog_rows()
            .iter()
            .find(|r| r.model_ref == model_ref)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| aira_desktop_runtime::catalog_display_name(model_ref))
    }

    fn default_selector_name(&self) -> String {
        if let Some(tip) = &self.model_catalog.tip_model_ref {
            return self.row_name_for_ref(tip);
        }
        self.settings
            .llm_ollama_model
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "—".into())
    }

    /// One model selector. Compare adds a second combo, not a second catalog.
    fn ui_model_combo(&mut self, ui: &mut egui::Ui, pick: WorkPick) {
        let l = self.labels();
        // Clone the paint snapshot — no tip/cache I/O (`#380`).
        let rows = self.catalog_projection.clone();
        let current = match pick {
            WorkPick::Required if self.work_executor_mode == work::WorkExecutorUiMode::Auto => None,
            WorkPick::Required => Some(self.work_required_ref.clone()).filter(|s| !s.is_empty()),
            WorkPick::CompareA => Some(self.work_compare_a.clone()).filter(|s| !s.is_empty()),
            WorkPick::CompareB => Some(self.work_compare_b.clone()).filter(|s| !s.is_empty()),
        };
        let selected_name = current.as_deref().map(|r| self.row_name_for_ref(r));
        let selected_text = work_selector_closed_label(
            pick,
            selected_name.as_deref(),
            l.work_selector_default,
            &self.default_selector_name(),
            l.work_compare_b_empty,
        );
        let id = match pick {
            WorkPick::Required => "work-model",
            WorkPick::CompareA => "work-model-a",
            WorkPick::CompareB => "work-model-b",
        };
        egui::ComboBox::from_id_source(id)
            .selected_text(selected_text)
            .width(ui.available_width().clamp(160.0, 520.0))
            .wrap()
            .show_ui(ui, |ui| {
                if matches!(pick, WorkPick::Required) {
                    let default_label = format!(
                        "{}: {}",
                        l.work_selector_default,
                        self.default_selector_name()
                    );
                    if ui
                        .selectable_label(current.is_none(), default_label)
                        .clicked()
                    {
                        self.work_executor_mode = work::WorkExecutorUiMode::Auto;
                        self.refresh_work_readiness();
                    }
                }
                for row in &rows {
                    let on = current.as_deref() == Some(row.model_ref.as_str());
                    if ui
                        .selectable_label(on, Self::projection_label(l, row))
                        .clicked()
                    {
                        let model_ref = row.model_ref.clone();
                        match pick {
                            WorkPick::Required => {
                                self.work_required_ref = model_ref;
                                self.work_executor_mode = work::WorkExecutorUiMode::Specific;
                            }
                            WorkPick::CompareA => self.work_compare_a = model_ref,
                            WorkPick::CompareB => self.work_compare_b = model_ref,
                        }
                        self.refresh_work_readiness();
                    }
                }
            });
    }

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
        } else if self.work_readiness.ready {
            l.strip_work_ready
        } else {
            l.strip_work_unready
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
                aira_desktop_runtime::ModelTripleConclusion::Ready => {
                    // #319: activate-ready + mock must not read as configured LLM.
                    if self.model_triple.executor_is_reference_mock() {
                        l.strip_model_ready_reference
                    } else {
                        l.strip_model_ready
                    }
                }
                aira_desktop_runtime::ModelTripleConclusion::UsedInResult => l.strip_model_used,
            };
        ui.horizontal_wrapped(|ui| {
            ui.small(format!("{} {}", l.strip_work, work));
            ui.separator();
            ui.small(format!("{} {}", l.strip_model, model));
            ui.separator();
            ui.small(format!("{} {}", l.strip_network, network));
        });
    }

    /// Offline F1 panel: search, topics, related links, embedded Markdown (`#263`/`#273`).
    fn ui_help_panel(&mut self, ui: &mut egui::Ui, docked: bool) {
        let l = self.labels();
        let lang = self.ui_lang();
        if docked {
            ui.horizontal_wrapped(|ui| {
                ui.heading(l.help_panel_title);
                if ui.button(l.help_close).clicked() {
                    self.close_help();
                }
            });
        }
        ui.small(l.help_offline_note);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(l.help_search);
            ui.add(
                egui::TextEdit::singleline(&mut self.help_search)
                    .desired_width(ui.available_width().clamp(80.0, 220.0))
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
        if let Some(ev) = &snap.last_confirmed_handshake {
            ui.horizontal(|ui| {
                ui.strong(l.mesh_last_handshake);
                ui.label(ev);
            });
        }
    }

    fn ui_work(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.heading(l.work_heading);

        if self.model_triple.executor_is_reference_mock() {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(255, 236, 179))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(180, 120, 40),
                ))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.colored_label(egui::Color32::from_rgb(120, 70, 10), l.work_mock_banner);
                    if ui.button(l.work_mock_banner_cta).clicked() {
                        self.set_tab(MainTab::Settings);
                        self.note_help_focus(HelpId::ModelSelect);
                    }
                });
            ui.add_space(6.0);
        }

        let comparing = self.work_executor_mode == work::WorkExecutorUiMode::Compare;
        ui.horizontal(|ui| {
            self.ui_model_combo(
                ui,
                if comparing {
                    WorkPick::CompareA
                } else {
                    WorkPick::Required
                },
            );
            if ui
                .selectable_label(comparing, l.work_executor_compare)
                .clicked()
            {
                if comparing {
                    self.work_executor_mode = work::WorkExecutorUiMode::Auto;
                } else {
                    self.work_executor_mode = work::WorkExecutorUiMode::Compare;
                    if self.work_compare_a.is_empty() {
                        let picked = self.work_required_ref.trim();
                        self.work_compare_a = if picked.is_empty() {
                            self.model_catalog.tip_model_ref.clone().unwrap_or_default()
                        } else {
                            picked.to_string()
                        };
                    }
                }
                self.refresh_work_readiness();
            }
        });
        if comparing {
            ui.horizontal(|ui| {
                ui.label(l.work_compare_b);
                self.ui_model_combo(ui, WorkPick::CompareB);
            });
        }
        if !self.work_readiness.ready {
            ui.colored_label(
                egui::Color32::from_rgb(180, 120, 40),
                l.work_readiness_blocked,
            );
            for reason in work_readiness_lines(comparing, &self.work_readiness.reasons) {
                ui.small(reason);
            }
        }

        let editor = ui.add(
            egui::TextEdit::multiline(&mut self.problem_text)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );
        if editor.gained_focus() || editor.changed() {
            self.note_help_focus(HelpId::WorkSubmit);
            if editor.changed() {
                self.refresh_work_readiness();
            }
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
        let can_run = self.work_can_run();
        let prepare = self.prepare_and_run_names();
        let mut do_submit = false;
        let mut do_prepare = false;
        if prepare.is_some() {
            let btn = ui.button(l.work_prepare_and_run);
            if btn.hovered() {
                self.note_help_focus(HelpId::WorkSubmit);
            }
            if btn.clicked() {
                do_prepare = true;
            }
        } else {
            ui.add_enabled_ui(can_run, |ui| {
                let btn = ui.button(l.work_submit);
                if btn.hovered() {
                    self.note_help_focus(HelpId::WorkSubmit);
                }
                if btn.clicked() {
                    do_submit = true;
                }
            });
        }
        if shortcut_run {
            if prepare.is_some() {
                do_prepare = true;
            } else if can_run {
                do_submit = true;
            }
        }
        if do_prepare {
            self.prepare_and_run(ctx);
        }
        if do_submit {
            // Draft (`problem_text`) is never cleared here — only cloned for submit.
            self.submit_work(ctx);
        }
        if submitting {
            ui.label(l.work_submitting);
        }
        egui::CollapsingHeader::new(l.work_how_it_works)
            .id_source("work-tech-note")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(l.work_tech_details);
                ui.small(l.work_hint);
                ui.small(l.work_shortcut_hint);
                ui.small(l.work_executor);
                ui.small(l.work_executor_auto);
                ui.small(l.work_executor_specific);
                ui.small(l.work_executor_hint);
                ui.small(l.work_compare_pick_a);
                ui.small(l.work_compare_pick_b);
                ui.small(l.work_capability_math);
                ui.small(l.work_capability_generate);
                ui.small(l.work_readiness_ready);
                ui.small(l.work_user_note);
                if submitting {
                    ui.small(l.work_cancel_honesty);
                }
                let process_dead = self
                    .last_problem
                    .as_ref()
                    .and_then(|p| p.detail.as_deref())
                    .is_some_and(crate::lexicon::process_death_confirmed);
                if crate::lexicon::cancel_affordance_visible(process_dead) {
                    ui.small(l.work_cancelled);
                }
                ui.small(l.settings_models_model_ref);
                if self.work_executor_mode == work::WorkExecutorUiMode::Specific
                    && ui
                        .text_edit_singleline(&mut self.work_required_ref)
                        .changed()
                {
                    self.refresh_work_readiness();
                }
                if self.work_executor_mode == work::WorkExecutorUiMode::Compare {
                    ui.horizontal(|ui| {
                        ui.label(l.work_compare_a);
                        if ui.text_edit_singleline(&mut self.work_compare_a).changed() {
                            self.refresh_work_readiness();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(l.work_compare_b);
                        if ui.text_edit_singleline(&mut self.work_compare_b).changed() {
                            self.refresh_work_readiness();
                        }
                    });
                }
            });
        if self.work_result.is_some()
            || self.work_result_b.is_some()
            || self.work_compare_b_error.is_some()
        {
            ui.separator();
            let ans = ui.strong(l.work_answer);
            if ans.hovered() {
                self.note_help_focus(HelpId::WorkResult);
            }
        }
        if let Some(view) = self.work_result.clone() {
            // P2: show A as soon as ComparePrimary arrives (B may still be running).
            if self.work_result_b.is_some()
                || self.work_compare_b_error.is_some()
                || (self.async_jobs.work_inflight()
                    && matches!(self.work_executor_mode, work::WorkExecutorUiMode::Compare))
            {
                ui.strong(l.work_compare_leg_a);
            }
            self.ui_work_result_panel(ui, &view, "a");
        }
        if let Some(view) = self.work_result_b.clone() {
            ui.separator();
            ui.strong(l.work_compare_leg_b);
            self.ui_work_result_panel(ui, &view, "b");
        } else if let Some(err) = &self.work_compare_b_error {
            ui.separator();
            ui.strong(l.work_compare_leg_b);
            ui.colored_label(
                egui::Color32::from_rgb(180, 80, 40),
                format!("{}: {err}", l.work_compare_b_failed),
            );
            ui.small(l.work_compare_no_substitute);
        }
    }

    /// One result panel (answer → status → verification → provenance → triple → details).
    fn ui_work_result_panel(
        &mut self,
        ui: &mut egui::Ui,
        view: &crate::work_view::WorkResultView,
        id_suffix: &str,
    ) {
        let l = self.labels();
        let answer = if view.answer.is_empty() {
            l.work_no_answer
        } else {
            view.answer.as_str()
        };
        let room = (ui.available_height() - 72.0).max(160.0);
        egui::ScrollArea::vertical()
            .id_source(format!("work-answer-{id_suffix}"))
            .max_height(room)
            .show(ui, |ui| {
                ui.label(egui::RichText::new(answer).size(16.0));
            });
        let none = l.work_triple_none;
        let req = view.model_triple.requested.as_deref().unwrap_or(none);
        let exec = match view.model_triple.executed.as_deref() {
            Some(crate::work_view::EXECUTED_MOCK_LABEL) => l.work_triple_executed_mock,
            Some(s) => s,
            None => none,
        };
        ui.horizontal_wrapped(|ui| {
            ui.strong(l.work_ran_model);
            if view.model_triple.executed.as_deref() == Some(crate::work_view::EXECUTED_MOCK_LABEL)
            {
                ui.colored_label(egui::Color32::from_rgb(180, 120, 40), exec);
            } else {
                ui.label(self.row_name_for_ref(exec));
            }
        });
        if view.model_triple.requested.is_some()
            && view.model_triple.executed.is_some()
            && view.model_triple.requested != view.model_triple.executed
            && view.model_triple.executed.as_deref() != Some(crate::work_view::EXECUTED_MOCK_LABEL)
        {
            ui.colored_label(egui::Color32::from_rgb(180, 120, 40), l.work_model_mismatch);
        }
        let check = view.verification_status.as_deref().unwrap_or_else(|| {
            if view.status.eq_ignore_ascii_case("completed") {
                l.work_status_completed
            } else if view.status.eq_ignore_ascii_case("executed") {
                l.work_status_executed
            } else if view.status.eq_ignore_ascii_case("needs_human_collapse") {
                l.work_status_needs_human
            } else {
                view.status.as_str()
            }
        });
        ui.horizontal(|ui| {
            ui.strong(l.work_verification);
            let color = if check.eq_ignore_ascii_case("VERIFIED") {
                egui::Color32::from_rgb(40, 140, 70)
            } else {
                egui::Color32::from_rgb(180, 120, 40)
            };
            ui.colored_label(color, check);
        });
        if ui.button(l.work_copy_answer).clicked() {
            ui.ctx().copy_text(answer.to_string());
        }
        let has_ids = view.problem_id.is_some()
            || view.verified_artifact_id.is_some()
            || view.execution_artifact_id.is_some()
            || view.field_artifact_id.is_some();
        if has_ids {
            egui::CollapsingHeader::new(l.work_ids)
                .id_source(format!("work-ids-{id_suffix}"))
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
            .id_source(format!("work-details-{id_suffix}"))
            .default_open(false)
            .show(ui, |ui| {
                ui.label(self.work_provenance_label(view.provenance));
                ui.small(l.work_provenance);
                ui.small(l.work_run_status);
                ui.small(l.work_triple_executed);
                if let Some(prompt) = &view.prompt_snapshot {
                    ui.strong(l.work_prompt_snapshot);
                    ui.label(prompt);
                }
                ui.small(format!("{} {req}", l.work_triple_requested));
                ui.small(format!(
                    "{} {}",
                    l.work_triple_applied,
                    view.model_triple.applied.as_deref().unwrap_or(none)
                ));
                ui.monospace(&view.details_json);
            });
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
        if self.quit_after_stop && self.async_jobs.work_inflight() {
            ui.colored_label(egui::Color32::from_rgb(180, 120, 40), l.quit_waiting_submit);
        }
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
        if view.model.ready {
            ui.label(l.sys_model_ready_line);
        } else {
            ui.colored_label(egui::Color32::from_rgb(180, 120, 40), l.sys_model_not_ready);
            if ui.button(l.work_mock_banner_cta).clicked() {
                self.set_tab(MainTab::Settings);
                self.note_help_focus(HelpId::ModelSelect);
            }
        }
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("sys-model-tech")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(l.sys_model_selected);
                    ui.monospace(view.model.selected.as_display());
                });
                ui.horizontal(|ui| {
                    ui.label(l.sys_model_used);
                    ui.monospace(view.model.used.as_display());
                });
                ui.horizontal(|ui| {
                    ui.label(l.sys_model_executor);
                    ui.monospace(&view.model.executor_kind);
                });
                ui.label(format!("ready_detail: {}", view.model.ready_detail));
                ui.small(l.sys_model_ready);
                ui.small(l.sys_model_executor_mock_hint);
                ui.small(l.sys_model_triple_hint);
                ui.label(l.heading);
                ui.label(l.subtitle);
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
        // Phase R `#288` / Phase X `#352`: invites stay on System; profile edit → Settings.
        ui.separator();
        self.ui_connect_primary(ui, ctx);
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("sys-connection-tech")
            .default_open(false)
            .show(ui, |ui| {
                ui.small(format!("{} {}", l.sys_observed, view.observed_at));
                ui.small(format!("{} {}", l.sys_loaded, view.loaded_at));
                ui.small(format!(
                    "{} {}",
                    l.sys_quality,
                    view.network_quality.as_str()
                ));
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
        egui::CollapsingHeader::new(l.sys_events)
            .id_source("sys-events")
            .default_open(false)
            .show(ui, |ui| {
                let mut any = false;
                if let Some(problem) = &self.last_problem {
                    any = true;
                    ui.colored_label(egui::Color32::from_rgb(200, 60, 60), &problem.message);
                    let action_view = crate::problem_action::ProblemActionView::from_problem(
                        problem,
                        self.ui_lang(),
                    );
                    if let Some(next) = action_view.next_step {
                        ui.label(next);
                    }
                    ui.small(action_view.code_wire);
                }
                if let Some(msg) = &self.discovery_msg {
                    any = true;
                    ui.label(msg);
                }
                if let Some(msg) = &self.invite_msg {
                    any = true;
                    ui.label(msg);
                }
                if !any {
                    ui.label(l.sys_events_empty);
                }
            });
    }

    /// Primary connect path on System: observe + invites (`#288` / `#352`).
    ///
    /// Network profile / peer listen edit lives in Settings → Connection.
    fn ui_connect_primary(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
        ui.small(l.sys_connection_observe_hint);
        let profile_label = match self.settings.network_profile {
            NetworkProfile::P0 => l.p0,
            NetworkProfile::P1 => l.p1,
            NetworkProfile::P2 => l.p2,
            NetworkProfile::P3 => l.p3_relay,
            NetworkProfile::P4 => l.p4_gossip,
            NetworkProfile::P5 => l.federation,
            NetworkProfile::P6 => l.discovery,
        };
        ui.horizontal(|ui| {
            ui.strong(l.network_profile);
            ui.label(profile_label);
        });
        if ui.button(l.open_settings_connection).clicked() {
            self.note_help_focus(HelpId::NetworkConnect);
            self.set_tab(MainTab::Settings);
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

    /// Advanced Connection ops under System Technical details (`#288` / `#351` / `#352`).
    ///
    /// P3|P4 profile edit moved to Settings → Connection; dial/discovery stay diagnostic.
    fn ui_network_advanced(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
        ui.heading(l.advanced);
        ui.label(l.advanced_hint);
        ui.small(l.p34_mutex_hint);
        ui.small(l.settings_connection_edit_hint);
        if ui.button(l.open_settings_connection).clicked() {
            self.set_tab(MainTab::Settings);
        }
        if self.settings.network_profile.is_relay_profile() {
            let ttl = self
                .settings
                .relay_ttl_days
                .unwrap_or(DEFAULT_RELAY_TTL_DAYS);
            ui.label(format!("relay status: enabled · TTL {ttl} days"));
        }
        if self.settings.network_profile.is_gossip_profile() {
            ui.label("gossip status: enabled (dht+apply-book+apply-trust)");
        }

        ui.separator();
        ui.heading(l.peer_dial);
        ui.label(l.peer_dial_hint);
        let dialing = self.async_jobs.dial_inflight();
        ui.horizontal(|ui| {
            ui.label(l.dial_peer);
            ui.add_enabled(
                !dialing,
                egui::TextEdit::singleline(&mut self.dial_peer_edit),
            );
        });
        ui.horizontal(|ui| {
            ui.label(l.dial_addr);
            ui.add_enabled(
                !dialing,
                egui::TextEdit::singleline(&mut self.dial_addr_edit),
            );
            if ui
                .add_enabled(!dialing, egui::Button::new(l.dial_run))
                .clicked()
            {
                self.request_opt_in_peer_dial(ui.ctx());
            }
        });
        if let Some(msg) = &self.dial_msg {
            ui.label(msg);
        }
        if let Some(ev) = &self.mesh_snapshot.last_confirmed_handshake {
            ui.small(format!("{}: {ev}", l.mesh_last_handshake));
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
        ui.small(l.addr_advertised);
        ui.small(l.addr_roles_hint);
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

    fn ui_settings_general(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let l = self.labels();
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
                self.note_settings_apply_error(format!("{e:#}"));
            } else {
                self.clear_settings_apply_error();
            }
        }
    }

    /// Show the saved value. Repeat Applied only when it differs.
    fn ui_saved_or_diff(&self, ui: &mut egui::Ui, label: &str, saved: &str, applied: &str) {
        let l = self.labels();
        ui.horizontal(|ui| {
            ui.label(label);
            if saved == applied {
                ui.monospace(saved);
            } else {
                ui.strong(l.settings_saved);
                ui.monospace(saved);
            }
        });
        if saved != applied {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.strong(l.settings_applied);
                ui.monospace(applied);
            });
        }
    }

    fn ui_settings_timeout(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
        ui.horizontal(|ui| {
            ui.label(l.settings_ollama_timeout);
            ui.add(
                egui::TextEdit::singleline(&mut self.llm_timeout_edit)
                    .desired_width(100.0)
                    .hint_text("120"),
            );
            if ui.button(l.settings_ollama_timeout_apply).clicked() {
                match aira_desktop_runtime::timeout_ms_from_seconds_text(&self.llm_timeout_edit) {
                    Ok(ms) => {
                        self.settings.llm_process_timeout_ms = ms;
                        match self.persist_settings() {
                            Ok(()) => {
                                self.ollama_msg = Some(
                                    match ms {
                                        Some(_) => l.timeout_saved,
                                        None => l.timeout_cleared,
                                    }
                                    .into(),
                                );
                            }
                            Err(e) => self.set_problem(ErrorCode::SettingsPersistFailed, e),
                        }
                    }
                    Err(_) => self.ollama_msg = Some(l.timeout_invalid.into()),
                }
            }
        });
        ui.small(l.settings_ollama_timeout_hint);
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
            crate::settings_apply::SettingsApplyPhase::Changed => {
                ui.colored_label(
                    egui::Color32::from_rgb(40, 100, 180),
                    l.settings_phase_changed,
                );
                ui.small(l.settings_phase_changed_hint);
                ui.horizontal(|ui| {
                    if ui.button(l.settings_draft_save).clicked() {
                        if self.settings.network_profile.is_relay_profile() {
                            self.save_relay_ttl();
                        } else if self.settings.network_profile.requires_peer_listen() {
                            self.save_peer_listen();
                        }
                    }
                    if ui.button(l.settings_draft_cancel).clicked() {
                        self.cancel_connection_draft();
                    }
                });
            }
        }
        if let Some(err) = &self.settings_apply_error {
            ui.colored_label(
                egui::Color32::from_rgb(180, 60, 40),
                format!("{}: {err}", l.settings_apply_error),
            );
            ui.small(l.settings_apply_error_hint);
        }
        ui.label(l.settings_close_not_stop);

        ui.separator();
        let g = ui.strong(l.settings_group_models);
        if g.hovered() {
            self.note_help_focus(HelpId::ModelSelect);
        }
        ui.small(l.settings_models_catalog_hint);
        ui.horizontal(|ui| {
            ui.label(l.settings_models_source);
            if ui
                .selectable_label(
                    self.models_source == crate::app::ModelsSourceKind::HostOllama,
                    l.settings_models_source_ollama,
                )
                .clicked()
            {
                self.models_source = crate::app::ModelsSourceKind::HostOllama;
            }
            if ui
                .selectable_label(
                    self.models_source == crate::app::ModelsSourceKind::LocalFile,
                    l.settings_models_source_file,
                )
                .clicked()
            {
                self.models_source = crate::app::ModelsSourceKind::LocalFile;
            }
        });

        match self.models_source {
            crate::app::ModelsSourceKind::HostOllama => self.ui_settings_models_ollama(ui),
            crate::app::ModelsSourceKind::LocalFile => self.ui_settings_models_file(ui),
        }

        ui.separator();
        self.ui_settings_general(ui, ctx);

        ui.separator();
        let g = ui.strong(l.settings_group_connection);
        if g.hovered() {
            self.note_help_focus(HelpId::NetworkConnect);
        }
        // Phase X `#352`: network edit lives in Settings; System Connection observes.
        ui.small(l.settings_connection_edit_hint);
        ui.small(l.addr_roles_hint);
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
            ui.small(l.peer_listen_loopback_hint);
        }
        let profile_label = match self.settings.network_profile {
            NetworkProfile::P0 => l.p0,
            NetworkProfile::P1 => l.p1,
            NetworkProfile::P2 => l.p2,
            NetworkProfile::P3 => l.p3_relay,
            NetworkProfile::P4 => l.p4_gossip,
            NetworkProfile::P5 => l.federation,
            NetworkProfile::P6 => l.discovery,
        };
        let applied_profile = self
            .applied_runtime
            .as_ref()
            .map(|a| match a.network_profile {
                NetworkProfile::P0 => l.p0,
                NetworkProfile::P1 => l.p1,
                NetworkProfile::P2 => l.p2,
                NetworkProfile::P3 => l.p3_relay,
                NetworkProfile::P4 => l.p4_gossip,
                NetworkProfile::P5 => l.federation,
                NetworkProfile::P6 => l.discovery,
            })
            .unwrap_or(l.settings_applied_undefined);
        self.ui_saved_or_diff(ui, l.network_profile, profile_label, applied_profile);
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
        self.ui_saved_or_diff(ui, l.addr_peer_listen, saved_listen, applied_listen);

        ui.separator();
        let g = ui.strong(l.settings_group_advanced);
        if g.hovered() {
            self.note_help_focus(HelpId::SettingsApply);
        }
        ui.horizontal(|ui| {
            ui.label(l.addr_http);
            ui.monospace(&self.settings.http_listen);
        });
        if self
            .applied_runtime
            .as_ref()
            .is_some_and(|a| a.http_listen != self.settings.http_listen)
        {
            ui.horizontal(|ui| {
                ui.label(l.addr_http);
                ui.strong(l.settings_applied);
                if let Some(applied) = self.applied_runtime.as_ref() {
                    ui.monospace(&applied.http_listen);
                }
            });
        }
        self.ui_settings_timeout(ui);
        ui.small(l.p34_mutex_hint);
        let relay_on = self.settings.network_profile.is_relay_profile();
        let gossip_on = self.settings.network_profile.is_gossip_profile();
        let base_on = !relay_on && !gossip_on;
        ui.horizontal(|ui| {
            if ui.selectable_label(base_on, l.p34_base).clicked() && !base_on {
                self.apply_profile(NetworkProfile::P2);
            }
            if ui.selectable_label(relay_on, l.p3_relay).clicked() && !relay_on {
                self.apply_profile(NetworkProfile::P3);
            }
            if ui.selectable_label(gossip_on, l.p4_gossip).clicked() && !gossip_on {
                self.apply_profile(NetworkProfile::P4);
            }
        });
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
        }
        ui.label(format!("instance: {}", self.settings.instance_id));
    }

    fn ui_settings_models_ollama(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
        ui.strong(l.settings_ollama_heading);
        ui.small(l.settings_ollama_hint);
        ui.horizontal(|ui| {
            let process_on = matches!(
                self.settings.llm_backend,
                aira_desktop_runtime::LlmBackend::Process
            );
            if ui
                .selectable_label(process_on, l.settings_ollama_use_process)
                .clicked()
            {
                self.request_use_ollama_process(ui.ctx());
            }
            if ui
                .selectable_label(!process_on, l.settings_ollama_use_mock)
                .clicked()
            {
                self.bind_ollama_process(None);
            }
            if ui.button(l.settings_ollama_refresh).clicked() {
                self.refresh_ollama_list(ui.ctx());
            }
        });
        ui.horizontal(|ui| {
            ui.label(l.settings_ollama_bound_model);
            ui.monospace(self.settings.llm_ollama_model.as_deref().unwrap_or("—"));
        });
        if matches!(
            self.settings_apply_phase(),
            crate::settings_apply::SettingsApplyPhase::RestartNeeded
        ) && matches!(
            self.settings.llm_backend,
            aira_desktop_runtime::LlmBackend::Process
        ) {
            ui.small(l.settings_ollama_restart_hint);
        }
        let loading = self.async_jobs.catalog_kind()
            == Some(crate::async_jobs::CatalogJobKind::OllamaList)
            || self.ollama_bind_pending;
        if loading && self.ollama_models.is_empty() {
            ui.small(l.settings_ollama_loading);
        } else if self.ollama_models.is_empty() {
            ui.small(l.settings_ollama_empty);
        } else {
            let rows: Vec<_> = self
                .catalog_rows()
                .iter()
                .filter(|r| r.source == CatalogSource::HostOllama)
                .cloned()
                .collect();
            egui::ScrollArea::vertical()
                .max_height(140.0)
                .id_source("settings-ollama-list")
                .show(ui, |ui| {
                    for row in rows {
                        let selected = self.ollama_pick.as_deref() == Some(row.name.as_str())
                            || (self.ollama_pick.is_none()
                                && self.settings.llm_ollama_model.as_deref()
                                    == Some(row.name.as_str()));
                        if ui
                            .selectable_label(selected, Self::projection_label(l, &row))
                            .clicked()
                        {
                            // Request pick only — Make default writes the tip.
                            self.ollama_pick = Some(row.name.clone());
                        }
                    }
                });
        }
        if let Some(msg) = &self.ollama_msg {
            ui.small(msg);
        }

        if ui.button(l.settings_models_make_default).clicked() {
            let cli = aira_desktop_runtime::exact_cli_name_for_bind(
                self.ollama_pick.as_deref(),
                &self.ollama_models,
                &self.catalog_projection,
            );
            match cli {
                Some(m) => self.bind_ollama_process(Some(m)),
                None if self.ollama_pick.is_some() => {
                    self.ollama_msg = Some(l.settings_ollama_need_exact_cli.into());
                }
                None => {
                    self.ollama_msg = Some(l.settings_ollama_select_first.into());
                }
            }
        }
    }

    fn ui_settings_models_file(&mut self, ui: &mut egui::Ui) {
        let l = self.labels();
        ui.strong(l.settings_models_storage);
        ui.small(l.settings_models_storage_hint);
        ui.horizontal(|ui| {
            ui.label(l.settings_models_storage_root);
            ui.monospace(self.model_storage.models_root.display().to_string());
        });
        ui.horizontal(|ui| {
            ui.label(l.settings_models_storage_used);
            ui.label(aira_desktop_runtime::format_bytes(
                self.model_storage.used_bytes,
            ));
        });
        ui.horizontal(|ui| {
            ui.label(l.settings_models_storage_free);
            match self.model_storage.available_bytes {
                Some(b) => ui.label(aira_desktop_runtime::format_bytes(b)),
                None => ui.label(l.settings_models_storage_unknown),
            };
        });
        egui::CollapsingHeader::new(l.settings_models_storage_subdirs)
            .id_source("settings-models-storage-subdirs")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("quarantine");
                    ui.monospace(self.model_storage.quarantine_dir.display().to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("verified");
                    ui.monospace(self.model_storage.verified_dir.display().to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("cache");
                    ui.monospace(self.model_storage.cache_dir.display().to_string());
                });
            });
        ui.horizontal(|ui| {
            let catalog_busy = self.async_jobs.catalog_inflight();
            let work_busy = self.async_jobs.work_inflight();
            if ui
                .add_enabled(!catalog_busy, egui::Button::new(l.settings_models_scan))
                .clicked()
            {
                let ctx = ui.ctx().clone();
                if !self.async_jobs.try_spawn_catalog(
                    crate::async_jobs::CatalogJobKind::Scan,
                    self.paths.clone(),
                    work_busy,
                    None,
                    None,
                    None,
                    None,
                    move || ctx.request_repaint(),
                ) {
                    self.catalog_msg = Some(l.catalog_job_busy.into());
                }
            }
            if ui
                .selectable_label(self.catalog_auto, l.settings_models_auto)
                .clicked()
            {
                self.catalog_auto = true;
            }
            if ui
                .add_enabled(!catalog_busy, egui::Button::new(l.settings_models_select))
                .clicked()
            {
                let selection = if self.catalog_auto {
                    Some(CatalogSelection::Auto)
                } else if let Some(r) = self.catalog_highlight.clone() {
                    if aira_desktop_runtime::is_host_ollama_catalog_ref(&r) {
                        self.catalog_msg = Some(l.catalog_ollama_not_file.into());
                        None
                    } else {
                        Some(CatalogSelection::Required(r))
                    }
                } else {
                    self.catalog_msg = Some(l.catalog_select_row.into());
                    None
                };
                if let Some(selection) = selection {
                    if work_busy {
                        self.catalog_msg = Some(l.catalog_work_locked.into());
                    } else {
                        let ctx = ui.ctx().clone();
                        if !self.async_jobs.try_spawn_catalog(
                            crate::async_jobs::CatalogJobKind::Select,
                            self.paths.clone(),
                            work_busy,
                            None,
                            Some(selection),
                            None,
                            None,
                            move || ctx.request_repaint(),
                        ) {
                            self.catalog_msg = Some(l.catalog_job_busy.into());
                        }
                    }
                }
            }
            let file_prepare = self
                .catalog_highlight
                .as_deref()
                .is_none_or(|r| !aira_desktop_runtime::is_host_ollama_catalog_ref(r));
            if file_prepare
                && ui
                    .add_enabled(!catalog_busy, egui::Button::new(l.settings_models_prepare))
                    .clicked()
            {
                if work_busy {
                    self.catalog_msg = Some(l.catalog_work_locked.into());
                } else if let Some(r) = self.catalog_highlight.clone() {
                    if aira_desktop_runtime::is_host_ollama_catalog_ref(&r) {
                        self.catalog_msg = Some(l.catalog_not_prepare.into());
                    } else {
                        let ctx = ui.ctx().clone();
                        if !self.async_jobs.try_spawn_catalog(
                            crate::async_jobs::CatalogJobKind::Prepare,
                            self.paths.clone(),
                            work_busy,
                            Some(r),
                            None,
                            None,
                            None,
                            move || ctx.request_repaint(),
                        ) {
                            self.catalog_msg = Some(l.catalog_job_busy.into());
                        }
                    }
                } else {
                    self.catalog_msg = Some(l.catalog_select_before_prepare.into());
                }
            }
            if ui
                .add_enabled(!catalog_busy, egui::Button::new(l.settings_models_verify))
                .clicked()
            {
                let art = self.catalog_artifact_edit.trim().to_string();
                if art.is_empty() {
                    self.catalog_msg = Some(l.catalog_need_artifact.into());
                } else if work_busy {
                    self.catalog_msg = Some(l.catalog_work_locked.into());
                } else {
                    let ctx = ui.ctx().clone();
                    if !self.async_jobs.try_spawn_catalog(
                        crate::async_jobs::CatalogJobKind::Verify,
                        self.paths.clone(),
                        work_busy,
                        None,
                        None,
                        Some(std::path::PathBuf::from(art)),
                        None,
                        move || ctx.request_repaint(),
                    ) {
                        self.catalog_msg = Some(l.catalog_job_busy.into());
                    }
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(l.settings_models_model_ref);
            ui.text_edit_singleline(&mut self.catalog_add_ref);
            if !self.model_catalog.local_add_allowed
                && ui.button(l.settings_models_enable_add).clicked()
            {
                match actions::models_catalog_enable_local_add(&self.paths) {
                    Ok(snap) => self.apply_catalog_snapshot(snap),
                    Err(e) => self.catalog_msg = Some(format!("{e:#}")),
                }
            }
            let catalog_busy = self.async_jobs.catalog_inflight();
            let work_busy = self.async_jobs.work_inflight();
            if ui
                .add_enabled(
                    !catalog_busy && self.model_catalog.local_add_allowed,
                    egui::Button::new(l.settings_models_add),
                )
                .clicked()
            {
                if work_busy {
                    self.catalog_msg = Some(l.catalog_work_locked.into());
                } else {
                    let path = rfd::FileDialog::new()
                        .add_filter("weights", &["bin", "gguf", "ggml", "safetensors"])
                        .pick_file();
                    if let Some(path) = path {
                        let ctx = ui.ctx().clone();
                        if !self.async_jobs.try_spawn_catalog(
                            crate::async_jobs::CatalogJobKind::Add,
                            self.paths.clone(),
                            work_busy,
                            Some(self.catalog_add_ref.clone()),
                            None,
                            Some(path),
                            None,
                            move || ctx.request_repaint(),
                        ) {
                            self.catalog_msg = Some(l.catalog_job_busy.into());
                        }
                    }
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(l.settings_models_artifact);
            ui.text_edit_singleline(&mut self.catalog_artifact_edit);
        });

        let file_rows: Vec<_> = self
            .catalog_rows()
            .iter()
            .filter(|r| r.source == CatalogSource::LocalFile)
            .cloned()
            .collect();
        if file_rows.is_empty() {
            ui.small(l.settings_models_empty);
        } else {
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .show(ui, |ui| {
                    for row in file_rows {
                        let selected =
                            self.catalog_highlight.as_deref() == Some(row.model_ref.as_str());
                        let tip = self.model_catalog.tip_model_ref.as_deref()
                            == Some(row.model_ref.as_str());
                        let mut label = Self::projection_label(l, &row);
                        if tip {
                            label.push_str(" (tip)");
                        }
                        if ui.selectable_label(selected, label).clicked() {
                            self.catalog_highlight = Some(row.model_ref.clone());
                            self.catalog_auto = false;
                            self.catalog_add_ref = row.model_ref.clone();
                        }
                    }
                });
        }
        if let Some(msg) = &self.catalog_msg {
            ui.small(msg);
        }
        egui::CollapsingHeader::new(l.sys_tech_details)
            .id_source("settings-models-tech")
            .default_open(false)
            .show(ui, |ui| {
                ui.label(l.not_llm);
                ui.label(format!("ready_detail: {}", self.model_triple.ready_detail));
                if let Some(tip) = &self.model_catalog.tip_model_ref {
                    ui.label(format!("default: {tip}"));
                }
                if let Some(h) = &self.catalog_highlight {
                    ui.monospace(h);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::i18n::Labels;
    use aira_desktop_runtime::UiLang;

    #[test]
    fn empty_compare_b_is_not_labeled_as_the_tip() {
        for lang in [UiLang::En, UiLang::Uk] {
            let l = Labels::get(lang);
            let empty = work_selector_closed_label(
                WorkPick::CompareB,
                None,
                l.work_selector_default,
                "kimi-k2.7-code:cloud",
                l.work_compare_b_empty,
            );
            let tip = format!("{}: kimi-k2.7-code:cloud", l.work_selector_default);
            assert_ne!(empty, tip);
            assert!(!empty.contains(l.work_selector_default));
            assert_eq!(empty, l.work_compare_b_empty);

            let still_default = work_selector_closed_label(
                WorkPick::Required,
                None,
                l.work_selector_default,
                "kimi-k2.7-code:cloud",
                l.work_compare_b_empty,
            );
            assert_eq!(still_default, tip);
        }
    }

    #[test]
    fn prepare_and_run_label_is_not_the_run_button() {
        let uk = Labels::get(UiLang::Uk);
        let en = Labels::get(UiLang::En);
        assert_eq!(uk.work_prepare_and_run, "Підготувати й виконати");
        assert_eq!(en.work_prepare_and_run, "Prepare and run");
        assert_ne!(uk.work_prepare_and_run, uk.work_submit);
        assert_ne!(en.work_prepare_and_run, en.work_submit);
        assert!(!uk.work_prepare_and_run.contains("Типова"));
    }

    #[test]
    fn compare_keeps_b_reason_when_a_is_ready() {
        let reasons = vec![
            "Compare A aira:model:ollama-a available; choice ≠ VERIFIED; RequireNewExecution"
                .into(),
            "Compare B aira:model:ollama-b not ready — no silent substitute".into(),
            "aira:model:ollama-b not in lifecycle catalog — Scan/Prepare in Settings → Models"
                .into(),
        ];
        let shown = work_readiness_lines(true, &reasons);
        assert!(shown
            .iter()
            .any(|s| s.contains("Compare A") && s.contains("available")));
        assert!(shown
            .iter()
            .any(|s| s.contains("Compare B") && s.contains("not ready")));
        assert!(shown.iter().any(|s| s.contains("not in lifecycle catalog")));
        let single = work_readiness_lines(false, &reasons);
        assert_eq!(single, vec![reasons[0].as_str()]);
    }
}
