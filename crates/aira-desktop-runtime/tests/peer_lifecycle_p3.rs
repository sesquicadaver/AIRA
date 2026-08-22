//! P3 relay peer lifecycle (QUEUE #98).

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use aira_desktop_runtime::{
    load_or_create_settings, start, stop, write_settings, DesktopPaths, LifecycleStatus,
    NetworkProfile, DEFAULT_RELAY_TTL_DAYS,
};

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

fn relay_hub_path(root: &Path) -> PathBuf {
    root.join("peers").join("relay_hub.json")
}

#[test]
fn p3_starts_http_and_relay_peer() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let http = free_listen();
    let peer = free_listen();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = http.clone();
    settings.network_profile = NetworkProfile::P3;
    settings.peer_listen = Some(peer.clone());
    settings.relay_ttl_days = Some(14);
    write_settings(&paths, &settings).unwrap();

    let outcome = start(&paths, Some(node)).expect("start P3");
    assert_eq!(outcome.status, LifecycleStatus::Running);
    assert_eq!(outcome.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(outcome.peer_pid.is_some());

    let pid_text = std::fs::read_to_string(paths.peer_pid_file()).unwrap();
    assert!(
        pid_text.contains("\"network_profile\": \"P3\""),
        "{pid_text}"
    );
    assert!(pid_text.contains("\"relay_ttl_days\": 14"), "{pid_text}");

    std::thread::sleep(Duration::from_millis(300));
    assert!(
        relay_hub_path(&paths.data_root).is_file(),
        "relay_hub.json missing"
    );

    let _ = stop(&paths);
}

#[test]
fn p3_relay_registry_survives_restart() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = free_listen();
    settings.network_profile = NetworkProfile::P3;
    settings.peer_listen = Some(free_listen());
    settings.relay_ttl_days = Some(DEFAULT_RELAY_TTL_DAYS);
    write_settings(&paths, &settings).unwrap();

    start(&paths, Some(node.clone())).expect("first start");
    std::thread::sleep(Duration::from_millis(300));
    let hub = relay_hub_path(&paths.data_root);
    assert!(hub.is_file());
    let before = std::fs::read_to_string(&hub).unwrap();
    stop(&paths).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let again = start(&paths, Some(node)).expect("second start");
    assert!(again.peer_attached || again.peer_pid.is_some());
    std::thread::sleep(Duration::from_millis(200));
    let after = std::fs::read_to_string(&hub).unwrap();
    assert_eq!(before, after, "registry should reload from disk");

    let _ = stop(&paths);
}
