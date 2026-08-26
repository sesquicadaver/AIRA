//! egui application state for AIRA Desktop (QUEUE #78 / #85 / #130 split).

mod discovery;
mod federation;
mod invite;
mod labels;
mod profile;
mod ui;

use std::path::PathBuf;

use aira_desktop_runtime::{
    load_or_create_settings, start, status, stop, sync_autostart_from_settings, DesktopPaths,
    DesktopSettings, DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};

use crate::actions;
use crate::camera;

pub struct AiraDesktopApp {
    pub(super) paths: DesktopPaths,
    pub(super) node_bin: Option<PathBuf>,
    pub(super) settings: DesktopSettings,
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

    pub(super) fn refresh_status(&mut self) -> anyhow::Result<()> {
        let (st, rec) = status(&self.paths)?;
        self.status_label = labels::status_label(st).to_string();
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
                    _ => "peer off (P0)".into(),
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
                    "peer off (P0)".into()
                };
            }
        }
        Ok(())
    }

    pub(super) fn do_start(&mut self) -> anyhow::Result<()> {
        let outcome = start(&self.paths, self.node_bin.clone())?;
        self.status_label = labels::status_label(outcome.status).to_string();
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
            _ => "peer off (P0)".into(),
        };
        self.restart_hint = false;
        self.last_error = None;
        Ok(())
    }

    pub(super) fn do_stop(&mut self) -> anyhow::Result<()> {
        let st = stop(&self.paths)?;
        self.status_label = labels::status_label(st).to_string();
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
