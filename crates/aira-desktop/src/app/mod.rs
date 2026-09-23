//! egui application state for AIRA Desktop (QUEUE #78 / #85 / #130 split).

mod discovery;
mod federation;
mod i18n;
mod invite;
mod labels;
mod peer_dial;
mod profile;
mod ui;
mod work;

use std::path::PathBuf;

use aira_desktop_runtime::{
    apply_ollama_list_refresh, evaluate_work_readiness, load_or_create_settings,
    load_or_create_ui_prefs, load_system_snapshot, project_shared_catalog,
    sync_autostart_from_settings, write_ui_prefs, CatalogProjectionRow, DesktopPaths,
    DesktopSettings, LifecycleStatus, LlmBackend, ModelCatalogSnapshot, ModelFact,
    ModelStorageSnapshot, ModelTripleSnapshot, NetworkMeshSnapshot, OllamaHostListFreshness,
    OllamaHostListSnapshot, SystemSnapshot, UiLang, UiPrefs, WorkExecutorPreference, WorkReadiness,
    DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;
use crate::async_jobs::{
    quit_arm_policy, quit_followup_after_lifecycle, quit_followup_after_submit, AsyncDesktopJobs,
    CatalogJobKind, CatalogJobResult, LifecycleJobKind, LifecycleJobResult, QuitArm, QuitFollowup,
    StatusSnapshot, WorkJobEvent,
};
use crate::camera;
use crate::lexicon::{ActionId, ErrorCode, HelpId, UiProblem};

use self::i18n::Labels;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MainTab {
    Work,
    System,
    Settings,
}

impl MainTab {
    /// Default Help topic for this shell section (`#259`).
    pub(super) fn default_help(self) -> HelpId {
        match self {
            Self::Work => HelpId::WorkSubmit,
            Self::System => HelpId::NetworkReachability,
            Self::Settings => HelpId::SettingsApply,
        }
    }
}

/// Pure F1 topic resolution: focus → section → screen (`#273`).
///
/// Does **not** consult `last_problem` (explicit problem Help uses `open_help`).
pub(super) fn resolve_help_routing(
    focus: Option<HelpId>,
    tab: MainTab,
    has_work_result: bool,
    work_inflight: bool,
) -> HelpId {
    if let Some(id) = focus {
        return id;
    }
    match tab {
        MainTab::Work => {
            if has_work_result {
                HelpId::WorkResult
            } else if work_inflight {
                HelpId::WorkWaiting
            } else {
                HelpId::WorkSubmit
            }
        }
        MainTab::System | MainTab::Settings => tab.default_help(),
    }
}

/// Settings → Models source surface (P3 catalog IA).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelsSourceKind {
    /// Host `ollama list` / process executor bind.
    HostOllama,
    /// Local weight files (Scan / Add / Verify / Prepare).
    LocalFile,
}

