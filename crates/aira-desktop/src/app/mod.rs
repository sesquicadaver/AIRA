//! egui application state for AIRA Desktop (QUEUE #78 / #85 / #130 split).

mod discovery;
mod federation;
mod i18n;
mod invite;
mod labels;
mod profile;
mod ui;
mod work;

use std::path::PathBuf;

use aira_desktop_runtime::{
    load_or_create_settings, load_or_create_ui_prefs, load_system_snapshot, start, stop,
    sync_autostart_from_settings, write_ui_prefs, DesktopPaths, DesktopSettings, LifecycleStatus,
    NetworkMeshSnapshot, SystemSnapshot, UiLang, UiPrefs, DEFAULT_PEER_LISTEN,
    DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;
use crate::async_jobs::{AsyncDesktopJobs, StatusSnapshot};
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
    pub(super) last_problem: Option<UiProblem>,
    pub(super) qr_texture: Option<egui::TextureHandle>,
    pub(super) qr_camera: Option<camera::InviteQrCamera>,
    pub(super) qr_camera_status: Option<String>,
    pub(super) restart_hint: bool,
    /// Runtime-applied network/listen subset (`#262`); differs from disk → Restart needed.
    pub(super) applied_runtime: crate::settings_apply::AppliedRuntimeSettings,
    pub(super) async_jobs: AsyncDesktopJobs,
    /// Side help panel open (`#259` chrome; topics filled in `#263`).
    pub(super) help_open: bool,
    /// Active help topic key for the panel.
    pub(super) help_topic: HelpId,
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
        let applied_runtime =
            crate::settings_apply::AppliedRuntimeSettings::from_settings(&settings);
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
            last_problem,
            qr_texture: None,
            qr_camera: None,
            qr_camera_status: None,
            restart_hint: false,
            applied_runtime,
            async_jobs: AsyncDesktopJobs::new(),
            help_open: false,
            help_topic: HelpId::Start,
        };
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            Labels::get(app.ui_lang()).window_title.to_string(),
        ));
        let _ = app.refresh_status();
        app.refresh_federation_detail();
        if auto_start {
            if let Err(e) = app.do_start() {
                app.set_problem(ErrorCode::NodeStartFailed, format!("{e:#}"));
            }
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

    /// Apply a status snapshot collected off the UI thread (`#257`).
    pub(super) fn apply_status_snapshot(&mut self, snap: StatusSnapshot) {
        self.lifecycle = snap.lifecycle;
        self.node_running = matches!(snap.lifecycle, LifecycleStatus::Running);
        self.status_label = labels::status_label(snap.lifecycle, self.ui_lang()).to_string();
        let l = self.labels();
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

    /// Poll background jobs and schedule periodic data refresh (`#257`).
    pub(super) fn pump_async_jobs(&mut self, ctx: &egui::Context) {
        if let Some(outcome) = self.async_jobs.poll_submit() {
            match outcome {
                Ok(view) => {
                    self.work_result = Some(view);
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
            }
            Err(e) => {
                self.mesh_snapshot = NetworkMeshSnapshot::unavailable();
                self.system_snapshot = SystemSnapshot::unavailable();
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

    /// Default Help topic for the current shell section (`#259`).
    pub(super) fn help_topic_for_tab(&self) -> HelpId {
        self.tab.default_help()
    }

    /// Open Help·F1 for an explicit topic (does not clear draft / settings).
    pub(super) fn open_help(&mut self, topic: HelpId) {
        self.help_topic = topic;
        self.help_open = true;
    }

    /// Open Help for the current context: last problem → section default.
    pub(super) fn open_help_contextual(&mut self) {
        let topic = self
            .last_problem
            .as_ref()
            .map(|p| p.help_id)
            .unwrap_or_else(|| self.help_topic_for_tab());
        self.open_help(topic);
    }

    /// Close only the help panel (Esc); never cancels work.
    pub(super) fn close_help(&mut self) {
        self.help_open = false;
    }

    /// Settings lifecycle phase from saved disk vs runtime-applied (`#262`).
    pub(super) fn settings_apply_phase(&self) -> crate::settings_apply::SettingsApplyPhase {
        crate::settings_apply::settings_apply_phase(&self.settings, &self.applied_runtime)
    }

    /// True when saved network/listen differ from applied (Stop→Start needed).
    pub(super) fn settings_need_restart(&self) -> bool {
        matches!(
            self.settings_apply_phase(),
            crate::settings_apply::SettingsApplyPhase::RestartNeeded
        ) || self.restart_hint
    }

    /// Mark current settings as applied to the running node (after Start).
    pub(super) fn mark_settings_applied(&mut self) {
        self.applied_runtime =
            crate::settings_apply::AppliedRuntimeSettings::from_settings(&self.settings);
        self.restart_hint = false;
    }

    pub(super) fn do_start(&mut self) -> anyhow::Result<()> {
        let outcome = start(&self.paths, self.node_bin.clone())?;
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
        self.mark_settings_applied();
        self.clear_problem();
        self.refresh_mesh_snapshot();
        Ok(())
    }

    pub(super) fn do_stop(&mut self) -> anyhow::Result<()> {
        let st = stop(&self.paths)?;
        self.lifecycle = st;
        self.node_running = false;
        self.status_label = labels::status_label(st, self.ui_lang()).to_string();
        self.detail.clear();
        self.peer_detail.clear();
        self.clear_problem();
        self.refresh_mesh_snapshot();
        Ok(())
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
}
