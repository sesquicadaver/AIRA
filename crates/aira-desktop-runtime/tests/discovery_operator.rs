//! P6 discovery operators (QUEUE #105).

use aira_desktop_runtime::{run_discv_announce, run_discv_find, run_stun_query, DesktopPaths};

#[test]
fn stun_query_requires_explicit_server() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let err = run_stun_query(&paths, "").unwrap_err().to_string();
    assert!(err.contains("STUN server required"), "{err}");
}

#[test]
fn discv_announce_requires_explicit_addr() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let err = run_discv_announce(&paths, "127.0.0.1:9", "")
        .unwrap_err()
        .to_string();
    assert!(err.contains("explicit --addr"), "{err}");
}

#[test]
fn discv_announce_smoke_sends() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    // No listener required: operator shortcut only proves signed send path.
    let msg = run_discv_announce(&paths, "127.0.0.1:9", "127.0.0.1:19099").unwrap();
    assert!(msg.contains("127.0.0.1:19099"));
}

#[test]
fn discv_find_requires_key_ref() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let err = run_discv_find(&paths, "", None, 8).unwrap_err().to_string();
    assert!(err.contains("key_ref"), "{err}");
}

#[test]
fn discv_find_smoke_after_bootstrap() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let report = run_discv_find(&paths, "aira:identity:find-target-p6", None, 8).unwrap();
    assert!(report.hops >= 1);
}
