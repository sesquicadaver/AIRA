//! Phase W contract smoke (#331 wiring … #342 RFC-0215 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_w_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-w-plan.md")).unwrap();
    for needle in [
        "Phase W",
        "Pack 1 residual honesty",
        "#331",
        "#342",
        "IN PROGRESS",
        "Math capsule fidelity",
        "Strict submit decoding",
        "Constraints enforce-or-reject",
        "Activate evidence authority",
        "Backend verified binding",
        "AIRA-RFC-0215",
        "confirmed free",
        "QUEUE V closed",
        "RFC-0208",
        "first OPEN `#332`",
        "D1",
        "D6",
        "GPU marketplace",
        "Calculate 2 + 2",
        "M1–M6",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-w-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-w-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        !text.contains("**DONE** @ [AIRA-RFC-0215"),
        "phase-w-plan must not claim RFC-0215 DONE before #342"
    );
}

#[test]
fn phase_w_queue_331_done_332_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-w-plan.md"));
    assert!(text.contains("| 331 | **DONE**"), "QUEUE #331 must be DONE");
    assert!(text.contains("| 332 | **OPEN**"), "QUEUE #332 must be OPEN");
    for n in 333..=342 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 332 | **DONE**") && !text.contains("| 342 | **DONE**"),
        "QUEUE #332/#342 must not be DONE yet"
    );
    for needle in [
        "Analyze-368",
        "RFC-0215",
        "Pack 1 residual",
        "first OPEN `#332`",
        "QUEUE V closed",
        "desktop-ux.md",
        "Math capsule",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_w_rfc_0215_file_free() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0215-phase-w-pack1-residual-honesty.md");
    assert!(
        !path.exists(),
        "RFC-0215 must stay file-free until #342 close"
    );
}

#[test]
fn phase_w_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-w-plan.md",
        "#331",
        "#332",
        "RFC-0215",
        "QUEUE V closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_w_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-w-plan.md") || readme.contains("Phase W"));
    assert!(readme.contains("#332") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-w-plan.md"));
    assert!(docs.contains("#331") || docs.contains("IN PROGRESS"));
}

#[test]
fn phase_w_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-w-plan.md") || text.contains("Phase W"));
    assert!(text.contains("#332") || text.contains("перший OPEN"));
    assert!(text.contains("QUEUE V closed") || text.contains("RFC-0208"));
}

#[test]
fn phase_w_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #331 |") || status.contains("#331"));
    assert!(status.contains("phase_w_doc.rs"));
    assert!(status.contains("phase-w-plan.md"));
    assert!(status.contains("RFC-0215") || status.contains("0215"));
    assert!(status.contains("#332"));
}

#[test]
fn phase_w_repair_package_1_honesty() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-1.md")).unwrap();
    for needle in [
        "RFC-0208",
        "PARTIAL",
        "phase-w-plan.md",
        "#331",
        "#332",
        "RFC-0215",
    ] {
        assert!(text.contains(needle), "repair-package-1 missing: {needle}");
    }
}

#[test]
fn phase_v_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-v-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE V closed") || text.contains("RFC-0208"),
        "phase-v-plan should remain closed canon"
    );
    assert!(
        text.contains("**DONE** @ [AIRA-RFC-0208"),
        "Phase V must stay DONE @ RFC-0208"
    );
}