pub struct AiraDesktopApp {
    pub(super) paths: DesktopPaths,
    pub(super) node_bin: Option<PathBuf>,
    pub(super) settings: DesktopSettings,
    pub(super) ui_prefs: UiPrefs,
    pub(super) tab: MainTab,
    pub(super) node_running: bool,
    pub(super) lifecycle: LifecycleStatus,
    pub(super) problem_text: String,
    pub(super) work_result: Option<crate::work_view::WorkResultView>,
    /// Compare leg B (`#354`); cleared on single submit.
    pub(super) work_result_b: Option<crate::work_view::WorkResultView>,
    /// Explicit Compare B failure (A kept); never silent substitute.
    pub(super) work_compare_b_error: Option<String>,
    pub(super) status_label: String,
    pub(super) detail: String,
    pub(super) peer_detail: String,
    pub(super) mesh_snapshot: NetworkMeshSnapshot,
    pub(super) system_snapshot: SystemSnapshot,
    /// Model selected ≠ ready ≠ used (`#269`).
    pub(super) model_triple: ModelTripleSnapshot,
    /// Settings → Models catalog (`#348` / RFC-0231).
    pub(super) model_catalog: ModelCatalogSnapshot,
    /// Paint-safe shared catalog rows (`#380`). Rebuilt when tip/list/catalog changes —
    /// never via tip/cache I/O inside `update`.
    pub(super) catalog_projection: Vec<CatalogProjectionRow>,
    /// Settings → Models storage paths + space (`#355` / RFC-0238).
    pub(super) model_storage: ModelStorageSnapshot,
    pub(super) catalog_highlight: Option<String>,
    pub(super) catalog_auto: bool,
    pub(super) catalog_add_ref: String,
    pub(super) catalog_msg: Option<String>,
    /// Path to ModelArtifact JSON for Verify (Pack C).
    pub(super) catalog_artifact_edit: String,
    /// Host `ollama list` names for Settings process bind (not AIRA catalog).
    pub(super) ollama_models: Vec<String>,
    /// `#365`: Absent vs Present vs Unknown (error keeps prior names).
    pub(super) ollama_list_freshness: OllamaHostListFreshness,
    /// Host name chosen for a request. Does not write the default tip.
    pub(super) ollama_pick: Option<String>,
    /// First “Use Ollama” while the list is still empty: bind after the list returns.
    pub(super) ollama_bind_pending: bool,
    pub(super) ollama_msg: Option<String>,
    /// Draft for `llm_process_timeout_ms`, shown as whole seconds (empty = unset).
    pub(super) llm_timeout_edit: String,
    /// Settings → Models source surface (P3 IA).
    pub(super) models_source: ModelsSourceKind,
    /// Work executor Auto / Specific / Compare + readiness (`#349` / `#354`).
    pub(super) work_executor_mode: work::WorkExecutorUiMode,
    pub(super) work_required_ref: String,
    pub(super) work_compare_a: String,
    pub(super) work_compare_b: String,
    pub(super) work_readiness: WorkReadiness,
    pub(super) peer_listen_edit: String,
    pub(super) relay_ttl_edit: String,
    pub(super) invite_msg: Option<String>,
    pub(super) federation_detail: String,
    pub(super) discovery_msg: Option<String>,
    pub(super) stun_server_edit: String,
    pub(super) discv_to_edit: String,
    pub(super) discv_addr_edit: String,
    pub(super) find_key_edit: String,
    pub(super) find_to_edit: String,
    /// Phase S `#300`: opt-in dial peer identity + explicit address (Technical details).
    pub(super) dial_peer_edit: String,
    pub(super) dial_addr_edit: String,
    pub(super) dial_msg: Option<String>,
    pub(super) last_problem: Option<UiProblem>,
    pub(super) qr_texture: Option<egui::TextureHandle>,
    pub(super) qr_camera: Option<camera::InviteQrCamera>,
    pub(super) qr_camera_status: Option<String>,
    pub(super) restart_hint: bool,
    /// Last Settings apply/persist error detail (`#356`); cleared on success.
    pub(super) settings_apply_error: Option<String>,
    /// Confirmed runtime-applied network/listen subset (`#262`/`#268`).
    /// `None` = Explicitly Undefined (no Running confirmation yet).
    pub(super) applied_runtime: Option<crate::settings_apply::AppliedRuntimeSettings>,
    pub(super) async_jobs: AsyncDesktopJobs,
    /// Close viewport after an in-flight Stop completes (`#272`).
    pub(super) quit_after_stop: bool,
    /// Side help panel open (`#259` chrome; offline articles `#263`).
    pub(super) help_open: bool,
    /// Active help topic key for the panel.
    pub(super) help_topic: HelpId,
    /// Offline Help search query (`#263`).
    pub(super) help_search: String,
    /// Last UI element / section that claimed F1 context (`#273`).
    pub(super) help_focus: Option<HelpId>,
}

