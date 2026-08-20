//! Launcher unit tests (QUEUE #77).

use aira_desktop_runtime::{
    install_launcher_to, uninstall_launcher_from, validate_desktop_entry, validate_desktop_file,
    AIRA_DESKTOP_ENTRY, AIRA_DESKTOP_FILENAME,
};
use std::path::PathBuf;

#[test]
fn embedded_desktop_entry_valid() {
    validate_desktop_entry(AIRA_DESKTOP_ENTRY).expect("embedded entry");
}

#[test]
fn deploy_desktop_file_matches_embed() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../deploy/desktop/aira.desktop");
    validate_desktop_file(&path).expect("deploy file");
    let disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(disk, AIRA_DESKTOP_ENTRY);
}

#[test]
fn install_uninstall_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let apps = tmp.path().join("applications");
    let dest = install_launcher_to(&apps).expect("install");
    assert!(dest.ends_with(AIRA_DESKTOP_FILENAME));
    assert!(dest.is_file());
    validate_desktop_file(&dest).unwrap();
    let removed = uninstall_launcher_from(&apps).expect("uninstall");
    assert_eq!(removed, Some(dest));
    assert!(uninstall_launcher_from(&apps).unwrap().is_none());
}
