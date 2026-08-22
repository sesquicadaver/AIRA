//! P2 peer lifecycle with `--dht --apply-book` (QUEUE #95).

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use aira_desktop_runtime::{
    export_invite_file, import_invite_file, load_or_create_settings, start, stop, write_settings,
    DesktopPaths, LifecycleStatus, NetworkProfile,
};
use aira_peer::AddressBook;

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

#[test]
fn p2_starts_http_and_peer_with_dht_flags() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let http = free_listen();
    let peer = free_listen();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = http.clone();
    settings.network_profile = NetworkProfile::P2;
    settings.peer_listen = Some(peer.clone());
    write_settings(&paths, &settings).unwrap();

    let outcome = start(&paths, Some(node)).expect("start P2");
    assert_eq!(outcome.status, LifecycleStatus::Running);
    assert_eq!(outcome.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(outcome.peer_pid.is_some());

    let pid_text = std::fs::read_to_string(paths.peer_pid_file()).unwrap();
    assert!(
        pid_text.contains("\"network_profile\": \"P2\""),
        "{pid_text}"
    );

    let _ = stop(&paths);
}

#[test]
fn p2_dht_apply_book_smoke() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
    let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
    alice.ensure_dirs().unwrap();
    bob.ensure_dirs().unwrap();

    let alice_http = free_listen();
    let alice_peer = free_listen();
    let bob_http = free_listen();
    let bob_peer = free_listen();

    let mut alice_settings = load_or_create_settings(&alice).unwrap();
    alice_settings.http_listen = alice_http;
    alice_settings.network_profile = NetworkProfile::P2;
    alice_settings.peer_listen = Some(alice_peer.clone());
    write_settings(&alice, &alice_settings).unwrap();

    let mut bob_settings = load_or_create_settings(&bob).unwrap();
    bob_settings.http_listen = bob_http;
    bob_settings.network_profile = NetworkProfile::P2;
    bob_settings.peer_listen = Some(bob_peer.clone());
    write_settings(&bob, &bob_settings).unwrap();

    let alice_out = start(&alice, Some(node.clone())).expect("alice start");
    let bob_out = start(&bob, Some(node.clone())).expect("bob start");
    assert!(alice_out.peer_pid.is_some());
    assert!(bob_out.peer_pid.is_some());

    let alice_invite_path = tmp.path().join("alice.invite.json");
    let bob_invite_path = tmp.path().join("bob.invite.json");
    let alice_invite =
        export_invite_file(&alice, &alice_invite_path, Some(alice_peer.clone())).unwrap();
    let _bob_invite = export_invite_file(&bob, &bob_invite_path, Some(bob_peer.clone())).unwrap();

    import_invite_file(&bob, &alice_invite_path).expect("bob imports alice");
    import_invite_file(&alice, &bob_invite_path).expect("alice imports bob");

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

    std::thread::sleep(Duration::from_millis(800));

    let book = AddressBook::load(&bob.data_root).unwrap();
    let ep = book
        .peers
        .iter()
        .find(|p| p.identity_id == alice_invite.identity_ref)
        .expect("bob book missing alice after apply-book");
    assert_eq!(ep.addr, alice_peer);

    let _ = stop(&alice);
    let _ = stop(&bob);
}
