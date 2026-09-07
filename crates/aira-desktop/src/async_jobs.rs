//! Non-blocking Desktop jobs (`#257`): submit + status refresh off the egui thread.
//!
//! `request_repaint_after` only schedules a redraw; data refresh is a separate job.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use aira_desktop_runtime::{
    load_network_mesh_snapshot, start, status, DesktopPaths, DesktopSettings, LifecycleStatus,
    NetworkMeshSnapshot, PidRecordView,
};

use crate::actions;
use crate::work_view::WorkResultView;

/// Interval for light status polling while the window is open.
pub const STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Authoritative status payload collected off the UI thread.
#[derive(Debug, Clone)]
pub struct StatusSnapshot {
    pub lifecycle: LifecycleStatus,
    pub record: Option<PidRecordView>,
    pub mesh: NetworkMeshSnapshot,
}

/// Collect lifecycle + mesh without touching egui.
pub fn collect_status_snapshot(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
) -> anyhow::Result<StatusSnapshot> {
    let (lifecycle, record) = status(paths)?;
    let mesh = load_network_mesh_snapshot(&paths.data_root, settings.peer_listen.as_deref())?;
    Ok(StatusSnapshot {
        lifecycle,
        record,
        mesh,
    })
}

/// Ensure node is running (if requested), then submit work — all off the UI thread.
pub fn run_submit_job(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
    node_bin: Option<PathBuf>,
    text: &str,
    ensure_started: bool,
) -> anyhow::Result<WorkResultView> {
    if ensure_started {
        let (st, _) = status(paths)?;
        if !matches!(st, LifecycleStatus::Running) {
            let _ = start(paths, node_bin)?;
        }
    }
    actions::submit_problem(paths, settings, text)
}

/// In-flight submit / refresh slots (at most one of each).
#[derive(Debug, Default)]
pub struct AsyncDesktopJobs {
    work_rx: Option<Receiver<Result<WorkResultView, String>>>,
    refresh_rx: Option<Receiver<(u64, Result<StatusSnapshot, String>)>>,
    /// Monotonic generation so a stale refresh cannot overwrite a newer one.
    refresh_generation: u64,
    last_refresh_started: Option<Instant>,
}

impl AsyncDesktopJobs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn work_inflight(&self) -> bool {
        self.work_rx.is_some()
    }

    pub fn refresh_inflight(&self) -> bool {
        self.refresh_rx.is_some()
    }

    /// Start at most one submit worker. Returns false if already in flight.
    pub fn try_spawn_submit(
        &mut self,
        paths: DesktopPaths,
        settings: DesktopSettings,
        node_bin: Option<PathBuf>,
        text: String,
        ensure_started: bool,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if self.work_rx.is_some() {
            return false;
        }
        let (tx, rx) = mpsc::channel();
        self.work_rx = Some(rx);
        thread::spawn(move || {
            let outcome = run_submit_job(&paths, &settings, node_bin, &text, ensure_started)
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(outcome);
            on_done();
        });
        true
    }

    /// Start at most one status refresh. Returns false if already in flight.
    pub fn try_spawn_refresh(
        &mut self,
        paths: DesktopPaths,
        settings: DesktopSettings,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if self.refresh_rx.is_some() {
            return false;
        }
        self.refresh_generation = self.refresh_generation.wrapping_add(1);
        let gen = self.refresh_generation;
        let (tx, rx) = mpsc::channel();
        self.refresh_rx = Some(rx);
        self.last_refresh_started = Some(Instant::now());
        thread::spawn(move || {
            let outcome = collect_status_snapshot(&paths, &settings).map_err(|e| format!("{e:#}"));
            let _ = tx.send((gen, outcome));
            on_done();
        });
        true
    }

    /// Kick a refresh when the interval elapsed and no refresh is running.
    pub fn maybe_schedule_periodic_refresh(
        &mut self,
        paths: DesktopPaths,
        settings: DesktopSettings,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if self.refresh_inflight() {
            return false;
        }
        let due = match self.last_refresh_started {
            None => true,
            Some(t) => t.elapsed() >= STATUS_REFRESH_INTERVAL,
        };
        if !due {
            return false;
        }
        self.try_spawn_refresh(paths, settings, on_done)
    }

    /// Non-blocking poll for a finished submit.
    pub fn poll_submit(&mut self) -> Option<Result<WorkResultView, String>> {
        let Some(rx) = self.work_rx.as_ref() else {
            return None;
        };
        match rx.try_recv() {
            Ok(v) => {
                self.work_rx = None;
                Some(v)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.work_rx = None;
                Some(Err("submit worker disconnected".into()))
            }
        }
    }

    /// Non-blocking poll for a finished refresh (drops stale generations).
    pub fn poll_refresh(&mut self) -> Option<Result<StatusSnapshot, String>> {
        let Some(rx) = self.refresh_rx.as_ref() else {
            return None;
        };
        match rx.try_recv() {
            Ok((gen, outcome)) => {
                self.refresh_rx = None;
                if gen != self.refresh_generation {
                    // Superseded — ignore payload but clear slot.
                    return None;
                }
                Some(outcome)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.refresh_rx = None;
                Some(Err("refresh worker disconnected".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aira_desktop_runtime::DesktopPaths;

    #[test]
    fn second_submit_rejected_while_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.work_rx = Some(hold_rx);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-a"));
        let settings = DesktopSettings::default_p0(&paths);
        assert!(!jobs.try_spawn_submit(
            paths,
            settings,
            None,
            "Calculate 2 + 2".into(),
            false,
            || {}
        ));
    }

    #[test]
    fn second_refresh_rejected_while_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.refresh_rx = Some(hold_rx);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-b"));
        let settings = DesktopSettings::default_p0(&paths);
        assert!(!jobs.try_spawn_refresh(paths, settings, || {}));
        assert_eq!(jobs.refresh_generation, 0);
    }

    #[test]
    fn refresh_generation_advances_on_spawn() {
        let mut jobs = AsyncDesktopJobs::new();
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let settings = DesktopSettings::default_p0(&paths);
        assert!(jobs.try_spawn_refresh(paths.clone(), settings.clone(), || {}));
        assert_eq!(jobs.refresh_generation, 1);
        for _ in 0..200 {
            if jobs.poll_refresh().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!jobs.refresh_inflight());
        assert!(jobs.try_spawn_refresh(paths, settings, || {}));
        assert_eq!(jobs.refresh_generation, 2);
        for _ in 0..200 {
            if jobs.poll_refresh().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn submit_empty_fails_closed_off_thread() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let settings = DesktopSettings::default_p0(&paths);
        let err = run_submit_job(&paths, &settings, None, "  \n", false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("non-empty"), "{err}");
    }

    #[test]
    fn periodic_refresh_respects_interval() {
        let mut jobs = AsyncDesktopJobs::new();
        jobs.last_refresh_started = Some(Instant::now());
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-c"));
        let settings = DesktopSettings::default_p0(&paths);
        assert!(!jobs.maybe_schedule_periodic_refresh(paths, settings, || {}));
    }

    #[test]
    fn status_refresh_interval_is_independent_of_repaint() {
        assert_eq!(STATUS_REFRESH_INTERVAL, Duration::from_secs(2));
    }
}
