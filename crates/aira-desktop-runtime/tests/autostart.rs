//! Autostart hook tests (QUEUE #78).

use aira_desktop_runtime::{
    autostart_desktop_entry, is_autostart_enabled_in, set_autostart_in, AIRA_AUTOSTART_FILENAME,
};

#[test]
fn autostart_entry_execs_aira_desktop() {
    let body = autostart_desktop_entry();
    assert!(body.contains("Exec=aira-desktop"));
    assert!(body.contains("Type=Application"));
}

#[test]
fn set_autostart_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("autostart");
    assert!(!is_autostart_enabled_in(&dir));
    let path = set_autostart_in(&dir, true).unwrap();
    assert!(path.ends_with(AIRA_AUTOSTART_FILENAME));
    assert!(is_autostart_enabled_in(&dir));
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text, autostart_desktop_entry());
    set_autostart_in(&dir, false).unwrap();
    assert!(!is_autostart_enabled_in(&dir));
}
