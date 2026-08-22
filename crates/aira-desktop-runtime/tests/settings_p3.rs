//! Settings P3 + relay_ttl_days (QUEUE #97).

use aira_desktop_runtime::{
    load_or_create_settings, write_settings, DesktopPaths, NetworkProfile, DEFAULT_PEER_LISTEN,
    DEFAULT_RELAY_TTL_DAYS,
};

#[test]
fn p3_roundtrip_persists_default_relay_ttl() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P3;
    s.peer_listen = None;
    s.relay_ttl_days = None;
    write_settings(&paths, &s).unwrap();
    let loaded = load_or_create_settings(&paths).unwrap();
    assert_eq!(loaded.network_profile, NetworkProfile::P3);
    assert_eq!(loaded.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
    assert_eq!(loaded.relay_ttl_days, Some(DEFAULT_RELAY_TTL_DAYS));
}

#[test]
fn p5_write_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P5;
    s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
    let err = write_settings(&paths, &s).unwrap_err().to_string();
    assert!(err.contains("P0|P1|P2|P3|P4"), "{err}");
}

#[test]
fn p3_custom_relay_ttl_kept() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P3;
    s.peer_listen = Some("127.0.0.1:19003".into());
    s.relay_ttl_days = Some(14);
    write_settings(&paths, &s).unwrap();
    let loaded = load_or_create_settings(&paths).unwrap();
    assert_eq!(loaded.relay_ttl_days, Some(14));
    assert_eq!(loaded.peer_listen.as_deref(), Some("127.0.0.1:19003"));
}
