//! Supervise `aira-node --http` (PID/lock, attach, stop).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::bootstrap::{ensure_bootstrap, read_http_token};
use crate::health::{health_ok, port_in_use, wait_healthy};
use crate::paths::DesktopPaths;
use crate::settings::{load_or_create_settings, resolve_token_path, DesktopSettings};

const HEALTH_TIMEOUT: Duration = Duration::from_secs(15);
const STOP_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    Stopped,
    Starting,
    Running,
    Unhealthy,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PidRecord {
    pid: u32,
    instance_id: String,
    root: String,
    listen: String,
    node_bin: String,
}

#[derive(Debug, Clone)]
pub struct StartOutcome {
    pub status: LifecycleStatus,
    pub attached: bool,
    pub pid: Option<u32>,
    pub listen: String,
    pub instance_id: String,
    pub data_root: PathBuf,
}

/// Start or attach to a compatible Desktop node.
pub fn start(paths: &DesktopPaths, node_bin: Option<PathBuf>) -> Result<StartOutcome> {
    paths.ensure_dirs()?;
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    let listen = settings.http_listen.clone();
    let instance_id = settings.instance_id.clone();

    if let Some(outcome) = try_attach(paths, &settings)? {
        return Ok(outcome);
    }

    if port_in_use(&listen, Duration::from_millis(200)) {
        bail!(
            "listen {listen} is occupied by another process (not a compatible AIRA instance for {})",
            settings.instance_id
        );
    }

    let node_bin = resolve_node_bin(node_bin)?;
    let token_path = resolve_token_path(paths, &settings)?;
    let token = read_http_token(&token_path)?;

    acquire_lock(paths)?;
    let stdout = File::create(paths.log_dir.join("aira-node.stdout.log"))?;
    let stderr = File::create(paths.log_dir.join("aira-node.stderr.log"))?;

    let mut child = Command::new(&node_bin)
        .arg("--root")
        .arg(&paths.data_root)
        .arg("--http")
        .arg("--listen")
        .arg(&listen)
        .arg("--http-token")
        .arg(&token)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| format!("spawn {}", node_bin.display()))?;

    let pid = child.id();
    write_pid_record(
        paths,
        &PidRecord {
            pid,
            instance_id: instance_id.clone(),
            root: paths.data_root.display().to_string(),
            listen: listen.clone(),
            node_bin: node_bin.display().to_string(),
        },
    )?;

    match wait_healthy(&listen, HEALTH_TIMEOUT) {
        Ok(()) => Ok(StartOutcome {
            status: LifecycleStatus::Running,
            attached: false,
            pid: Some(pid),
            listen,
            instance_id,
            data_root: paths.data_root.clone(),
        }),
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            clear_runtime_files(paths);
            bail!("node started but unhealthy: {e}");
        }
    }
}

