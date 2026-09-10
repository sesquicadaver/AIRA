//! Phase V contract smoke (#323 wiring … #330 RFC-0208 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_v_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-v-plan.md")).unwrap();
    for needle in [
        "Phase V",
        "Repair Pack 1",
        "#323",
        "#330",
        "IN PROGRESS",
        "Admission snapshot",
        "Reuse after constraints",
        "activate_verified",
        "Capsule↔Output↔Result",
        "AIRA-RFC-0208",
        "confirmed free",
        "QUEUE U closed",
        "RFC-0207",
        "first OPEN `#324`",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-v-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-v-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        !text.contains("**DONE** @ [AIRA-RFC-0208"),
        "phase-v-plan must not claim RFC-0208 DONE before #330"
    );
}

#[test]
fn phase_v_queue_323_done_324_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-v-plan.md"));
    assert!(
        text.contains("| 323 | **DONE**"),
        "QUEUE #323 must be DONE"
    );
    assert!(
        text.contains("| 324 | **OPEN**"),
        "QUEUE #324 must be OPEN"
    );
    for n in 325..=330 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE yet"
        );
    }
    for needle in [
        "Analyze-360",
        "RFC-0208",
        "Repair Pack 1",
        "admission integrity",
        "first OPEN `#324`",
        "QUEUE U closed",
        "desktop-ux.md",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0208_file_free() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0208-phase-v-admission-integrity.md");
    assert!(
        !path.exists(),
        "RFC-0208 must stay file-free until #330 close"
    );
}

#[test]
fn phase_v_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-v-plan.md",
        "#323",
        "#324",
        "RFC-0208",
        "QUEUE U closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_v_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-v-plan.md") || readme.contains("Phase V"));
    assert!(readme.contains("#324") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-v-plan.md"));
    assert!(docs.contains("#323") || docs.contains("IN PROGRESS"));
    assert!(docs.contains("repair-package-1.md"));
}

#[test]
fn phase_v_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-v-plan.md") || text.contains("Phase V"));
    assert!(text.contains("#324") || text.contains("перший OPEN"));
    assert!(text.contains("phase-u-plan.md") || text.contains("Phase U"));
    assert!(text.contains("QUEUE U closed") || text.contains("RFC-0198"));
}

#[test]
fn phase_v_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #323 |") || status.contains("#323"));
    assert!(status.contains("phase_v_doc.rs"));
    assert!(status.contains("phase-v-plan.md"));
    assert!(status.contains("RFC-0208") || status.contains("0208"));
    assert!(status.contains("#324"));
}

#[test]
fn phase_v_repair_package_1_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-1.md")).unwrap();
    for needle in [
        "IN PROGRESS",
        "phase-v-plan.md",
        "#323",
        "#324",
        "RFC-0208",
        "Analyze-360",
    ] {
        assert!(text.contains(needle), "repair-package-1 missing: {needle}");
    }
}

#[test]
fn phase_v_pack0_still_done() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-0.md")).unwrap();
    assert!(text.contains("**DONE**"));
    assert!(text.contains("RFC-0207") || text.contains("0207"));
}

#[test]
fn phase_u_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-u-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE U closed") || text.contains("RFC-0198"),
        "phase-u-plan should remain closed canon"
    );
}
