//! Phase U contract smoke (#313 wiring … #322 RFC-0198 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_u_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-u-plan.md")).unwrap();
    for needle in [
        "Phase U",
        "#313",
        "#322",
        "Cross-path contract",
        "Verification capsule-sourced",
        "Result-by-problem",
        "CLI identity create",
        "Policy audit",
        "Failed submit durable",
        "Submit executor honesty",
        "AddressBook selective rollback",
        "systemd/docs prime-port",
        "AIRA-RFC-0198",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE T closed",
        "first OPEN `#314`",
        "ef5f69c",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-u-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-u-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("IN PROGRESS") || text.contains("**IN PROGRESS**"),
        "phase-u-plan must be IN PROGRESS after wiring"
    );
}

#[test]
fn phase_u_queue_313_done_314_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-u-plan.md"));
    assert!(text.contains("| 313 | **DONE**"), "QUEUE #313 must be DONE");
    assert!(
        !text.contains("| 313 | **OPEN**"),
        "QUEUE #313 must not stay OPEN"
    );
    assert!(
        text.contains("| 314 | **OPEN**"),
        "QUEUE #314 must be first OPEN"
    );
    for n in 315..=322 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-349",
        "RFC-0198",
        "first OPEN `#314`",
        "#313",
        "QUEUE T closed",
        "desktop-ux.md",
        "Cross-path contract",
        "ef5f69c",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_u_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in ["phase-u-plan.md", "#313", "first OPEN `#314`", "RFC-0198"] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_u_rfc_0198_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0198") || n.contains("rfc-0198"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0198 must stay file-free until #322; found {hits:?}"
    );
}

#[test]
fn phase_u_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-u-plan.md") || readme.contains("Phase U"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-u-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#314`") || docs.contains("#314"));
}

#[test]
fn phase_u_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-u-plan.md") || text.contains("Phase U"));
    assert!(text.contains("#314") || text.contains("перший OPEN"));
}

#[test]
fn phase_u_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #313 |") || status.contains("#313"));
    assert!(status.contains("phase_u_doc.rs"));
    assert!(status.contains("phase-u-plan.md"));
    assert!(status.contains("RFC-0198") || status.contains("0198"));
}

#[test]
fn phase_t_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-t-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE T closed") || text.contains("RFC-0192"),
        "phase-t-plan should remain closed canon"
    );
}
