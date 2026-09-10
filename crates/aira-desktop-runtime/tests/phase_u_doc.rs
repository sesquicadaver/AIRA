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
        "first OPEN `#315`",
        "RFC-0199",
        "#314",
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
fn phase_u_queue_314_done_315_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-u-plan.md"));
    for n in 313..=314 {
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
        text.contains("| 315 | **OPEN**"),
        "QUEUE #315 must be first OPEN"
    );
    for needle in [
        "Analyze-350",
        "RFC-0199",
        "RFC-0198",
        "first OPEN `#315`",
        "#314",
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
    for needle in [
        "phase-u-plan.md",
        "#314",
        "first OPEN `#315`",
        "RFC-0198",
        "RFC-0199",
    ] {
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
fn phase_u_rfc_0199_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0199-verification-capsule-sourced.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0199 missing");
    for needle in ["#314", "admitted capsule", "VerificationFailed", "RFC-0198"] {
        assert!(text.contains(needle), "RFC-0199 missing: {needle}");
    }
}

#[test]
fn phase_u_verification_module_present() {
    let src =
        std::fs::read_to_string(repo_root().join("csu/verification-basic/src/lib.rs")).unwrap();
    for needle in [
        "#314",
        "admitted_capsule",
        "capsule_action_expression",
        "output_matches_capsule",
        "substituted_output_expression_is_not_verified",
        "missing_admitted_capsule_is_not_verified",
    ] {
        assert!(src.contains(needle), "verification-basic missing: {needle}");
    }
}

#[test]
fn phase_u_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-u-plan.md") || readme.contains("Phase U"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-u-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#315`") || docs.contains("#315"));
}

#[test]
fn phase_u_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-u-plan.md") || text.contains("Phase U"));
    assert!(text.contains("#315") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#314`") && !text.contains("first OPEN `#314`"),
        "NEXT_PROBLEM must not keep #314 as first-OPEN after close"
    );
}

#[test]
fn phase_u_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #314 |") || status.contains("#314"));
    assert!(status.contains("phase_u_doc.rs"));
    assert!(status.contains("phase-u-plan.md"));
    assert!(status.contains("RFC-0199") || status.contains("0199"));
}

#[test]
fn phase_t_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-t-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE T closed") || text.contains("RFC-0192"),
        "phase-t-plan should remain closed canon"
    );
}
