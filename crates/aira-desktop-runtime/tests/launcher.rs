//! Launcher unit tests (QUEUE #77 / #79).

use aira_desktop_runtime::{
    install_menu_entries_to, uninstall_menu_entries_from, validate_desktop_entry,
    validate_desktop_file, validate_gui_desktop_entry, validate_gui_desktop_file,
    AIRA_DESKTOP_ENTRY, AIRA_DESKTOP_FILENAME, AIRA_GUI_DESKTOP_ENTRY, AIRA_GUI_DESKTOP_FILENAME,
};
use std::path::PathBuf;

#[test]
fn embedded_desktop_entry_valid() {
    validate_desktop_entry(AIRA_DESKTOP_ENTRY).expect("embedded entry");
}

#[test]
fn embedded_gui_desktop_entry_valid() {
    validate_gui_desktop_entry(AIRA_GUI_DESKTOP_ENTRY).expect("embedded gui entry");
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
fn deploy_gui_desktop_file_matches_embed() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../deploy/desktop/aira-desktop.desktop");
    validate_gui_desktop_file(&path).expect("deploy gui file");
    let disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(disk, AIRA_GUI_DESKTOP_ENTRY);
}

#[test]
fn install_uninstall_both_menu_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let apps = tmp.path().join("applications");
    let (start, gui) = install_menu_entries_to(&apps).expect("install");
    assert!(start.ends_with(AIRA_DESKTOP_FILENAME));
    assert!(gui.ends_with(AIRA_GUI_DESKTOP_FILENAME));
    assert!(start.is_file());
    assert!(gui.is_file());
    validate_desktop_file(&start).unwrap();
    validate_gui_desktop_file(&gui).unwrap();
    let removed = uninstall_menu_entries_from(&apps).expect("uninstall");
    assert_eq!(removed.len(), 2);
    assert!(uninstall_menu_entries_from(&apps).unwrap().is_empty());
}

#[test]
fn autostart_exec_matches_packaged_gui_binary_name() {
    let body = aira_desktop_runtime::autostart_desktop_entry();
    assert!(
        body.contains("Exec=aira-desktop"),
        "packaging must keep aira-desktop on PATH for #78 autostart"
    );
    assert!(AIRA_GUI_DESKTOP_ENTRY.contains("Exec=aira-desktop"));
}
