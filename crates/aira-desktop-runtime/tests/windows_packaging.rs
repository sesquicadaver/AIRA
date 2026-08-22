//! Windows zip packaging layout smoke (QUEUE #92).

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn package_script_layout_only_smoke() {
    let root = repo_root();
    let script = root.join("scripts/package-desktop-windows.sh");
    assert!(script.is_file(), "missing {}", script.display());

    let out = Command::new("bash")
        .arg(&script)
        .arg("--layout-only")
        .arg("--out")
        .arg(std::env::temp_dir().join("aira-windows-pack-test"))
        .current_dir(&root)
        .output()
        .expect("run package-desktop-windows.sh");
    assert!(
        out.status.success(),
        "package failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("OK:"));
    assert!(stdout.contains(".zip"));
}

#[test]
fn bash_n_packaging_script() {
    let path = repo_root().join("scripts/package-desktop-windows.sh");
    let out = Command::new("bash")
        .arg("-n")
        .arg(&path)
        .output()
        .unwrap_or_else(|e| panic!("bash -n {}: {e}", path.display()));
    assert!(
        out.status.success(),
        "bash -n {}: {}",
        path.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn windows_startup_filename_contract() {
    use aira_desktop_runtime::AIRA_WINDOWS_STARTUP_FILENAME;
    assert_eq!(AIRA_WINDOWS_STARTUP_FILENAME, "AIRA Desktop.bat");
}

#[test]
fn install_dest_matches_autostart_resolve_pattern() {
    let manifest = repo_root().join("scripts/package-desktop-windows.sh");
    let text = std::fs::read_to_string(manifest).expect("read package script");
    assert!(text.contains("Programs\\\\AIRA"));
    assert!(text.contains("aira-desktop.exe"));
}
