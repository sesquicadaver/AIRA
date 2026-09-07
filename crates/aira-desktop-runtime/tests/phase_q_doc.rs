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
        "first OPEN `#277`",
        "#276",
        "RFC-0165",
        "00f19cf",
    ] {
        assert!(text.contains(needle), "phase-q-plan missing: {needle}");
    }
}

#[test]
fn phase_q_queue_276_done_277_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-q-plan.md"));
    for n in 275..=276 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    assert!(
        text.contains("| 277 | **OPEN**"),
        "QUEUE #277 must be first OPEN"
    );
    for n in 278..=285 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-310",
        "RFC-0164",
        "first OPEN `#277`",
        "#276",
        "RFC-0165",
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
    for needle in ["phase-q-plan.md", "#276", "RFC-0165", "first OPEN `#277`"] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0165_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0165-unique-desktop-identity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0165 missing");
    for needle in ["#276", "desktop.", "KeyCollision", "ensure_bootstrap"] {
        assert!(text.contains(needle), "RFC-0165 missing: {needle}");
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
    assert!(text.contains("#277") || text.contains("перший OPEN"));
    assert!(
        !text.contains("немає OPEN") || text.contains("Phase Q"),
        "NEXT_PROBLEM must point at Phase Q backlog"
    );
}

#[test]
fn phase_q_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #276 |") || status.contains("#276"));
    assert!(status.contains("phase_q_doc.rs"));
    assert!(status.contains("phase-q-plan.md"));
    assert!(status.contains("RFC-0165") || status.contains("0165"));
}

#[test]
fn phase_p_points_to_phase_q() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-p-plan.md")).unwrap();
    assert!(
        text.contains("phase-q-plan.md") || text.contains("Phase Q"),
        "phase-p-plan should point at Phase Q after activation"
    );
}
