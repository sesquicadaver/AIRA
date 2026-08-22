//! macOS .app packaging layout smoke (#88) + packaging docs contract (#89).

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn macos_info_plist_contract() {
    let path = repo_root().join("deploy/macos/Info.plist");
    let text = std::fs::read_to_string(&path).expect("read Info.plist");
    assert!(text.contains("<key>CFBundleExecutable</key>"));
    assert!(text.contains("<string>aira-desktop</string>"));
    assert!(text.contains("ai.aira.desktop"));
    assert!(text.contains("@VERSION@"));
}

#[test]
fn package_script_layout_only_smoke() {
    let root = repo_root();
    let script = root.join("scripts/package-desktop-macos.sh");
    assert!(script.is_file(), "missing {}", script.display());

    let out = Command::new("bash")
        .arg(&script)
        .arg("--layout-only")
        .arg("--out")
        .arg(std::env::temp_dir().join("aira-macos-pack-test"))
        .current_dir(&root)
        .output()
        .expect("run package-desktop-macos.sh");
    assert!(
        out.status.success(),
        "package failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("OK:"));
    assert!(stdout.contains(".tar.gz"));
}

#[test]
fn bash_n_packaging_scripts() {
    for rel in [
        "scripts/package-desktop-macos.sh",
        "deploy/macos/install-user.sh",
        "deploy/macos/uninstall-user.sh",
    ] {
        let path = repo_root().join(rel);
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
}

#[test]
fn launch_agent_label_matches_bundle_id() {
    use aira_desktop_runtime::AIRA_LAUNCH_AGENT_LABEL;
    let path = repo_root().join("deploy/macos/Info.plist");
    let text = std::fs::read_to_string(path).unwrap();
    assert!(
        text.contains(AIRA_LAUNCH_AGENT_LABEL),
        "LaunchAgent label must match CFBundleIdentifier for #87/#88"
    );
}

#[test]
fn macos_packaging_doc_contract() {
    let path = repo_root().join("docs/desktop-packaging-macos.md");
    let text = std::fs::read_to_string(&path).expect("read desktop-packaging-macos.md");
    for needle in [
        "scripts/package-desktop-macos.sh",
        "~/Applications/AIRA Desktop.app",
        "Library/Application Support/AIRA",
        "ai.aira.desktop.plist",
        "install.sh",
        "AIRA-RFC-0038",
        "no `cargo`",
    ] {
        assert!(text.contains(needle), "doc missing: {needle}");
    }
}
