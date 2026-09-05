//! Settings P0/P1 load/save (QUEUE #81).

use aira_desktop_runtime::{
    load_or_create_settings, write_settings, DesktopPaths, NetworkProfile, DEFAULT_PEER_LISTEN,
};

#[test]
fn p1_roundtrip_persists_default_peer_listen() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    assert_eq!(s.network_profile, NetworkProfile::P0);
    s.network_profile = NetworkProfile::P1;
    s.peer_listen = None;
    write_settings(&paths, &s).unwrap();
    let loaded = load_or_create_settings(&paths).unwrap();
    assert_eq!(loaded.network_profile, NetworkProfile::P1);
    assert_eq!(loaded.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
}

#[test]
fn p5_write_rejected_from_p1_suite() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P5;
    s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
    let err = write_settings(&paths, &s).unwrap_err().to_string();
    assert!(err.contains("P0|P1|P2|P3|P4"), "{err}");
}

#[test]
fn p1_custom_peer_listen_kept() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P1;
    s.peer_listen = Some("127.0.0.1:49171".into());
    write_settings(&paths, &s).unwrap();
    let loaded = load_or_create_settings(&paths).unwrap();
    assert_eq!(loaded.peer_listen.as_deref(), Some("127.0.0.1:49171"));
}
