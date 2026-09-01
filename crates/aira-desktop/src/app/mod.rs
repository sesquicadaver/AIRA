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
    load_or_create_settings, load_or_create_ui_prefs, start, status, stop,
    sync_autostart_from_settings, write_ui_prefs, DesktopPaths, DesktopSettings, LifecycleStatus,
    UiLang, UiPrefs, DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;
use crate::camera;

use self::i18n::Labels;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MainTab {
    Work,
    Node,
    Network,
    Settings,
}

pub struct AiraDesktopApp {
    pub(super) paths: DesktopPaths,
    pub(super) node_bin: Option<PathBuf>,
    pub(super) settings: DesktopSettings,
    pub(super) ui_prefs: UiPrefs,
    pub(super) tab: MainTab,
    pub(super) node_running: bool,
    pub(super) problem_text: String,
    pub(super) work_result: Option<crate::work_view::WorkResultView>,
    pub(super) status_label: String,
    pub(super) detail: String,
    pub(super) peer_detail: String,
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
    pub(super) last_error: Option<String>,
    pub(super) qr_texture: Option<egui::TextureHandle>,
    pub(super) qr_camera: Option<camera::InviteQrCamera>,
    pub(super) qr_camera_status: Option<String>,
    pub(super) restart_hint: bool,
}

impl AiraDesktopApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        paths: DesktopPaths,
        node_bin: Option<PathBuf>,
        auto_start: bool,
    ) -> Self {
        i18n::install_cyrillic_font(&cc.egui_ctx);
        let mut last_error = None;
        let settings = match load_or_create_settings(&paths) {
            Ok(s) => s,
            Err(e) => {
                last_error = Some(format!("{e:#}"));
                DesktopSettings::default_p0(&paths)
            }
        };
        let ui_prefs = match load_or_create_ui_prefs(&paths) {
            Ok(p) => p,
            Err(e) => {
                last_error = Some(format!("ui prefs: {e:#}"));
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
        let mut app = Self {
            paths,
            node_bin,
            settings,
            ui_prefs,
            tab: MainTab::Work,
            node_running: false,
            problem_text: String::new(),
            work_result: None,
            status_label: Labels::get(UiLang::En).st_stopped.into(),
            detail: String::new(),
            peer_detail: String::new(),
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
            last_error,
            qr_texture: None,
            qr_camera: None,
            qr_camera_status: None,
            restart_hint: false,
        };
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Title(
            Labels::get(app.ui_lang()).window_title.to_string(),
        ));
        let _ = app.refresh_status();
        app.refresh_federation_detail();
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
            self.last_error = Some(format!("{e:#}"));
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
        let (st, rec) = status(&self.paths)?;
        self.node_running = matches!(st, LifecycleStatus::Running);
        self.status_label = labels::status_label(st, self.ui_lang()).to_string();
        let l = self.labels();
        match rec {
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
        Ok(())
    }

    pub(super) fn do_start(&mut self) -> anyhow::Result<()> {
        let outcome = start(&self.paths, self.node_bin.clone())?;
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
        self.restart_hint = false;
        self.last_error = None;
        Ok(())
    }

    pub(super) fn do_stop(&mut self) -> anyhow::Result<()> {
        let st = stop(&self.paths)?;
        self.node_running = false;
        self.status_label = labels::status_label(st, self.ui_lang()).to_string();
        self.detail.clear();
        self.peer_detail.clear();
        self.last_error = None;
        Ok(())
    }

    pub(super) fn persist_settings(&mut self) -> anyhow::Result<()> {
        actions::persist_settings(&self.paths, &self.settings)?;
        sync_autostart_from_settings(self.settings.autostart_on_login)?;
        self.last_error = None;
        Ok(())
    }
}
