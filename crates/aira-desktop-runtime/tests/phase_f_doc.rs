//! Phase F docs closure contract (#119).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_f_readme_contract() {
    let text = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    for needle in [
        "Phase F",
        "phase-f-plan.md",
        "implementation-status.md",
        "ci-governance.md",
        "conformance run --profile C2",
        "1.94.0",
        "#107",
        "#119",
    ] {
        assert!(text.contains(needle), "README missing: {needle}");
    }
}

#[test]
fn phase_f_implementation_status_contract() {
    let text = std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "c0.object.handle_opacity",
        "c0.object.verify_on_read",
        "c0.artifact.verify_on_read",
        "c0.csu.dispatch_policy",
        "c0.acquisition.fail_closed",
        "conformance-c2",
        "AIRA-RFC-0068",
        "identifier:0.1",
        "epistemic:assessment:0.1",
        "context-artifact:0.1",
        "Phase F",
        "#117",
    ] {
        assert!(
            text.contains(needle),
            "implementation-status missing: {needle}"
        );
    }
}

#[test]
fn phase_f_plan_done() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-f-plan.md")).unwrap();
    assert!(text.contains("**DONE**"), "phase-f-plan should mark DONE");
    assert!(text.contains("AIRA-RFC-0068"));
}