/// Stop supervised node (SIGTERM then kill).
pub fn stop(paths: &DesktopPaths) -> Result<LifecycleStatus> {
    let Some(rec) = read_pid_record(paths)? else {
        clear_runtime_files(paths);
        return Ok(LifecycleStatus::Stopped);
    };
    if !pid_alive(rec.pid) {
        clear_runtime_files(paths);
        return Ok(LifecycleStatus::Stopped);
    }
    signal_term(rec.pid)?;
    let start = std::time::Instant::now();
    while start.elapsed() < STOP_GRACE {
        if !pid_alive(rec.pid) {
            clear_runtime_files(paths);
            return Ok(LifecycleStatus::Stopped);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    signal_kill(rec.pid)?;
    clear_runtime_files(paths);
    Ok(LifecycleStatus::Stopped)
}

/// Report lifecycle status.
pub fn status(paths: &DesktopPaths) -> Result<(LifecycleStatus, Option<PidRecordView>)> {
    let settings = if paths.settings_file.is_file() {
        Some(load_or_create_settings(paths)?)
    } else {
        None
    };
    let listen = settings
        .as_ref()
        .map(|s| s.http_listen.clone())
        .unwrap_or_else(|| "127.0.0.1:8787".into());

    let Some(rec) = read_pid_record(paths)? else {
        if port_in_use(&listen, Duration::from_millis(100))
            && health_ok(&listen, Duration::from_millis(200)).unwrap_or(false)
        {
            return Ok((LifecycleStatus::Unhealthy, None));
        }
        return Ok((LifecycleStatus::Stopped, None));
    };

    let view = PidRecordView {
        pid: rec.pid,
        instance_id: rec.instance_id.clone(),
        root: rec.root.clone(),
        listen: rec.listen.clone(),
    };

    if !pid_alive(rec.pid) {
        clear_runtime_files(paths);
        return Ok((LifecycleStatus::Stopped, Some(view)));
    }
    if health_ok(&rec.listen, Duration::from_millis(300)).unwrap_or(false) {
        Ok((LifecycleStatus::Running, Some(view)))
    } else {
        Ok((LifecycleStatus::Unhealthy, Some(view)))
    }
}

#[derive(Debug, Clone)]
pub struct PidRecordView {
    pub pid: u32,
    pub instance_id: String,
    pub root: String,
    pub listen: String,
}

fn try_attach(paths: &DesktopPaths, settings: &DesktopSettings) -> Result<Option<StartOutcome>> {
    if let Some(rec) = read_pid_record(paths)? {
        if pid_alive(rec.pid)
            && rec.instance_id == settings.instance_id
            && Path::new(&rec.root) == paths.data_root.as_path()
            && rec.listen == settings.http_listen
        {
            if health_ok(&rec.listen, Duration::from_millis(500)).unwrap_or(false) {
                return Ok(Some(StartOutcome {
                    status: LifecycleStatus::Running,
                    attached: true,
                    pid: Some(rec.pid),
                    listen: rec.listen,
                    instance_id: rec.instance_id,
                    data_root: paths.data_root.clone(),
                }));
            }
            // Stale unhealthy — stop and continue to fresh start.
            let _ = signal_kill(rec.pid);
            clear_runtime_files(paths);
            return Ok(None);
        }
        if pid_alive(rec.pid) {
            bail!(
                "another AIRA desktop instance is recorded (pid={}, instance={}, root={})",
                rec.pid,
                rec.instance_id,
                rec.root
            );
        }
        // Stale PID.
        clear_runtime_files(paths);
    }

    // Port held by compatible health endpoint without our pidfile (attach opportunistic).
    if port_in_use(&settings.http_listen, Duration::from_millis(200))
        && health_ok(&settings.http_listen, Duration::from_millis(400)).unwrap_or(false)
    {
        // Foreign healthy listener without pidfile — fail-closed (cannot prove instance_id).
        bail!(
            "listen {} already serves /health but no matching pidfile for instance {}",
            settings.http_listen,
            settings.instance_id
        );
    }
    Ok(None)
}

fn resolve_node_bin(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    if let Ok(p) = std::env::var("AIRA_NODE_BIN") {
        return Ok(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join("aira-node");
            if cand.is_file() {
                return Ok(cand);
            }
        }
    }
    Ok(PathBuf::from("aira-node"))
}

fn acquire_lock(paths: &DesktopPaths) -> Result<()> {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(paths.lock_file())
    {
        Ok(mut f) => {
            writeln!(f, "{}", std::process::id())?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Stale lock without live pidfile — clear.
            if read_pid_record(paths)?.is_none() {
                let _ = fs::remove_file(paths.lock_file());
                return acquire_lock(paths);
            }
            bail!("desktop lock held: {}", paths.lock_file().display());
        }
        Err(e) => Err(e).context("create lock"),
    }
}

fn write_pid_record(paths: &DesktopPaths, rec: &PidRecord) -> Result<()> {
    let text = serde_json::to_string_pretty(rec)?;
    fs::write(paths.pid_file(), format!("{text}\n"))?;
    Ok(())
}

fn read_pid_record(paths: &DesktopPaths) -> Result<Option<PidRecord>> {
    if !paths.pid_file().is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(paths.pid_file())?;
    let rec: PidRecord = serde_json::from_str(&text)
        .with_context(|| format!("parse {}", paths.pid_file().display()))?;
    Ok(Some(rec))
}

fn clear_runtime_files(paths: &DesktopPaths) {
    let _ = fs::remove_file(paths.pid_file());
    let _ = fs::remove_file(paths.lock_file());
}

fn pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn signal_term(pid: u32) -> Result<()> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("kill -TERM")?;
    if !status.success() && pid_alive(pid) {
        bail!("SIGTERM failed for pid {pid}");
    }
    Ok(())
}

fn signal_kill(pid: u32) -> Result<()> {
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(())
}
