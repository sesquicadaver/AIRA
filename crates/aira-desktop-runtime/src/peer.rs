//! Supervise `aira peer listen --recv` for Desktop P1 (QUEUE #82).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::health::port_in_use;
use crate::paths::DesktopPaths;
use crate::process::{pid_alive, signal_kill, signal_term};
use crate::settings::{DesktopSettings, NetworkProfile};

const PEER_READY_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PeerPidRecord {
    pub pid: u32,
    pub instance_id: String,
    pub root: String,
    pub listen: String,
    pub aira_bin: String,
}

#[derive(Debug, Clone)]
pub struct PeerPidRecordView {
    pub pid: u32,
    pub instance_id: String,
    pub root: String,
    pub listen: String,
}

/// Ensure peer is running when `network_profile=P1`; no-op on P0.
pub(crate) fn ensure_peer(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
    aira_bin: Option<PathBuf>,
) -> Result<(Option<u32>, Option<String>, bool)> {
    if settings.network_profile != NetworkProfile::P1 {
        // P0: do not leave a stray peer from a previous P1 session.
        let _ = stop_peer(paths);
        return Ok((None, None, false));
    }
    let listen = settings
        .peer_listen
        .as_deref()
        .context("P1 requires peer_listen (normalize settings first)")?
        .to_string();
    require_loopback_bind(&listen)?;

    if let Some(rec) = try_attach_peer(paths, settings, &listen)? {
        return Ok((Some(rec.pid), Some(listen), true));
    }

    if port_in_use(&listen, Duration::from_millis(200)) {
        bail!(
            "peer_listen {listen} is occupied by another process (not a compatible AIRA peer for {})",
            settings.instance_id
        );
    }

    let aira_bin = resolve_aira_bin(aira_bin)?;
    acquire_peer_lock(paths)?;
    let stdout = File::create(paths.log_dir.join("aira-peer.stdout.log"))?;
    let stderr = File::create(paths.log_dir.join("aira-peer.stderr.log"))?;

    let mut child = Command::new(&aira_bin)
        .arg("--root")
        .arg(&paths.data_root)
        .arg("peer")
        .arg("listen")
        .arg("--bind")
        .arg(&listen)
        .arg("--recv")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| format!("spawn {} peer listen", aira_bin.display()))?;

    let pid = child.id();
    write_peer_pid_record(
        paths,
        &PeerPidRecord {
            pid,
            instance_id: settings.instance_id.clone(),
            root: paths.data_root.display().to_string(),
            listen: listen.clone(),
            aira_bin: aira_bin.display().to_string(),
        },
    )?;

    match wait_peer_bound(&listen, PEER_READY_TIMEOUT) {
        Ok(()) => Ok((Some(pid), Some(listen), false)),
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            clear_peer_runtime_files(paths);
            bail!("peer listen started but not bound: {e}");
        }
    }
}

pub(crate) fn stop_peer(paths: &DesktopPaths) -> Result<()> {
    let Some(rec) = read_peer_pid_record(paths)? else {
        clear_peer_runtime_files(paths);
        return Ok(());
    };
    if !pid_alive(rec.pid) {
        clear_peer_runtime_files(paths);
        return Ok(());
    }
    signal_term(rec.pid)?;
    let start = Instant::now();
    while start.elapsed() < STOP_GRACE {
        if !pid_alive(rec.pid) {
            clear_peer_runtime_files(paths);
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    signal_kill(rec.pid)?;
    clear_peer_runtime_files(paths);
    Ok(())
}

pub(crate) fn peer_status(
    paths: &DesktopPaths,
    settings: Option<&DesktopSettings>,
) -> Result<Option<PeerPidRecordView>> {
    let Some(rec) = read_peer_pid_record(paths)? else {
        return Ok(None);
    };
    let view = PeerPidRecordView {
        pid: rec.pid,
        instance_id: rec.instance_id.clone(),
        root: rec.root.clone(),
        listen: rec.listen.clone(),
    };
    if !pid_alive(rec.pid) {
        clear_peer_runtime_files(paths);
        return Ok(None);
    }
    if let Some(s) = settings {
        if s.network_profile != NetworkProfile::P1 {
            // Unexpected peer while on P0 — still report live pid.
            return Ok(Some(view));
        }
    }
    Ok(Some(view))
}

fn try_attach_peer(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
    listen: &str,
) -> Result<Option<PeerPidRecord>> {
    if let Some(rec) = read_peer_pid_record(paths)? {
        if pid_alive(rec.pid)
            && rec.instance_id == settings.instance_id
            && Path::new(&rec.root) == paths.data_root.as_path()
            && rec.listen == listen
            && port_in_use(&rec.listen, Duration::from_millis(300))
        {
            return Ok(Some(rec));
        }
        if pid_alive(rec.pid) {
            bail!(
                "another AIRA peer is recorded (pid={}, instance={}, root={})",
                rec.pid,
                rec.instance_id,
                rec.root
            );
        }
        clear_peer_runtime_files(paths);
    }
    Ok(None)
}

fn wait_peer_bound(listen: &str, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if port_in_use(listen, Duration::from_millis(100)) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    bail!("timeout waiting for peer bind on {listen}");
}

fn require_loopback_bind(bind: &str) -> Result<()> {
    if bind.starts_with("127.0.0.1:")
        || bind.starts_with("[::1]:")
        || bind.starts_with("localhost:")
    {
        return Ok(());
    }
    bail!(
        "Desktop P1 peer_listen must be loopback (got `{bind}`); non-loopback needs peer --explicit (Out of #82)"
    );
}

pub(crate) fn resolve_aira_bin(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    if let Ok(p) = std::env::var("AIRA_BIN") {
        return Ok(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("aira");
            if cand.is_file() {
                return Ok(cand);
            }
        }
    }
    if let Ok(node) = std::env::var("AIRA_NODE_BIN") {
        let p = PathBuf::from(node);
        if let Some(dir) = p.parent() {
            let cand = dir.join("aira");
            if cand.is_file() {
                return Ok(cand);
            }
        }
    }
    Ok(PathBuf::from("aira"))
}

fn acquire_peer_lock(paths: &DesktopPaths) -> Result<()> {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(paths.peer_lock_file())
    {
        Ok(mut f) => {
            writeln!(f, "{}", std::process::id())?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_peer_pid_record(paths)?.is_none() {
                let _ = fs::remove_file(paths.peer_lock_file());
                return acquire_peer_lock(paths);
            }
            bail!(
                "desktop peer lock held: {}",
                paths.peer_lock_file().display()
            );
        }
        Err(e) => Err(e).context("create peer lock"),
    }
}

fn write_peer_pid_record(paths: &DesktopPaths, rec: &PeerPidRecord) -> Result<()> {
    let text = serde_json::to_string_pretty(rec)?;
    fs::write(paths.peer_pid_file(), format!("{text}\n"))?;
    Ok(())
}

fn read_peer_pid_record(paths: &DesktopPaths) -> Result<Option<PeerPidRecord>> {
    if !paths.peer_pid_file().is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(paths.peer_pid_file())?;
    let rec: PeerPidRecord = serde_json::from_str(&text)
        .with_context(|| format!("parse {}", paths.peer_pid_file().display()))?;
    Ok(Some(rec))
}

fn clear_peer_runtime_files(paths: &DesktopPaths) {
    let _ = fs::remove_file(paths.peer_pid_file());
    let _ = fs::remove_file(paths.peer_lock_file());
}
