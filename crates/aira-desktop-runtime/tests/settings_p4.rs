//! Settings P4 gossip (QUEUE #100).

use aira_desktop_runtime::{
    load_or_create_settings, write_settings, DesktopPaths, NetworkProfile, DEFAULT_PEER_LISTEN,
};

#[test]
fn p4_roundtrip_persists_peer_listen() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P4;
    s.peer_listen = None;
    s.relay_ttl_days = Some(31);
    write_settings(&paths, &s).unwrap();
    let loaded = load_or_create_settings(&paths).unwrap();
    assert_eq!(loaded.network_profile, NetworkProfile::P4);
    assert_eq!(loaded.peer_listen.as_deref(), Some(DEFAULT_PEER_LISTEN));
    assert!(loaded.relay_ttl_days.is_none(), "P4 clears relay TTL");
}

#[test]
fn p4_mutex_clears_relay_ttl_from_p3() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let mut s = load_or_create_settings(&paths).unwrap();
    s.network_profile = NetworkProfile::P3;
    s.peer_listen = Some(DEFAULT_PEER_LISTEN.into());
    s.relay_ttl_days = Some(21);
    write_settings(&paths, &s).unwrap();
    s.network_profile = NetworkProfile::P4;
    write_settings(&paths, &s).unwrap();
    let loaded = load_or_create_settings(&paths).unwrap();
    assert_eq!(loaded.network_profile, NetworkProfile::P4);
    assert!(loaded.relay_ttl_days.is_none());
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
