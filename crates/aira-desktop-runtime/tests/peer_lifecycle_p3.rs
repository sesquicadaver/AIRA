//! P3 relay peer lifecycle (QUEUE #98).
//! Stabilized for CI parallel workspace tests (QUEUE #131): serial execution + port retry.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use aira_desktop_runtime::{
    load_or_create_settings, start, stop, write_settings, DesktopPaths, LifecycleStatus,
    NetworkProfile, DEFAULT_RELAY_TTL_DAYS,
};
use serial_test::serial;

const PORT_RETRY_ATTEMPTS: usize = 8;
const STOP_SETTLE: Duration = Duration::from_millis(300);
const RELAY_HUB_WAIT: Duration = Duration::from_secs(2);

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn node_bin() -> PathBuf {
    if let Ok(p) = std::env::var("AIRA_NODE_BIN") {
        return PathBuf::from(p);
    }
    for rel in [
        "../../target/debug/aira-node",
        "../../target/release/aira-node",
    ] {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push(rel);
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from("aira-node")
}

fn aira_bin() -> PathBuf {
    if let Ok(p) = std::env::var("AIRA_BIN") {
        return PathBuf::from(p);
    }
    let node = node_bin();
    if let Some(dir) = node.parent() {
        let cand = dir.join("aira");
        if cand.is_file() {
            return cand;
        }
    }
    for profile in ["debug", "release"] {
        let cand = workspace_root().join("target").join(profile).join("aira");
        if cand.is_file() {
            return cand;
        }
    }
    PathBuf::from("aira")
}

fn ensure_bins() -> (PathBuf, PathBuf) {
    let mut node = node_bin();
    let mut aira = aira_bin();
    if node.is_file() && aira.is_file() {
        return (node, aira);
    }
    let status = Command::new(env!("CARGO"))
        .args(["build", "-p", "aira-cli", "-p", "aira-node", "--quiet"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build aira-cli/aira-node");
    assert!(
        status.success(),
        "cargo build -p aira-cli -p aira-node failed"
    );
    node = node_bin();
    aira = aira_bin();
    assert!(node.is_file(), "missing aira-node ({})", node.display());
    assert!(aira.is_file(), "missing aira ({})", aira.display());
    (node, aira)
}

fn free_listen() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("127.0.0.1:{port}")
}

fn free_peer_listen() -> String {
    aira_peer::format_available_loopback_tcp_bind().expect("prime peer bind")
}

fn relay_hub_path(root: &Path) -> PathBuf {
    root.join("peers").join("relay_hub.json")
}

fn stop_and_settle(paths: &DesktopPaths) {
    let _ = stop(paths);
    std::thread::sleep(STOP_SETTLE);
}

fn start_err_port_conflict(err: &anyhow::Error) -> bool {
    let msg = format!("{err:#}").to_lowercase();
    msg.contains("occupied") || msg.contains("listen") || msg.contains("bind")
}

/// Start P3 stack with ephemeral loopback ports; retry when another test grabbed the port.
fn start_p3_with_ports(
    paths: &DesktopPaths,
    node: &Path,
    relay_ttl_days: Option<u32>,
) -> (aira_desktop_runtime::StartOutcome, String, String) {
    for attempt in 0..PORT_RETRY_ATTEMPTS {
        let http = free_listen();
        let peer = free_peer_listen();
        let mut settings = load_or_create_settings(paths).unwrap();
        settings.http_listen = http.clone();
        settings.network_profile = NetworkProfile::P3;
        settings.peer_listen = Some(peer.clone());
        settings.relay_ttl_days = relay_ttl_days;
        write_settings(paths, &settings).unwrap();

        match start(paths, Some(node.to_path_buf())) {
            Ok(outcome) => return (outcome, http, peer),
            Err(e) if start_err_port_conflict(&e) && attempt + 1 < PORT_RETRY_ATTEMPTS => {
                stop_and_settle(paths);
                continue;
            }
            Err(e) => panic!("start P3 failed: {e:#}"),
        }
    }
    panic!("start P3 failed after {PORT_RETRY_ATTEMPTS} port retries");
}

fn wait_relay_hub(data_root: &Path, timeout: Duration) {
    let hub = relay_hub_path(data_root);
    let start = Instant::now();
    while start.elapsed() < timeout {
        if hub.is_file() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("relay_hub.json missing after {:?}", timeout);
}

#[test]
#[serial(desktop_peer_integration)]
fn p3_starts_http_and_relay_peer() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();

    let (outcome, _http, peer) = start_p3_with_ports(&paths, &node, Some(14));
    assert_eq!(outcome.status, LifecycleStatus::Running);
    assert_eq!(outcome.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(outcome.peer_pid.is_some());

    let pid_text = std::fs::read_to_string(paths.peer_pid_file()).unwrap();
    assert!(
        pid_text.contains("\"network_profile\": \"P3\""),
        "{pid_text}"
    );
    assert!(pid_text.contains("\"relay_ttl_days\": 14"), "{pid_text}");

    wait_relay_hub(&paths.data_root, RELAY_HUB_WAIT);

    stop_and_settle(&paths);
}

#[test]
#[serial(desktop_peer_integration)]
fn p3_relay_registry_survives_restart() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();

    start_p3_with_ports(&paths, &node, Some(DEFAULT_RELAY_TTL_DAYS));
    wait_relay_hub(&paths.data_root, RELAY_HUB_WAIT);
    let hub = relay_hub_path(&paths.data_root);
    let before = std::fs::read_to_string(&hub).unwrap();
    stop_and_settle(&paths);

    let again = start(&paths, Some(node)).expect("second start");
    assert!(again.peer_attached || again.peer_pid.is_some());
    wait_relay_hub(&paths.data_root, RELAY_HUB_WAIT);
    let after = std::fs::read_to_string(&hub).unwrap();
    assert_eq!(before, after, "registry should reload from disk");

    stop_and_settle(&paths);
}
