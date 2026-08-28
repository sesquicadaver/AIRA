//! Phase H wiring contract (#152): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_h_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-h-plan.md")).unwrap();
    for needle in [
        "Phase H",
        "#152",
        "#183",
        "H1 durable stores",
        "H3 CRP",
        "H4 settlement",
        "H5 research promotion",
        "AIRA-RFC-0077",
        "без вилок",
    ] {
        assert!(text.contains(needle), "phase-h-plan missing: {needle}");
    }
}

#[test]
fn phase_h_queue_wiring_152_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-h-plan.md"), "QUEUE missing phase-h-plan");
    assert!(
        text.contains("| 152 | **DONE**"),
        "QUEUE #152 must be DONE after wiring"
    );
    assert!(
        text.contains("| 153 | **OPEN**"),
        "QUEUE #153 must be next OPEN"
    );
    assert!(text.contains("| 183 | **OPEN**"), "QUEUE missing #183");
    for needle in ["H0 govern", "H1 durable stores", "H3 CRP local", "RFC-0077"] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_h_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-h-plan.md"));
    assert!(readme.contains("#152"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-h-plan.md"));
    assert!(docs.contains("#152"));
}

#[test]
fn phase_g_points_to_phase_h() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-g-plan.md")).unwrap();
    assert!(text.contains("phase-h-plan.md"));
    assert!(text.contains("#152"));
}
