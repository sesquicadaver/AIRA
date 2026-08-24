//! P2 peer lifecycle with `--dht --apply-book` (QUEUE #95).
//! Stabilized for CI parallel workspace tests (QUEUE #118): serial execution + port retry.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use aira_desktop_runtime::{
    export_invite_file, import_invite_file, load_or_create_settings, start, stop, write_settings,
    DesktopPaths, LifecycleStatus, NetworkProfile,
};
use aira_peer::AddressBook;
use serial_test::serial;

const PORT_RETRY_ATTEMPTS: usize = 8;
const STOP_SETTLE: Duration = Duration::from_millis(300);

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

fn stop_and_settle(paths: &DesktopPaths) {
    let _ = stop(paths);
    std::thread::sleep(STOP_SETTLE);
}

fn start_err_port_conflict(err: &anyhow::Error) -> bool {
    let msg = format!("{err:#}").to_lowercase();
    msg.contains("occupied") || msg.contains("listen") || msg.contains("bind")
}

/// Start P2 stack with ephemeral loopback ports; retry when another test grabbed the port.
fn start_p2_with_ports(
    paths: &DesktopPaths,
    node: &Path,
) -> (aira_desktop_runtime::StartOutcome, String, String) {
    for attempt in 0..PORT_RETRY_ATTEMPTS {
        let http = free_listen();
        let peer = free_listen();
        let mut settings = load_or_create_settings(paths).unwrap();
        settings.http_listen = http.clone();
        settings.network_profile = NetworkProfile::P2;
        settings.peer_listen = Some(peer.clone());
        write_settings(paths, &settings).unwrap();

        match start(paths, Some(node.to_path_buf())) {
            Ok(outcome) => return (outcome, http, peer),
            Err(e) if start_err_port_conflict(&e) && attempt + 1 < PORT_RETRY_ATTEMPTS => {
                stop_and_settle(paths);
                continue;
            }
            Err(e) => panic!("start P2 failed: {e:#}"),
        }
    }
    panic!("start P2 failed after {PORT_RETRY_ATTEMPTS} port retries");
}

fn wait_book_addr(data_root: &Path, identity_id: &str, expected_addr: &str, timeout: Duration) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let book = AddressBook::load(data_root).unwrap();
        if let Some(ep) = book.peers.iter().find(|p| p.identity_id == identity_id) {
            if ep.addr == expected_addr {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let book = AddressBook::load(data_root).unwrap();
    panic!(
        "address book missing {identity_id} @ {expected_addr}; peers={:?}",
        book.peers
    );
}

#[test]
#[serial(desktop_peer_integration)]
fn p2_starts_http_and_peer_with_dht_flags() {
    let (node, _aira) = ensure_bins();

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();

    let (outcome, _http, peer) = start_p2_with_ports(&paths, &node);
    assert_eq!(outcome.status, LifecycleStatus::Running);
    assert_eq!(outcome.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(outcome.peer_pid.is_some());

    let pid_text = std::fs::read_to_string(paths.peer_pid_file()).unwrap();
    assert!(
        pid_text.contains("\"network_profile\": \"P2\""),
        "{pid_text}"
    );

    stop_and_settle(&paths);
}

#[test]
#[serial(desktop_peer_integration)]
fn p2_dht_apply_book_smoke() {
    let (node, aira) = ensure_bins();

    let tmp = tempfile::tempdir().unwrap();
    let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
    let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
    alice.ensure_dirs().unwrap();
    bob.ensure_dirs().unwrap();

    let (alice_out, _alice_http, alice_peer) = start_p2_with_ports(&alice, &node);
    let (bob_out, _bob_http, bob_peer) = start_p2_with_ports(&bob, &node);
    assert!(alice_out.peer_pid.is_some());
    assert!(bob_out.peer_pid.is_some());

    let alice_invite_path = tmp.path().join("alice.invite.json");
    let bob_invite_path = tmp.path().join("bob.invite.json");
    let alice_invite =
        export_invite_file(&alice, &alice_invite_path, Some(alice_peer.clone())).unwrap();
    let _bob_invite = export_invite_file(&bob, &bob_invite_path, Some(bob_peer.clone())).unwrap();

    import_invite_file(&bob, &alice_invite_path).expect("bob imports alice");
    import_invite_file(&alice, &bob_invite_path).expect("alice imports bob");

    wait_book_addr(
        &bob.data_root,
        &alice_invite.identity_ref,
        &alice_peer,
        Duration::from_secs(2),
    );

    let status = Command::new(&aira)
        .arg("--root")
        .arg(&alice.data_root)
        .arg("peer")
        .arg("dht")
        .arg("announce")
        .arg("--addr")
        .arg(&alice_peer)
        .status()
        .expect("dht announce");
    assert!(status.success(), "peer dht announce failed");

    stop_and_settle(&alice);
    stop_and_settle(&bob);
}
