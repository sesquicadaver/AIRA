//! Phase X contract smoke (#343 wiring … #358 RFC-0226 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_x_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-x-plan.md")).unwrap();
    for needle in [
        "Phase X",
        "Pack 2",
        "local multi-model GUI",
        "#343",
        "#358",
        "IN PROGRESS",
        "Per-model inventory",
        "Model select API",
        "Settings Models",
        "Work Compare",
        "Two-model installed acceptance",
        "AIRA-RFC-0226",
        "confirmed free",
        "QUEUE W closed",
        "RFC-0215",
        "RFC-0226",
        "first OPEN `#344`",
        "M1",
        "M6",
        "GPU marketplace",
        "aira-core",
        "public bind",
        "phase-w-plan.md",
    ] {
        assert!(text.contains(needle), "phase-x-plan missing: {needle}");
    }
    assert!(
        !text.contains("**QUEUED** (записано") || text.contains("**IN PROGRESS**"),
        "phase-x-plan must be activated (IN PROGRESS) after #343"
    );
    assert!(
        !text.contains("**DONE** @ [AIRA-RFC-0226"),
        "phase-x-plan must not claim RFC-0226 DONE before #358"
    );
    assert!(
        !text.contains("first OPEN `#343`"),
        "phase-x-plan must advance tip past #343"
    );
}

#[test]
fn phase_x_queue_343_done_344_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-x-plan.md"));
    assert!(text.contains("| 343 | **DONE**"), "QUEUE #343 must be DONE");
    assert!(text.contains("| 344 | **OPEN**"), "QUEUE #344 must be OPEN");
    for n in 344..=358 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 343 | **OPEN**"),
        "QUEUE #343 must not stay OPEN"
    );
    for needle in [
        "Analyze-380",
        "Analyze-381",
        "RFC-0226",
        "RFC-0215",
        "QUEUE W closed",
        "Pack 2",
        "first OPEN `#344`",
        "phase_x_doc",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        text.contains("**Перший OPEN:** `#344`"),
        "QUEUE tip must be first OPEN #344 after #343"
    );
    assert!(
        !text.contains("**Перший OPEN:** `#343`"),
        "QUEUE tip must not keep #343 as first-OPEN"
    );
}

#[test]
fn phase_x_rfc_0226_file_free() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0226-phase-x-pack2-multi-model-gui.md");
    assert!(
        !path.exists(),
        "RFC-0226 must stay file-free until #358 close"
    );
}

#[test]
fn phase_w_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-w-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE W closed") || text.contains("RFC-0215"),
        "phase-w-plan should remain closed canon"
    );
    assert!(
        text.contains("**DONE** @ [AIRA-RFC-0215"),
        "Phase W must stay DONE @ RFC-0215"
    );
}

#[test]
fn phase_x_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-x-plan.md",
        "#343",
        "#344",
        "RFC-0226",
        "QUEUE W closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_x_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-x-plan.md") || readme.contains("Phase X"));
    assert!(readme.contains("#344") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-x-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("#344"));
}

#[test]
fn phase_x_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-x-plan.md") || text.contains("Phase X"));
    assert!(text.contains("#344") || text.contains("перший OPEN"));
    assert!(text.contains("QUEUE W closed") || text.contains("RFC-0215"));
}

#[test]
fn phase_x_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #343 |") || status.contains("#343"));
    assert!(status.contains("phase_x_doc.rs"));
    assert!(status.contains("phase-x-plan.md"));
    assert!(status.contains("RFC-0226") || status.contains("0226"));
    assert!(status.contains("#344"));
}
