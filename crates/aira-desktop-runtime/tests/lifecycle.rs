//! Lifecycle integration tests (QUEUE #76). Requires built `aira-node`.

use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use aira_desktop_runtime::{
    load_or_create_settings, start, status, stop, write_settings, DesktopPaths, LifecycleStatus,
};

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

fn free_listen() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("127.0.0.1:{port}")
}

fn prepare_root() -> (tempfile::TempDir, DesktopPaths, String) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().expect("dirs");
    let listen = free_listen();
    let mut settings = load_or_create_settings(&paths).expect("settings");
    settings.http_listen = listen.clone();
    write_settings(&paths, &settings).expect("write settings");
    (tmp, paths, listen)
}

#[test]
fn start_idempotent_attach_and_stop() {
    let bin = node_bin();
    assert!(
        bin.is_file(),
        "build aira-node first (missing {})",
        bin.display()
    );

    let (_tmp, paths, listen) = prepare_root();
    let first = start(&paths, Some(bin.clone())).expect("start");
    assert_eq!(first.status, LifecycleStatus::Running);
    assert!(!first.attached);
    assert_eq!(first.listen, listen);

    let second = start(&paths, Some(bin)).expect("attach");
    assert!(second.attached);
    assert_eq!(second.status, LifecycleStatus::Running);
    assert_eq!(second.pid, first.pid);

    let (st, rec) = status(&paths).expect("status");
    assert_eq!(st, LifecycleStatus::Running);
    assert!(rec.is_some());

    let stopped = stop(&paths).expect("stop");
    assert_eq!(stopped, LifecycleStatus::Stopped);
    std::thread::sleep(Duration::from_millis(200));
    let (st2, _) = status(&paths).expect("status after stop");
    assert_eq!(st2, LifecycleStatus::Stopped);
}

#[test]
fn stale_pid_recovered_on_start() {
    let bin = node_bin();
    assert!(bin.is_file(), "build aira-node first");
    let (_tmp, paths, _listen) = prepare_root();
    let settings = load_or_create_settings(&paths).unwrap();
    let dead_pid = 4_294_967_294u32;
    std::fs::write(
        paths.pid_file(),
        serde_json::json!({
            "pid": dead_pid,
            "instance_id": settings.instance_id,
            "root": paths.data_root.display().to_string(),
            "listen": settings.http_listen,
            "node_bin": "aira-node"
        })
        .to_string(),
    )
    .unwrap();

    let outcome = start(&paths, Some(bin)).expect("start after stale pid");
    assert!(!outcome.attached);
    assert_eq!(outcome.status, LifecycleStatus::Running);
    let _ = stop(&paths);
}

#[test]
fn port_conflict_fails_closed() {
    let bin = node_bin();
    assert!(bin.is_file(), "build aira-node first");
    let (_tmp, paths, listen) = prepare_root();
    let _holder = TcpListener::bind(&listen).expect("hold port");
    let err = start(&paths, Some(bin)).expect_err("must fail on foreign port");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("occupied") || msg.contains("listen"),
        "unexpected err: {msg}"
    );
}
