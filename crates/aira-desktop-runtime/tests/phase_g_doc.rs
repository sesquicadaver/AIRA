//! Phase G docs closure contract (#151).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_g_readme_contract() {
    let text = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    for needle in [
        "Phase G",
        "phase-g-plan.md",
        "implementation-status.md",
        "v0.2",
        "What AIRA is not",
        "GPU marketplace",
        "#120",
        "#151",
        "AIRA-RFC-0069",
    ] {
        assert!(text.contains(needle), "README missing: {needle}");
    }
}

#[test]
fn phase_g_implementation_status_contract() {
    let text = std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "Reference v0.2",
        "AIRA-RFC-0069",
        "c1.csu.external_partner_fixture",
        "epistemic_assessment_roundtrip_via_plane_and_session",
        "macos-codesign-notarize",
        "package-desktop-windows-msi",
        "package-desktop-linux-deb",
        "Phase G",
        "#151",
        "**DONE**",
    ] {
        assert!(
            text.contains(needle),
            "implementation-status missing: {needle}"
        );
    }
}

#[test]
fn phase_g_plan_done() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-g-plan.md")).unwrap();
    assert!(text.contains("**DONE**"), "phase-g-plan should mark DONE");
    assert!(text.contains("AIRA-RFC-0069"));
    assert!(text.contains("#151"));
}

#[test]
fn phase_g_rfc_0069_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0069 file");
    assert!(text.contains("Phase G"));
    assert!(text.contains("#120"));
    assert!(text.contains("#151"));
}
