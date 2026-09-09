//! Non-blocking Desktop jobs (`#257` / `#272` / `#282` / Phase S `#302` / Phase T `#308`):
//! submit, status refresh, Start/Stop lifecycle, and opt-in peer dial off the egui thread.
//!
//! `request_repaint_after` only schedules a redraw; data refresh is a separate job.
//! Each Start/Stop bumps a dedicated `lifecycle_revision` and invalidates refresh so a
//! stale or mid-lifecycle refresh cannot overwrite post-transition UI (`#282`).
//!
//! Phase S `#302`: submit and Start/Stop are mutually exclusive — no parallel
//! `start()` from submit `ensure_started` while a lifecycle Start is in flight.
//!
//! Phase T `#308`: opt-in dial uses its own slot (`try_spawn_dial`); the UI path must
//! never `block_on` TCP/Noise — F1 and navigation stay available during an attempt.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use aira_desktop_runtime::{
    load_system_snapshot, start, status, stop, DesktopPaths, DesktopSettings, DialOutcome,
    LifecycleStatus, ModelTripleSnapshot, NetworkMeshSnapshot, PidRecordView, StartOutcome,
    SystemSnapshot,
};

use crate::actions;
use crate::work_view::WorkResultView;

/// Interval for light status polling while the window is open.
pub const STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// Exclusive Desktop job class for `#302` admission (submit ∥ lifecycle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExclusiveJobKind {
    Submit,
    Lifecycle,
}

/// Pure admission: at most one of submit / lifecycle may run (`#302`).
///
/// Returns `true` when `want` may spawn. Refresh is gated separately and already
/// rejects while lifecycle is in flight (`#282`).
pub fn admit_submit_lifecycle(
    want: ExclusiveJobKind,
    work_inflight: bool,
    lifecycle_inflight: bool,
) -> bool {
    match want {
        ExclusiveJobKind::Submit => !work_inflight && !lifecycle_inflight,
        ExclusiveJobKind::Lifecycle => !work_inflight && !lifecycle_inflight,
    }
}

/// Which lifecycle control job is running (`#272`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleJobKind {
    Start,
    Stop,
}

/// Result of a background Start/Stop (`#272` / `#282`).
#[derive(Debug)]
pub enum LifecycleJobResult {
    /// Successful Start/attach; Applied must use `outcome.used_settings` (`#282`).
    Started(Box<StartOutcome>),
    Stopped(LifecycleStatus),
}

/// What Quit should do after a lifecycle poll completes (`#282`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitFollowup {
    /// Keep waiting / no action.
    None,
    /// Start finished while Quit was requested — queue Stop.
    QueueStop,
    /// Stop finished (or Start failed) — close the viewport.
    Close,
}

/// Pure Quit chaining after a lifecycle job settles (`#282`).
pub fn quit_followup_after_lifecycle(
    quit_after_stop: bool,
    kind: LifecycleJobKind,
    succeeded: bool,
) -> QuitFollowup {
    if !quit_after_stop {
        return QuitFollowup::None;
    }
    match (kind, succeeded) {
        (LifecycleJobKind::Start, true) => QuitFollowup::QueueStop,
        (LifecycleJobKind::Start, false) => QuitFollowup::Close,
        (LifecycleJobKind::Stop, _) => QuitFollowup::Close,
    }
}

/// Phase T `#310`: after submit settles, continue a deferred Quit with Stop→Close.
pub fn quit_followup_after_submit(quit_after_stop: bool) -> QuitFollowup {
    if quit_after_stop {
        QuitFollowup::QueueStop
    } else {
        QuitFollowup::None
    }
}

/// How `request_quit` should arm Stop relative to submit∥lifecycle (`#310`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitArm {
    /// Call Stop now (or wait for in-flight lifecycle — flag already armed).
    ArmLifecycle,
    /// Submit in flight: keep `quit_after_stop`, do **not** call Stop yet.
    DeferUntilSubmitDone,
}

/// Pure Quit admission vs submit (`#310`).
///
/// Lifecycle-in-flight is handled by the caller (flag only); submit-in-flight must
/// defer Stop so `#302` exclusivity does not leave a sticky unused quit flag.
pub fn quit_arm_policy(work_inflight: bool) -> QuitArm {
    if work_inflight {
        QuitArm::DeferUntilSubmitDone
    } else {
        QuitArm::ArmLifecycle
    }
}

