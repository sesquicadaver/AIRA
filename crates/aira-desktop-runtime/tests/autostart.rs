//! Autostart hook tests (QUEUE #78 Linux / #87 macOS).

use aira_desktop_runtime::{
    autostart_desktop_entry, is_autostart_enabled_in, is_launch_agent_enabled_in,
    launch_agent_plist, launch_agents_dir_for_home, set_autostart_in, set_launch_agent_in,
    AIRA_AUTOSTART_FILENAME, AIRA_LAUNCH_AGENT_FILENAME, AIRA_LAUNCH_AGENT_LABEL,
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

#[test]
fn launch_agent_roundtrip_under_home() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = launch_agents_dir_for_home(tmp.path());
    let prog = "/Applications/AIRA.app/Contents/MacOS/aira-desktop";
    let path = set_launch_agent_in(&dir, true, prog).unwrap();
    assert!(path.ends_with(AIRA_LAUNCH_AGENT_FILENAME));
    assert!(is_launch_agent_enabled_in(&dir));
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text, launch_agent_plist(prog));
    assert!(text.contains(AIRA_LAUNCH_AGENT_LABEL));
    set_launch_agent_in(&dir, false, prog).unwrap();
    assert!(!is_launch_agent_enabled_in(&dir));
}
