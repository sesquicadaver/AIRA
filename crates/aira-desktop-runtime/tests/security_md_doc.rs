//! SECURITY.md contract (#110).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn security_md_not_skeleton_only() {
    let text = std::fs::read_to_string(repo_root().join("SECURITY.md")).expect("read SECURITY.md");
    assert!(
        !text.contains("workspace skeleton"),
        "SECURITY.md must describe actual controls, not skeleton-only posture"
    );
    for needle in [
        "Developer Preview",
        "dependency firewall",
        "CSU isolation",
        "unsigned",
        "private artifacts",
        "loopback",
        "desktop-network-profiles",
        "ci-governance.md",
        "implementation-status.md",
        "AIRA-RFC-0059",
        "Security Advisories",
    ] {
        assert!(text.contains(needle), "SECURITY.md missing: {needle}");
    }
}
