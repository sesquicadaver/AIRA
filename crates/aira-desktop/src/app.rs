//! egui application state for AIRA Desktop.

use std::path::PathBuf;

use aira_desktop_runtime::{
    load_or_create_settings, start, status, stop, sync_autostart_from_settings, write_settings,
    DesktopPaths, DesktopSettings, LifecycleStatus,
};

pub struct AiraDesktopApp {
    paths: DesktopPaths,
    node_bin: Option<PathBuf>,
    settings: DesktopSettings,
    status_label: String,
    detail: String,
    last_error: Option<String>,
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
        let mut app = Self {
            paths,
            node_bin,
            settings,
            status_label: "stopped".into(),
            detail: String::new(),
            last_error,
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
        self.detail = match rec {
            Some(r) => format!("pid {} · {} · {}", r.pid, r.listen, r.instance_id),
            None => format!("listen {}", self.settings.http_listen),
        };
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
        self.last_error = None;
        Ok(())
    }

    fn do_stop(&mut self) -> anyhow::Result<()> {
        let st = stop(&self.paths)?;
        self.status_label = status_label(st).to_string();
        self.detail.clear();
        self.last_error = None;
        Ok(())
    }

    fn persist_settings(&mut self) -> anyhow::Result<()> {
        write_settings(&self.paths, &self.settings)?;
        sync_autostart_from_settings(self.settings.autostart_on_login)?;
        self.last_error = None;
        Ok(())
    }
}

impl eframe::App for AiraDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AIRA Desktop");
            ui.label("Developer Preview · P0 local loopback");
            ui.separator();

            ui.horizontal(|ui| {
                ui.strong("Status:");
                ui.label(&self.status_label);
            });
            if !self.detail.is_empty() {
                ui.label(&self.detail);
            }
            ui.label(format!("data_root: {}", self.paths.data_root.display()));
            ui.label(format!("settings: {}", self.paths.settings_file.display()));

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

            if let Some(err) = &self.last_error {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(200, 60, 60), err);
            }
        });

        // Periodic status refresh.
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
