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
    load_or_create_settings, load_or_create_ui_prefs, load_system_snapshot,
    sync_autostart_from_settings, write_ui_prefs, DesktopPaths, DesktopSettings, LifecycleStatus,
    ModelFact, ModelTripleSnapshot, NetworkMeshSnapshot, SystemSnapshot, UiLang, UiPrefs,
    DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;
use crate::async_jobs::{
    quit_followup_after_lifecycle, AsyncDesktopJobs, LifecycleJobKind, LifecycleJobResult,
    QuitFollowup, StatusSnapshot,
};
use crate::camera;
use crate::lexicon::{ErrorCode, HelpId, UiProblem};

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
    pub(super) status_label: String,
    pub(super) detail: String,
    pub(super) peer_detail: String,
    pub(super) mesh_snapshot: NetworkMeshSnapshot,
    pub(super) system_snapshot: SystemSnapshot,
    /// Model selected ≠ ready ≠ used (`#269`).
    pub(super) model_triple: ModelTripleSnapshot,
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
        let applied_runtime = None;
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
            status_label: Labels::get(UiLang::En).st_stopped.into(),
            detail: String::new(),
            peer_detail: String::new(),
            mesh_snapshot: NetworkMeshSnapshot::unavailable(),
            system_snapshot: SystemSnapshot::unavailable(),
            model_triple: ModelTripleSnapshot::undefined(),
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
        self.model_triple =
            ModelTripleSnapshot::load(&self.paths.data_root).with_used(self.used_model_fact());
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
        if let Some(outcome) = self.async_jobs.poll_submit() {
            match outcome {
                Ok(view) => {
                    self.work_result = Some(view);
                    self.model_triple.used = self.used_model_fact();
                    self.clear_problem();
                    // Lifecycle may have changed if submit started the node.
                    self.request_status_refresh(ctx);
                }
                Err(e) => {
                    self.last_problem = Some(UiProblem::from_submit_err(&e, self.ui_lang()));
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
            self.work_result.is_some(),
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

    /// Settings lifecycle phase from saved disk vs runtime-applied (`#262`).
    pub(super) fn settings_apply_phase(&self) -> crate::settings_apply::SettingsApplyPhase {
        crate::settings_apply::settings_apply_phase(&self.settings, self.applied_runtime.as_ref())
    }

    /// True when saved network/listen differ from applied (Stop→Start needed).
    pub(super) fn settings_need_restart(&self) -> bool {
        matches!(
            self.settings_apply_phase(),
            crate::settings_apply::SettingsApplyPhase::RestartNeeded
        ) || self.restart_hint
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

    /// Queue Start/Stop off the UI thread (`#272`). Invalidates in-flight refresh.
    pub(super) fn request_lifecycle(&mut self, kind: LifecycleJobKind, ctx: &egui::Context) {
        if self.async_jobs.lifecycle_inflight() {
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

    /// Quit: Stop off-thread, then close (`#272` / `#282`).
    ///
    /// If Start is in flight, only set the flag — `pump_async_jobs` queues Stop after Start.
    pub(super) fn request_quit(&mut self, ctx: &egui::Context) {
        self.quit_after_stop = true;
        if self.async_jobs.lifecycle_inflight() {
            return;
        }
        self.request_lifecycle(LifecycleJobKind::Stop, ctx);
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
