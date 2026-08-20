//! P1 dual-process lifecycle (QUEUE #82). Requires built `aira-node` + `aira`.

use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use aira_desktop_runtime::{
    load_or_create_settings, start, status, stop, write_settings, DesktopPaths, LifecycleStatus,
    NetworkProfile,
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

/// Ensure `aira` + `aira-node` exist (CI may run this crate's tests before `aira-cli` bins).
fn ensure_bins() -> (PathBuf, PathBuf) {
    let mut node = node_bin();
    let mut aira = aira_bin();
    if node.is_file() && aira.is_file() {
        return (node, aira);
    }
    let status = std::process::Command::new(env!("CARGO"))
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
    assert!(
        node.is_file(),
        "missing aira-node after build ({})",
        node.display()
    );
    assert!(
        aira.is_file(),
        "missing aira after build ({})",
        aira.display()
    );
    (node, aira)
}

fn free_listen() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("127.0.0.1:{port}")
}

#[test]
fn p1_starts_http_and_peer_then_stop() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let http = free_listen();
    let peer = free_listen();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = http.clone();
    settings.network_profile = NetworkProfile::P1;
    settings.peer_listen = Some(peer.clone());
    write_settings(&paths, &settings).unwrap();

    let outcome = start(&paths, Some(node)).expect("start P1");
    assert_eq!(outcome.status, LifecycleStatus::Running);
    assert_eq!(outcome.listen, http);
    assert_eq!(outcome.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(outcome.peer_pid.is_some());
    assert!(!outcome.peer_attached);

    let (st, rec) = status(&paths).unwrap();
    assert_eq!(st, LifecycleStatus::Running);
    let rec = rec.expect("record");
    assert_eq!(rec.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(rec.peer_pid.is_some());

    // Attach should keep both.
    let again = start(&paths, Some(node_bin())).expect("attach");
    assert!(again.attached);
    assert!(again.peer_pid.is_some());

    let stopped = stop(&paths).unwrap();
    assert_eq!(stopped, LifecycleStatus::Stopped);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!paths.peer_pid_file().is_file());
    let (st2, _) = status(&paths).unwrap();
    assert_eq!(st2, LifecycleStatus::Stopped);
}

#[test]
fn p0_does_not_start_peer() {
    let (node, _) = ensure_bins();
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let http = free_listen();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = http;
    settings.network_profile = NetworkProfile::P0;
    write_settings(&paths, &settings).unwrap();

    let outcome = start(&paths, Some(node)).expect("start P0");
    assert!(outcome.peer_pid.is_none());
    assert!(outcome.peer_listen.is_none());
    let _ = stop(&paths);
}
