//! Phase Q contract smoke (#275 wiring … #285 RFC-0164 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_q_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-q-plan.md")).unwrap();
    for needle in [
        "Phase Q",
        "#275",
        "#285",
        "cross-install",
        "light monitoring",
        "Unique Desktop identity",
        "Model light observe",
        "Reachability",
        "AIRA-RFC-0164",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE P closed",
        "GPU marketplace",
        "Calculate 2 + 2",
        "first OPEN `#276`",
        "00f19cf",
    ] {
        assert!(text.contains(needle), "phase-q-plan missing: {needle}");
    }
}

#[test]
fn phase_q_queue_wiring() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-q-plan.md"));
    assert!(text.contains("| 275 | **DONE**"), "QUEUE #275 must be DONE");
    assert!(
        !text.contains("| 275 | **OPEN**"),
        "QUEUE #275 must not stay OPEN"
    );
    assert!(
        text.contains("| 276 | **OPEN**"),
        "QUEUE #276 must be first OPEN"
    );
    for n in 277..=285 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-310",
        "RFC-0164",
        "first OPEN `#276`",
        "QUEUE P closed",
        "desktop-ux.md",
        "aira-current.md",
        "00f19cf",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_q_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in ["phase-q-plan.md", "#275", "RFC-0164", "first OPEN `#276`"] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0164_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0164") || n.contains("rfc-0164"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0164 must stay file-free until #285; found {hits:?}"
    );
}

#[test]
fn phase_q_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-q-plan.md") || readme.contains("Phase Q"));
    assert!(
        !readme.contains("first OPEN `#268`"),
        "README must not keep stale first OPEN #268"
    );
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-q-plan.md"));
}

#[test]
fn phase_q_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-q-plan.md") || text.contains("Phase Q"));
    assert!(text.contains("#276") || text.contains("перший OPEN"));
    assert!(
        !text.contains("немає OPEN") || text.contains("Phase Q"),
        "NEXT_PROBLEM must point at Phase Q backlog"
    );
}

#[test]
fn phase_q_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #275 |") || status.contains("#275"));
    assert!(status.contains("phase_q_doc.rs"));
    assert!(status.contains("phase-q-plan.md"));
    assert!(status.contains("RFC-0164") || status.contains("0164"));
}

#[test]
fn phase_p_points_to_phase_q() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-p-plan.md")).unwrap();
    assert!(
        text.contains("phase-q-plan.md") || text.contains("Phase Q"),
        "phase-p-plan should point at Phase Q after activation"
    );
}