impl AiraDesktopApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        paths: DesktopPaths,
        node_bin: Option<PathBuf>,
        auto_start: bool,
    ) -> Self {
        i18n::install_cyrillic_font(&cc.egui_ctx);
        let mut last_problem = None;
        let settings = match load_or_create_settings(&paths) {
            Ok(s) => s,
            Err(e) => {
                last_problem = Some(UiProblem::from_code_err(
                    ErrorCode::Generic,
                    UiLang::En,
                    format!("{e:#}"),
                ));
                DesktopSettings::default_p0(&paths)
            }
        };
        let ui_prefs = match load_or_create_ui_prefs(&paths) {
            Ok(p) => p,
            Err(e) => {
                last_problem = Some(UiProblem::from_code_err(
                    ErrorCode::Generic,
                    UiLang::En,
                    format!("ui prefs: {e:#}"),
                ));
                UiPrefs::new(UiLang::En)
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
        let llm_timeout_edit =
            aira_desktop_runtime::timeout_seconds_text(settings.llm_process_timeout_ms);
        let models_source = ModelsSourceKind::HostOllama;
        let applied_runtime = None;
        let work_readiness = evaluate_work_readiness(
            &paths.data_root,
            &settings,
            "",
            WorkExecutorPreference::Auto,
        );
        let mut app = Self {
            paths,
            node_bin,
            settings,
            ui_prefs,
            tab: MainTab::Work,
            node_running: false,
            lifecycle: LifecycleStatus::Stopped,
            problem_text: String::new(),
            work_result: None,
            work_result_b: None,
            work_compare_b_error: None,
            status_label: Labels::get(UiLang::En).st_stopped.into(),
            detail: String::new(),
            peer_detail: String::new(),
            mesh_snapshot: NetworkMeshSnapshot::unavailable(),
            system_snapshot: SystemSnapshot::unavailable(),
            model_triple: ModelTripleSnapshot::undefined(),
            model_catalog: ModelCatalogSnapshot::default(),
            catalog_projection: Vec::new(),
            model_storage: ModelStorageSnapshot::default(),
            catalog_highlight: None,
            catalog_auto: true,
            catalog_add_ref: String::new(),
            catalog_msg: None,
            catalog_artifact_edit: String::new(),
            ollama_models: Vec::new(),
            ollama_list_freshness: OllamaHostListFreshness::Absent,
            ollama_pick: None,
            ollama_bind_pending: false,
            ollama_msg: None,
            llm_timeout_edit,
            models_source,
            work_executor_mode: work::WorkExecutorUiMode::Auto,
            work_required_ref: String::new(),
            work_compare_a: String::new(),
            work_compare_b: String::new(),
            work_readiness,
            peer_listen_edit,
            relay_ttl_edit,
            invite_msg: None,
            federation_detail: String::new(),
            discovery_msg: None,
            stun_server_edit: String::new(),
            discv_to_edit: String::new(),
            discv_addr_edit: String::new(),
            find_key_edit: String::new(),
            find_to_edit: String::new(),
            dial_peer_edit: String::new(),
            dial_addr_edit: String::new(),
            dial_msg: None,
            last_problem,
            qr_texture: None,
            qr_camera: None,
            qr_camera_status: None,
            restart_hint: false,
            settings_apply_error: None,
            applied_runtime,
            async_jobs: AsyncDesktopJobs::new(),
            quit_after_stop: false,
            help_open: false,
            help_topic: HelpId::Start,
            help_search: String::new(),
            help_focus: None,
        };
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            Labels::get(app.ui_lang()).window_title.to_string(),
        ));
        let _ = app.refresh_status();
        app.refresh_model_catalog();
        app.refresh_work_readiness();
        app.refresh_federation_detail();
        if auto_start {
            app.request_lifecycle(LifecycleJobKind::Start, &cc.egui_ctx);
        }
        if let Err(e) = sync_autostart_from_settings(app.settings.autostart_on_login) {
            app.set_problem(ErrorCode::AutostartSyncFailed, format!("{e:#}"));
        }
        app
    }

    pub(super) fn ui_lang(&self) -> UiLang {
        self.ui_prefs.ui_lang
    }

    pub(super) fn labels(&self) -> &'static Labels {
        Labels::get(self.ui_lang())
    }

    pub(super) fn set_ui_lang(&mut self, lang: UiLang, ctx: &egui::Context) {
        if self.ui_prefs.ui_lang == lang {
            return;
        }
        self.ui_prefs.ui_lang = lang;
        if let Err(e) = write_ui_prefs(&self.paths, &self.ui_prefs) {
            self.set_problem(ErrorCode::SettingsPersistFailed, format!("{e:#}"));
            return;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            Labels::get(lang).window_title.to_string(),
        ));
        let _ = self.refresh_status();
        self.refresh_federation_detail();
        if self.qr_camera.is_some() {
            self.qr_camera_status = Some(Labels::get(lang).scan_camera.to_string());
        }
    }

    pub(super) fn refresh_status(&mut self) -> anyhow::Result<()> {
        let snap = crate::async_jobs::collect_status_snapshot(&self.paths, &self.settings)?;
        self.apply_status_snapshot(snap);
        Ok(())
    }

    /// Apply a status snapshot collected off the UI thread (`#257` / `#282`).
    ///
    /// While a Start/Stop is in flight, do not overwrite optimistic lifecycle or Applied.
    pub(super) fn apply_status_snapshot(&mut self, snap: StatusSnapshot) {
        if self.async_jobs.lifecycle_inflight() {
            self.mesh_snapshot = snap.mesh;
            self.system_snapshot = snap.system;
            self.model_triple = snap.model.with_used(self.used_model_fact());
            return;
        }
        self.lifecycle = snap.lifecycle;
        self.node_running = matches!(snap.lifecycle, LifecycleStatus::Running);
        self.status_label = labels::status_label(snap.lifecycle, self.ui_lang()).to_string();
        let l = self.labels();
        // `#268`: sync Applied from confirmed Running status (never from settings alone).
        self.applied_runtime = crate::settings_apply::AppliedRuntimeSettings::from_status(
            snap.lifecycle,
            snap.record.as_ref(),
        );
        if self.applied_runtime.is_some() {
            self.restart_hint = false;
        }
        match snap.record {
            Some(r) => {
                self.detail = format!("pid {} · {} · {}", r.pid, r.listen, r.instance_id);
                self.peer_detail = match (r.peer_pid, r.peer_listen.as_ref()) {
                    (Some(pp), Some(pl)) => labels::format_peer_running(
                        self.settings.network_profile,
                        pp,
                        pl,
                        self.settings.relay_ttl_days,
                    ),
                    _ if self.settings.network_profile.requires_peer_listen() => {
                        labels::format_peer_not_running(self.settings.network_profile)
                    }
                    _ => l.peer_off_p0.into(),
                };
            }
            None => {
                self.detail = format!("listen {}", self.settings.http_listen);
                self.peer_detail = if self.settings.network_profile.requires_peer_listen() {
                    labels::format_peer_configured(
                        self.settings.network_profile,
                        self.settings.peer_listen.as_deref(),
                        self.settings.relay_ttl_days,
                    )
                } else {
                    l.peer_off_p0.into()
                };
            }
        }
        self.mesh_snapshot = snap.mesh;
        self.system_snapshot = snap.system;
        self.model_triple = snap.model.with_used(self.used_model_fact());
    }

    /// Used-in-result fact from the last Work payload only (`#269` / `#283`).
    ///
    /// `None` used_model → [`ModelFact::None`] (no model evidence); never invent `backend:*`.
    pub(super) fn used_model_fact(&self) -> ModelFact {
        match &self.work_result {
            Some(w) => match &w.used_model {
                Some(m) if !m.starts_with("backend:") => ModelFact::Value(m.clone()),
                Some(_) => ModelFact::None,
                None => ModelFact::None,
            },
            None => ModelFact::Undefined,
        }
    }

    /// Reload selected/ready from disk; preserve used from last Work result.
    pub(super) fn refresh_model_triple(&mut self) {
        let executor = self
            .applied_runtime
            .as_ref()
            .map(|a| a.llm_backend.as_env_str())
            .unwrap_or_else(|| self.settings.llm_backend.as_env_str());
        self.model_triple = ModelTripleSnapshot::load(&self.paths.data_root, executor)
            .with_used(self.used_model_fact());
    }

    /// Reload Settings → Models catalog (`#348`) and storage snapshot (`#355`).
    pub(super) fn refresh_model_catalog(&mut self) {
        self.model_storage = actions::models_storage_load(&self.paths);
        match actions::models_catalog_load(&self.paths) {
            Ok(snap) => {
                if self.catalog_highlight.is_none() {
                    self.catalog_highlight = snap.tip_model_ref.clone();
                }
                self.model_catalog = snap;
                self.rebuild_catalog_projection();
                self.refresh_work_readiness();
            }
            Err(e) => {
                self.catalog_msg = Some(format!("{e:#}"));
            }
        }
    }

    /// Rebuild paint-safe projection from in-memory catalog + list (`#380`).
    ///
    /// Disk/evidence work happens here (and in async catalog jobs), not per frame.
    pub(super) fn rebuild_catalog_projection(&mut self) {
        self.catalog_projection = project_shared_catalog(
            &self.model_catalog,
            &self.ollama_models,
            &self.paths.data_root,
        );
    }

    pub(super) fn apply_catalog_snapshot(&mut self, snap: ModelCatalogSnapshot) {
        self.catalog_msg = snap.last_message.clone();
        if self.catalog_highlight.is_none() {
            self.catalog_highlight = snap.tip_model_ref.clone();
        }
        self.model_catalog = snap;
        self.model_storage = actions::models_storage_load(&self.paths);
        self.rebuild_catalog_projection();
        self.refresh_model_triple();
        self.refresh_work_readiness();
    }

    /// Probe host `ollama list` into Settings UI (observe-only; not VERIFIED).
    ///
    /// Pack D: runs off the egui thread with a bounded CLI timeout.
    pub(super) fn refresh_ollama_list(&mut self, ctx: &egui::Context) {
        let ctx = ctx.clone();
        let on_done = move || ctx.request_repaint();
        if !self.async_jobs.try_spawn_catalog(
            CatalogJobKind::OllamaList,
            self.paths.clone(),
            self.async_jobs.work_inflight(),
            None,
            None,
            None,
            self.settings.llm_process_bin.clone(),
            self.settings.llm_ollama_host.clone(),
            on_done,
        ) {
            self.ollama_msg = Some(if self.async_jobs.catalog_inflight() {
                self.labels().catalog_job_busy.into()
            } else {
                self.labels().catalog_work_locked.into()
            });
        }
    }

    /// Persist Ollama process bind.
    ///
    /// Backend Mock↔Process needs a node restart. Switching the host model on an
    /// already applied Process does not: admission `host_cli_model` is per request.
    /// Choosing a row (`ollama_pick`) does not call this — Make default / Use Ollama does.
    pub(super) fn bind_ollama_process(&mut self, model: Option<String>) {
        match model {
            Some(m) => {
                match aira_flow::ActivatedPointerGate::install_host_ollama_bind(
                    &self.paths.data_root,
                    &m,
                ) {
                    Ok((_gate, model_ref)) => {
                        self.settings.llm_backend = LlmBackend::Process;
                        self.settings.llm_ollama_model = Some(m);
                        if self.settings.llm_process_bin.is_none() {
                            self.settings.llm_process_bin = Some("ollama".into());
                        }
                        if let Err(e) = self.persist_settings() {
                            self.note_settings_apply_error(format!("{e:#}"));
                        } else {
                            self.clear_settings_apply_error();
                            self.refresh_model_triple();
                            self.refresh_model_catalog();
                            self.ollama_msg = Some(format!(
                                "{} ({model_ref})",
                                self.labels().settings_ollama_bound
                            ));
                        }
                    }
                    Err(e) => {
                        self.note_settings_apply_error(format!(
                            "{}: {e}",
                            self.labels().settings_apply_error
                        ));
                        self.ollama_msg = Some(e.to_string());
                    }
                }
            }
            None => {
                self.settings.llm_backend = LlmBackend::Mock;
                // Keep last model name for re-bind convenience; tip left as-is.
                if let Err(e) = self.persist_settings() {
                    self.note_settings_apply_error(format!("{e:#}"));
                } else {
                    self.clear_settings_apply_error();
                    self.refresh_model_triple();
                    self.ollama_msg = Some(self.labels().settings_ollama_bound.into());
                }
            }
        }
    }

    /// Apply one host-list refresh outcome (`#365`): keep names on Err; Absent on empty Ok.
    pub(super) fn apply_ollama_list_outcome(&mut self, outcome: Result<Vec<String>, String>) {
        let previous = OllamaHostListSnapshot {
            names: self.ollama_models.clone(),
            freshness: self.ollama_list_freshness,
        };
        let next = apply_ollama_list_refresh(&previous, outcome);
        self.ollama_models = next.names;
        self.ollama_list_freshness = next.freshness;
        self.rebuild_catalog_projection();
    }

    /// Finish a pending Use Ollama bind once `ollama list` has returned.
    pub(super) fn finish_pending_ollama_bind(&mut self) {
        if !self.ollama_bind_pending {
            return;
        }
        self.ollama_bind_pending = false;
        match aira_desktop_runtime::resolve_bind_after_ollama_list(
            self.ollama_pick.as_deref(),
            &self.ollama_models,
        ) {
            Some(m) => self.bind_ollama_process(Some(m)),
            None => {
                // `#364`/`#365`: never invent first-row; Absent ≠ Unknown.
                self.ollama_msg = Some(match self.ollama_list_freshness {
                    OllamaHostListFreshness::Absent => self.labels().settings_ollama_empty.into(),
                    OllamaHostListFreshness::Unknown => {
                        self.labels().settings_ollama_unknown.into()
                    }
                    OllamaHostListFreshness::Present => {
                        self.labels().settings_ollama_need_exact_cli.into()
                    }
                });
            }
        }
    }

    /// Use Ollama process. If no name is known yet, wait for the host list.
    ///
    /// `#364`: a loaded list without an exact pick/bind does not take the first row.
    pub(super) fn request_use_ollama_process(&mut self, ctx: &egui::Context) {
        if let Some(m) = aira_desktop_runtime::resolve_use_ollama_bind(
            self.ollama_pick.as_deref(),
            self.settings.llm_ollama_model.as_deref(),
            &self.ollama_models,
        ) {
            self.ollama_bind_pending = false;
            self.bind_ollama_process(Some(m));
            return;
        }
        if !self.ollama_models.is_empty() {
            self.ollama_bind_pending = false;
            self.ollama_msg = Some(self.labels().settings_ollama_need_exact_cli.into());
            return;
        }
        self.ollama_bind_pending = true;
        let already = self.async_jobs.catalog_kind() == Some(CatalogJobKind::OllamaList);
        if !already {
            self.refresh_ollama_list(ctx);
        }
        if self.async_jobs.catalog_kind() == Some(CatalogJobKind::OllamaList) {
            self.ollama_msg = Some(self.labels().settings_ollama_loading.into());
        } else {
            self.ollama_bind_pending = false;
        }
    }

    /// Request a background status refresh (no-op if one is already running).
    pub(super) fn request_status_refresh(&mut self, ctx: &egui::Context) {
        let ctx = ctx.clone();
        let _ = self.async_jobs.try_spawn_refresh(
            self.paths.clone(),
            self.settings.clone(),
            move || ctx.request_repaint(),
        );
    }

    /// Poll background jobs and schedule periodic data refresh (`#257` / `#282`).
    pub(super) fn pump_async_jobs(&mut self, ctx: &egui::Context) {
        if let Some((kind, outcome)) = self.async_jobs.poll_lifecycle() {
            let succeeded = outcome.is_ok();
            match outcome {
                Ok(LifecycleJobResult::Started(outcome)) => {
                    self.apply_start_outcome(*outcome);
                }
                Ok(LifecycleJobResult::Stopped(st)) => {
                    self.apply_stop_status(st);
                }
                Err(e) => {
                    let code = match kind {
                        LifecycleJobKind::Start => ErrorCode::NodeStartFailed,
                        LifecycleJobKind::Stop => ErrorCode::NodeStopFailed,
                    };
                    self.set_problem(code, e);
                    // Refresh to resync honest lifecycle after failure.
                    self.request_status_refresh(ctx);
                }
            }
            match quit_followup_after_lifecycle(self.quit_after_stop, kind, succeeded) {
                QuitFollowup::None => {}
                QuitFollowup::QueueStop => {
                    self.request_lifecycle(LifecycleJobKind::Stop, ctx);
                }
                QuitFollowup::Close => {
                    self.quit_after_stop = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if let Some(event) = self.async_jobs.poll_submit() {
            match event {
                WorkJobEvent::ComparePrimary(primary) => {
                    // P2: show Compare A immediately while B still runs.
                    self.work_result = Some(*primary);
                    self.work_result_b = None;
                    self.work_compare_b_error = None;
                    self.model_triple.used = self.used_model_fact();
                    self.clear_problem();
                }
                WorkJobEvent::Done(outcome) => {
                    let was_prepare = self.async_jobs.take_prepare_and_run_flag();
                    if was_prepare {
                        self.refresh_model_catalog();
                    }
                    match *outcome {
                        Ok(job) => {
                            self.work_result = Some(job.primary);
                            match job.compare_b {
                                None => {
                                    self.work_result_b = None;
                                    self.work_compare_b_error = None;
                                }
                                Some(Ok(b)) => {
                                    self.work_result_b = Some(b);
                                    self.work_compare_b_error = None;
                                }
                                Some(Err(e)) => {
                                    self.work_result_b = None;
                                    self.work_compare_b_error = Some(e);
                                }
                            }
                            self.model_triple.used = self.used_model_fact();
                            self.clear_problem();
                            // Lifecycle may have changed if submit started the node.
                            self.request_status_refresh(ctx);
                        }
                        Err(e) => {
                            if was_prepare {
                                self.last_problem = Some(UiProblem::new(
                                    ErrorCode::WorkModelUnready,
                                    self.ui_lang(),
                                    Some(e),
                                ));
                            } else {
                                self.last_problem =
                                    Some(UiProblem::from_submit_err(&e, self.ui_lang()));
                            }
                        }
                    }
                    // Phase T `#310`: Quit during submit → Stop→Close after submit settles.
                    match quit_followup_after_submit(self.quit_after_stop) {
                        QuitFollowup::None => {}
                        QuitFollowup::QueueStop => {
                            self.request_lifecycle(LifecycleJobKind::Stop, ctx);
                        }
                        QuitFollowup::Close => {
                            self.quit_after_stop = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            }
        }
        if let Some(outcome) = self.async_jobs.poll_refresh() {
            match outcome {
                Ok(snap) => {
                    self.apply_status_snapshot(snap);
                }
                Err(e) => self.set_problem(ErrorCode::StatusRefreshFailed, e),
            }
        }
        // Phase T `#308`: opt-in dial completes off-thread; apply evidence / error here.
        if let Some(outcome) = self.async_jobs.poll_dial() {
            match outcome {
                Ok(out) => {
                    self.dial_msg = Some(format!(
                        "confirmed handshake {}",
                        out.evidence.summary_line()
                    ));
                    self.clear_problem();
                    self.request_status_refresh(ctx);
                }
                Err(e) => {
                    self.dial_msg = None;
                    self.set_problem(ErrorCode::Generic, e);
                }
            }
        }
        // Pack D: catalog / ollama-list workers.
        if let Some((kind, outcome)) = self.async_jobs.poll_catalog() {
            match (kind, outcome) {
                (
                    _,
                    Ok(CatalogJobResult::Scan(snap))
                    | Ok(CatalogJobResult::Prepare(snap))
                    | Ok(CatalogJobResult::Verify(snap))
                    | Ok(CatalogJobResult::Add(snap)),
                ) => {
                    self.apply_catalog_snapshot(snap);
                }
                (_, Ok(CatalogJobResult::Select { chosen, snap })) => {
                    self.catalog_highlight = Some(chosen.clone());
                    self.catalog_auto = false;
                    self.apply_catalog_snapshot(snap);
                    // Request selection does not write the default tip. Make default is separate.
                    if chosen.starts_with("aira:model:ollama-") {
                        self.catalog_msg = Some(self.labels().catalog_not_prepare.into());
                    }
                }
                (_, Ok(CatalogJobResult::OllamaList(names))) => {
                    let empty = names.is_empty();
                    self.apply_ollama_list_outcome(Ok(names));
                    if self.ollama_bind_pending {
                        self.finish_pending_ollama_bind();
                    } else if empty {
                        self.ollama_msg = Some(self.labels().settings_ollama_empty.into());
                    } else {
                        self.ollama_msg = Some(format!(
                            "{} ({})",
                            self.labels().settings_ollama_listed,
                            self.ollama_models.len()
                        ));
                    }
                }
                (CatalogJobKind::OllamaList, Err(e)) => {
                    // `#365`: keep prior snapshot; mark Unknown — do not clear names.
                    self.apply_ollama_list_outcome(Err(e.clone()));
                    self.ollama_bind_pending = false;
                    self.ollama_msg =
                        Some(format!("{} ({e})", self.labels().settings_ollama_unknown));
                }
                (_, Err(e)) => {
                    self.catalog_msg = Some(e);
                }
            }
        }
        let ctx2 = ctx.clone();
        let _ = self.async_jobs.maybe_schedule_periodic_refresh(
            self.paths.clone(),
            self.settings.clone(),
            move || ctx2.request_repaint(),
        );
    }

    /// Reload Network tab mesh fields from node root (orchestrates peer APIs only).
    pub(super) fn refresh_mesh_snapshot(&mut self) {
        match load_system_snapshot(&self.paths.data_root, self.settings.peer_listen.as_deref()) {
            Ok(sys) => {
                self.mesh_snapshot = sys.network.clone();
                self.system_snapshot = sys;
                self.refresh_model_triple();
            }
            Err(e) => {
                self.mesh_snapshot = NetworkMeshSnapshot::unavailable();
                self.system_snapshot = SystemSnapshot::unavailable();
                self.model_triple =
                    ModelTripleSnapshot::undefined().with_used(self.used_model_fact());
                self.set_problem(ErrorCode::MeshSnapshotFailed, format!("{e:#}"));
            }
        }
    }

    pub(super) fn clear_problem(&mut self) {
        self.last_problem = None;
    }

    pub(super) fn set_problem(&mut self, code: ErrorCode, detail: impl ToString) {
        self.last_problem = Some(UiProblem::from_code_err(code, self.ui_lang(), detail));
    }

    /// F1 routing: focused element → section → screen (never stale `last_problem`).
    pub(super) fn resolve_contextual_help(&self) -> HelpId {
        resolve_help_routing(
            self.help_focus,
            self.tab,
            self.work_result.is_some()
                || self.work_result_b.is_some()
                || self.work_compare_b_error.is_some(),
            self.async_jobs.work_inflight(),
        )
    }

    /// Remember the active control/section for the next F1 (`#273`).
    pub(super) fn note_help_focus(&mut self, id: HelpId) {
        self.help_focus = Some(id);
    }

    /// Open Help·F1 for an explicit topic; clears stale search (`#273`).
    ///
    /// Does not clear draft / settings / in-flight work.
    pub(super) fn open_help(&mut self, topic: HelpId) {
        self.help_topic = topic;
        self.help_search.clear();
        self.help_open = true;
    }

    /// Open Help from F1 / chrome: focus → section → screen (`#273`).
    pub(super) fn open_help_contextual(&mut self) {
        let topic = self.resolve_contextual_help();
        self.open_help(topic);
    }

    /// Close only the help panel (Esc); never cancels work.
    pub(super) fn close_help(&mut self) {
        self.help_open = false;
    }

    /// Tab change drops element focus so F1 falls back to section/screen (`#273`).
    pub(super) fn set_tab(&mut self, tab: MainTab) {
        if self.tab != tab {
            self.tab = tab;
            self.help_focus = None;
        }
    }

    /// Settings lifecycle phase: draft Changed → disk Restart/Applied/Undefined (`#262` / `#356`).
    pub(super) fn settings_apply_phase(&self) -> crate::settings_apply::SettingsApplyPhase {
        let dirty = crate::settings_apply::settings_connection_draft_dirty(
            &self.settings,
            &self.peer_listen_edit,
            &self.relay_ttl_edit,
        );
        crate::settings_apply::settings_apply_phase(
            &self.settings,
            self.applied_runtime.as_ref(),
            dirty,
        )
    }

    /// True when saved network/listen differ from applied (Stop→Start needed).
    ///
    /// Ignores unsaved draft (`#356`) — restart CTA is for disk vs runtime only.
    pub(super) fn settings_need_restart(&self) -> bool {
        matches!(
            crate::settings_apply::settings_apply_phase_disk(
                &self.settings,
                self.applied_runtime.as_ref(),
            ),
            crate::settings_apply::SettingsApplyPhase::RestartNeeded
        ) || self.restart_hint
    }

    /// Record an apply/persist failure inline on Settings (`#356`).
    pub(super) fn note_settings_apply_error(&mut self, detail: String) {
        self.settings_apply_error = Some(detail.clone());
        self.set_problem(crate::lexicon::ErrorCode::SettingsPersistFailed, detail);
    }

    /// Clear inline apply error after a successful save.
    pub(super) fn clear_settings_apply_error(&mut self) {
        self.settings_apply_error = None;
    }

    /// Restore Connection draft fields from disk-saved settings (`#356`).
    pub(super) fn cancel_connection_draft(&mut self) {
        self.peer_listen_edit = self
            .settings
            .peer_listen
            .clone()
            .unwrap_or_else(|| DEFAULT_PEER_LISTEN.to_string());
        self.relay_ttl_edit = self
            .settings
            .relay_ttl_days
            .map(|d| d.to_string())
            .unwrap_or_else(|| DEFAULT_RELAY_TTL_DAYS.to_string());
        self.clear_settings_apply_error();
    }

    /// Confirm applied values from a successful Start / attach outcome (`#268` / `#282`).
    ///
    /// Uses the worker's `used_settings` snapshot, not live UI settings.
    pub(super) fn mark_settings_applied_from_outcome(
        &mut self,
        outcome: &aira_desktop_runtime::StartOutcome,
    ) {
        self.applied_runtime = Some(
            crate::settings_apply::AppliedRuntimeSettings::from_start_outcome(
                outcome,
                &outcome.used_settings,
            ),
        );
        self.restart_hint = false;
    }

    /// Clear applied confirmation (node stopped / unconfirmed).
    pub(super) fn clear_applied_runtime(&mut self) {
        self.applied_runtime = None;
    }

    /// Queue Start/Stop off the UI thread (`#272` / `#302`). Invalidates in-flight refresh.
    /// Rejected while Work submit is in flight (no parallel `start()`).
    pub(super) fn request_lifecycle(&mut self, kind: LifecycleJobKind, ctx: &egui::Context) {
        let action = match kind {
            LifecycleJobKind::Start => ActionId::NodeStart,
            LifecycleJobKind::Stop => ActionId::NodeStop,
        };
        let gate = crate::lexicon::lifecycle_action_gate(
            action,
            self.async_jobs.work_inflight(),
            self.async_jobs.lifecycle_inflight(),
        );
        if !gate.available {
            if let Some(code) = gate.reason {
                self.last_problem = Some(UiProblem::new(code, self.ui_lang(), None));
            }
            return;
        }
        match kind {
            LifecycleJobKind::Start => {
                self.lifecycle = LifecycleStatus::Starting;
                self.status_label =
                    labels::status_label(LifecycleStatus::Starting, self.ui_lang()).to_string();
            }
            LifecycleJobKind::Stop => {
                self.lifecycle = LifecycleStatus::Stopping;
                self.status_label =
                    labels::status_label(LifecycleStatus::Stopping, self.ui_lang()).to_string();
            }
        }
        let ctx = ctx.clone();
        let started = self.async_jobs.try_spawn_lifecycle(
            kind,
            self.paths.clone(),
            self.node_bin.clone(),
            move || ctx.request_repaint(),
        );
        if !started {
            let _ = self.refresh_status();
        }
    }

    /// Apply a completed Start outcome on the UI thread.
    pub(super) fn apply_start_outcome(&mut self, outcome: aira_desktop_runtime::StartOutcome) {
        self.lifecycle = outcome.status;
        self.node_running = matches!(outcome.status, LifecycleStatus::Running);
        self.status_label = labels::status_label(outcome.status, self.ui_lang()).to_string();
        let l = self.labels();
        self.detail = format!(
            "{}pid {:?} · {} · {}",
            if outcome.attached { "attached · " } else { "" },
            outcome.pid,
            outcome.listen,
            outcome.instance_id
        );
        self.peer_detail = match (outcome.peer_pid, outcome.peer_listen.as_ref()) {
            (Some(pp), Some(pl)) => labels::format_peer_running(
                self.settings.network_profile,
                pp,
                pl,
                self.settings.relay_ttl_days,
            ),
            _ if self.settings.network_profile.requires_peer_listen() => {
                labels::format_peer_not_running(self.settings.network_profile)
            }
            _ => l.peer_off_p0.into(),
        };
        self.mark_settings_applied_from_outcome(&outcome);
        self.clear_problem();
        self.refresh_mesh_snapshot();
    }

    /// Apply a completed Stop status on the UI thread.
    pub(super) fn apply_stop_status(&mut self, st: LifecycleStatus) {
        self.lifecycle = st;
        self.node_running = false;
        self.status_label = labels::status_label(st, self.ui_lang()).to_string();
        self.detail.clear();
        self.peer_detail.clear();
        self.clear_applied_runtime();
        self.clear_problem();
        self.refresh_mesh_snapshot();
    }

    /// Quit: Stop off-thread, then close (`#272` / `#282` / `#310`).
    ///
    /// If Start is in flight, only set the flag — `pump_async_jobs` queues Stop after Start.
    /// If Submit is in flight (`#310`), arm the same flag and defer Stop until submit
    /// settles (then Stop→Close), with an explicit waiting label — never leave a sticky
    /// unused quit intent after a rejected Stop.
    pub(super) fn request_quit(&mut self, ctx: &egui::Context) {
        self.quit_after_stop = true;
        match quit_arm_policy(self.async_jobs.work_inflight()) {
            QuitArm::DeferUntilSubmitDone => {
                self.status_label = self.labels().quit_waiting_submit.to_string();
            }
            QuitArm::ArmLifecycle => {
                if !self.async_jobs.lifecycle_inflight() {
                    self.request_lifecycle(LifecycleJobKind::Stop, ctx);
                }
            }
        }
    }

    pub(super) fn persist_settings(&mut self) -> anyhow::Result<()> {
        actions::persist_settings(&self.paths, &self.settings)?;
        sync_autostart_from_settings(self.settings.autostart_on_login)?;
        self.clear_problem();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_sections_bind_default_help_topics() {
        assert_eq!(MainTab::Work.default_help(), HelpId::WorkSubmit);
        assert_eq!(MainTab::System.default_help(), HelpId::NetworkReachability);
        assert_eq!(MainTab::Settings.default_help(), HelpId::SettingsApply);
    }

    #[test]
    fn help_routing_prefers_focus_over_section() {
        assert_eq!(
            resolve_help_routing(Some(HelpId::ModelSelect), MainTab::Work, false, false),
            HelpId::ModelSelect
        );
        assert_eq!(
            resolve_help_routing(None, MainTab::Work, true, false),
            HelpId::WorkResult
        );
        assert_eq!(
            resolve_help_routing(None, MainTab::Work, false, true),
            HelpId::WorkWaiting
        );
        assert_eq!(
            resolve_help_routing(None, MainTab::System, false, false),
            HelpId::NetworkReachability
        );
        assert_eq!(
            resolve_help_routing(None, MainTab::Settings, false, false),
            HelpId::SettingsApply
        );
    }

    #[test]
    fn help_routing_ignores_last_problem_by_design() {
        // Pure resolver has no last_problem parameter — F1 never takes it.
        assert_eq!(
            resolve_help_routing(None, MainTab::System, false, false),
            HelpId::NetworkReachability
        );
    }
}
