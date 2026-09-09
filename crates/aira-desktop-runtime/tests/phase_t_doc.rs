//! Phase T contract smoke (#306 wiring … #312 RFC-0192 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_t_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-t-plan.md")).unwrap();
    for needle in [
        "Phase T",
        "#306",
        "#312",
        "Operation lifecycle",
        "Dial evidence ≠ live",
        "Opt-in dial off UI-thread",
        "Observe miss fail durable",
        "Quit∥Submit",
        "AddressBook candidate",
        "AIRA-RFC-0192",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE S closed",
        "first OPEN `#307`",
        "3301d27",
        "live_session_count",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-t-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-t-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("IN PROGRESS") || text.contains("**IN PROGRESS**"),
        "phase-t-plan must be IN PROGRESS after wiring"
    );
}

#[test]
fn phase_t_queue_306_done_307_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-t-plan.md"));
    assert!(text.contains("| 306 | **DONE**"), "QUEUE #306 must be DONE");
    assert!(
        !text.contains("| 306 | **OPEN**"),
        "QUEUE #306 must not stay OPEN"
    );
    assert!(
        text.contains("| 307 | **OPEN**"),
        "QUEUE #307 must be first OPEN"
    );
    for n in 308..=312 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-342",
        "RFC-0192",
        "first OPEN `#307`",
        "#306",
        "QUEUE S closed",
        "desktop-ux.md",
        "Operation lifecycle",
        "live_session_count",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_t_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in ["phase-t-plan.md", "#306", "first OPEN `#307`", "RFC-0192"] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_t_rfc_0192_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0192") || n.contains("rfc-0192"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0192 must stay file-free until #312; found {hits:?}"
    );
}

#[test]
fn phase_t_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-t-plan.md") || readme.contains("Phase T"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-t-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#307`") || docs.contains("#307"));
}

#[test]
fn phase_t_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-t-plan.md") || text.contains("Phase T"));
    assert!(text.contains("#307") || text.contains("перший OPEN"));
}

#[test]
fn phase_t_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #306 |") || status.contains("#306"));
    assert!(status.contains("phase_t_doc.rs"));
    assert!(status.contains("phase-t-plan.md"));
    assert!(status.contains("RFC-0192") || status.contains("0192"));
}

#[test]
fn phase_s_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-s-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE S closed") || text.contains("RFC-0182"),
        "phase-s-plan should remain closed canon"
    );
}