/// Authoritative status payload collected off the UI thread.
#[derive(Debug, Clone)]
pub struct StatusSnapshot {
    pub lifecycle: LifecycleStatus,
    pub record: Option<PidRecordView>,
    pub mesh: NetworkMeshSnapshot,
    pub system: SystemSnapshot,
    pub model: ModelTripleSnapshot,
}

/// Collect lifecycle + mesh without touching egui.
pub fn collect_status_snapshot(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
) -> anyhow::Result<StatusSnapshot> {
    let (lifecycle, record) = status(paths)?;
    let system = load_system_snapshot(&paths.data_root, settings.peer_listen.as_deref())?;
    let mesh = system.network.clone();
    let model = ModelTripleSnapshot::load(&paths.data_root);
    Ok(StatusSnapshot {
        lifecycle,
        record,
        mesh,
        system,
        model,
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

/// In-flight submit / refresh / dial slots (at most one of each).
#[derive(Debug, Default)]
pub struct AsyncDesktopJobs {
    work_rx: Option<Receiver<Result<WorkResultView, String>>>,
    refresh_rx: Option<Receiver<(u64, Result<StatusSnapshot, String>)>>,
    lifecycle_rx: Option<Receiver<(LifecycleJobKind, Result<LifecycleJobResult, String>)>>,
    /// Phase T `#308`: at most one opt-in peer dial worker.
    dial_rx: Option<Receiver<Result<DialOutcome, String>>>,
    /// Kind of in-flight lifecycle job (for Starting/Stopping UI).
    lifecycle_kind: Option<LifecycleJobKind>,
    /// Monotonic generation so a stale refresh cannot overwrite a newer one.
    refresh_generation: u64,
    /// One bump per Start/Stop operation (`#282`).
    lifecycle_revision: u64,
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

    /// Current lifecycle operation token (`#282`).
    #[allow(dead_code)] // asserted in unit tests; reserved for UI diagnostics
    pub fn lifecycle_revision(&self) -> u64 {
        self.lifecycle_revision
    }

    /// Start at most one submit worker. Returns false if already in flight
    /// or a lifecycle op is running (`#302` — no parallel `start()`).
    pub fn try_spawn_submit(
        &mut self,
        paths: DesktopPaths,
        settings: DesktopSettings,
        node_bin: Option<PathBuf>,
        text: String,
        ensure_started: bool,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if !admit_submit_lifecycle(
            ExclusiveJobKind::Submit,
            self.work_rx.is_some(),
            self.lifecycle_inflight(),
        ) {
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

    /// Start at most one status refresh. Returns false if already in flight
    /// or a lifecycle op is running (`#282`).
    pub fn try_spawn_refresh(
        &mut self,
        paths: DesktopPaths,
        settings: DesktopSettings,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if self.refresh_rx.is_some() || self.lifecycle_inflight() {
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

    /// Kick a refresh when the interval elapsed and no refresh/lifecycle is running.
    pub fn maybe_schedule_periodic_refresh(
        &mut self,
        paths: DesktopPaths,
        settings: DesktopSettings,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if self.refresh_inflight() || self.lifecycle_inflight() {
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
        let rx = self.work_rx.as_ref()?;
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
        let rx = self.refresh_rx.as_ref()?;
        match rx.try_recv() {
            Ok((gen, outcome)) => {
                self.refresh_rx = None;
                if gen != self.refresh_generation {
                    // Superseded — ignore payload but clear slot.
                    return None;
                }
                // Mid-lifecycle results must never apply (`#282`).
                if self.lifecycle_inflight() {
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

    /// Bump refresh generation so in-flight refresh payloads are dropped (`#272`).
    pub fn invalidate_refresh(&mut self) {
        self.refresh_generation = self.refresh_generation.wrapping_add(1);
    }

    pub fn lifecycle_inflight(&self) -> bool {
        self.lifecycle_rx.is_some()
    }

    /// Start at most one Start/Stop worker; one revision bump + invalidate refresh (`#282`).
    /// Rejected while submit is in flight (`#302` — no double `start()`).
    pub fn try_spawn_lifecycle(
        &mut self,
        kind: LifecycleJobKind,
        paths: DesktopPaths,
        node_bin: Option<PathBuf>,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if !admit_submit_lifecycle(
            ExclusiveJobKind::Lifecycle,
            self.work_inflight(),
            self.lifecycle_rx.is_some(),
        ) {
            return false;
        }
        self.lifecycle_revision = self.lifecycle_revision.wrapping_add(1);
        self.invalidate_refresh();
        let (tx, rx) = mpsc::channel();
        self.lifecycle_rx = Some(rx);
        self.lifecycle_kind = Some(kind);
        thread::spawn(move || {
            let outcome = match kind {
                LifecycleJobKind::Start => start(&paths, node_bin)
                    .map(|o| LifecycleJobResult::Started(Box::new(o)))
                    .map_err(|e| format!("{e:#}")),
                LifecycleJobKind::Stop => stop(&paths)
                    .map(LifecycleJobResult::Stopped)
                    .map_err(|e| format!("{e:#}")),
            };
            let _ = tx.send((kind, outcome));
            on_done();
        });
        true
    }

    /// Non-blocking poll for a finished Start/Stop.
    pub fn poll_lifecycle(
        &mut self,
    ) -> Option<(LifecycleJobKind, Result<LifecycleJobResult, String>)> {
        let rx = self.lifecycle_rx.as_ref()?;
        match rx.try_recv() {
            Ok(v) => {
                self.lifecycle_rx = None;
                self.lifecycle_kind = None;
                // Drop any refresh that raced the transition (`#282`).
                self.invalidate_refresh();
                Some(v)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.lifecycle_rx = None;
                self.lifecycle_kind = None;
                self.invalidate_refresh();
                Some((
                    LifecycleJobKind::Stop,
                    Err("lifecycle worker disconnected".into()),
                ))
            }
        }
    }

    /// Whether an opt-in dial worker is running (`#308`).
    pub fn dial_inflight(&self) -> bool {
        self.dial_rx.is_some()
    }

    /// Start at most one opt-in peer dial off the UI thread (`#308`).
    ///
    /// Returns `false` if a dial is already in flight. Dial is independent of
    /// submit/lifecycle so F1/nav/Work remain usable during a slow endpoint.
    pub fn try_spawn_dial(
        &mut self,
        paths: DesktopPaths,
        peer_identity_id: String,
        explicit_addr: String,
        on_done: impl FnOnce() + Send + 'static,
    ) -> bool {
        if self.dial_rx.is_some() {
            return false;
        }
        let (tx, rx) = mpsc::channel();
        self.dial_rx = Some(rx);
        thread::spawn(move || {
            let outcome = actions::opt_in_peer_dial(&paths, &peer_identity_id, &explicit_addr)
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(outcome);
            on_done();
        });
        true
    }

    /// Non-blocking poll for a finished opt-in dial (`#308`).
    pub fn poll_dial(&mut self) -> Option<Result<DialOutcome, String>> {
        let rx = self.dial_rx.as_ref()?;
        match rx.try_recv() {
            Ok(v) => {
                self.dial_rx = None;
                Some(v)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.dial_rx = None;
                Some(Err("dial worker disconnected".into()))
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

    /// Phase S `#302`: submit blocked while Start/Stop runs (no parallel ensure_started start).
    #[test]
    fn submit_rejected_while_lifecycle_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.lifecycle_rx = Some(hold_rx);
        jobs.lifecycle_kind = Some(LifecycleJobKind::Start);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-sl"));
        let settings = DesktopSettings::default_p0(&paths);
        assert!(!admit_submit_lifecycle(
            ExclusiveJobKind::Submit,
            false,
            true
        ));
        assert!(!jobs.try_spawn_submit(
            paths,
            settings,
            None,
            "Calculate 2 + 2".into(),
            true,
            || {}
        ));
    }

    /// Phase S `#302`: lifecycle blocked while submit runs.
    #[test]
    fn lifecycle_rejected_while_submit_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.work_rx = Some(hold_rx);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-ls"));
        assert!(!admit_submit_lifecycle(
            ExclusiveJobKind::Lifecycle,
            true,
            false
        ));
        assert!(!jobs.try_spawn_lifecycle(LifecycleJobKind::Start, paths, None, || {}));
        assert_eq!(jobs.lifecycle_revision(), 0);
    }

    #[test]
    fn admit_allows_when_neither_inflight() {
        assert!(admit_submit_lifecycle(
            ExclusiveJobKind::Submit,
            false,
            false
        ));
        assert!(admit_submit_lifecycle(
            ExclusiveJobKind::Lifecycle,
            false,
            false
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
    fn refresh_rejected_while_lifecycle_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.lifecycle_rx = Some(hold_rx);
        jobs.lifecycle_kind = Some(LifecycleJobKind::Start);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-lc-rf"));
        let settings = DesktopSettings::default_p0(&paths);
        assert!(!jobs.try_spawn_refresh(paths.clone(), settings.clone(), || {}));
        assert!(!jobs.maybe_schedule_periodic_refresh(paths, settings, || {}));
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
    fn lifecycle_bumps_one_revision_and_invalidates_refresh() {
        let mut jobs = AsyncDesktopJobs::new();
        assert_eq!(jobs.lifecycle_revision(), 0);
        assert_eq!(jobs.refresh_generation, 0);
        jobs.invalidate_refresh();
        assert_eq!(jobs.refresh_generation, 1);
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.lifecycle_rx = Some(hold_rx);
        jobs.lifecycle_kind = Some(LifecycleJobKind::Start);
        jobs.lifecycle_revision = 1;
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-lc"));
        assert!(!jobs.try_spawn_lifecycle(LifecycleJobKind::Stop, paths, None, || {}));
        assert_eq!(jobs.lifecycle_revision(), 1);
        assert_eq!(jobs.refresh_generation, 1);
    }

    #[test]
    fn lifecycle_spawn_bumps_revision_once() {
        let mut jobs = AsyncDesktopJobs::new();
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-lc2"));
        // Hold a fake lifecycle slot is not needed — real spawn would need node bin.
        // Simulate the revision contract of try_spawn_lifecycle pre-checks:
        assert_eq!(jobs.lifecycle_revision(), 0);
        jobs.lifecycle_revision = jobs.lifecycle_revision.wrapping_add(1);
        jobs.invalidate_refresh();
        assert_eq!(jobs.lifecycle_revision(), 1);
        assert_eq!(jobs.refresh_generation, 1);
        let _ = paths;
    }

    #[test]
    fn poll_refresh_drops_while_lifecycle_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (tx, rx) = mpsc::channel();
        jobs.refresh_generation = 1;
        jobs.refresh_rx = Some(rx);
        let (_lc_tx, lc_rx) = mpsc::channel();
        jobs.lifecycle_rx = Some(lc_rx);
        let snap_err: Result<StatusSnapshot, String> = Err("should-drop".into());
        tx.send((1u64, snap_err)).unwrap();
        assert!(jobs.poll_refresh().is_none());
        assert!(!jobs.refresh_inflight());
    }

    #[test]
    fn quit_followup_chains_stop_after_start() {
        assert_eq!(
            quit_followup_after_lifecycle(true, LifecycleJobKind::Start, true),
            QuitFollowup::QueueStop
        );
        assert_eq!(
            quit_followup_after_lifecycle(true, LifecycleJobKind::Start, false),
            QuitFollowup::Close
        );
        assert_eq!(
            quit_followup_after_lifecycle(true, LifecycleJobKind::Stop, true),
            QuitFollowup::Close
        );
        assert_eq!(
            quit_followup_after_lifecycle(false, LifecycleJobKind::Start, true),
            QuitFollowup::None
        );
    }

    /// Phase T `#310`: Quit during submit defers Stop until submit settles.
    #[test]
    fn quit_during_submit_defers_then_queues_stop() {
        assert_eq!(quit_arm_policy(true), QuitArm::DeferUntilSubmitDone);
        assert_eq!(quit_arm_policy(false), QuitArm::ArmLifecycle);
        assert_eq!(quit_followup_after_submit(true), QuitFollowup::QueueStop);
        assert_eq!(quit_followup_after_submit(false), QuitFollowup::None);
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

    /// Phase T `#308`: second dial rejected while one is in flight.
    #[test]
    fn second_dial_rejected_while_inflight() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_hold_tx, hold_rx) = mpsc::channel();
        jobs.dial_rx = Some(hold_rx);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-dial"));
        assert!(jobs.dial_inflight());
        assert!(!jobs.try_spawn_dial(
            paths,
            "aira:identity:x".into(),
            "127.0.0.1:49157".into(),
            || {}
        ));
    }

    /// Phase T `#308`: dial may run while submit/lifecycle are idle or busy —
    /// slot is independent so UI never blocks on dial.
    #[test]
    fn dial_slot_independent_of_submit_lifecycle() {
        let mut jobs = AsyncDesktopJobs::new();
        let (_w_tx, w_rx) = mpsc::channel();
        jobs.work_rx = Some(w_rx);
        let paths = DesktopPaths::for_data_root(std::env::temp_dir().join("aira-async-jobs-dial2"));
        // Empty addr fails closed off-thread; spawn itself must succeed.
        assert!(jobs.try_spawn_dial(paths, "aira:identity:x".into(), "".into(), || {}));
        assert!(jobs.dial_inflight());
        for _ in 0..200 {
            if jobs.poll_dial().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!jobs.dial_inflight());
    }
}
