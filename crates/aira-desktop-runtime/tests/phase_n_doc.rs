//! Phase N contract smoke (#231–#247 DONE @ RFC-0123). QUEUE N closed.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_n_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-plan.md")).unwrap();
    for needle in [
        "Phase N",
        "Global Node Rendezvous",
        "#231",
        "#247",
        "DONE",
        "P_AIRA",
        "AIRA-RFC-0123",
        "aira-core",
        "QUEUE N closed",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_closed() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=247 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for needle in [
        "QUEUE N closed",
        "Analyze-282",
        "RFC-0123",
        "RFC-0138",
        "no OPEN N",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_n_rfc_0123_present() {
    let text = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0123-phase-n-global-rendezvous.md"),
    )
    .unwrap();
    for needle in [
        "Phase N",
        "QUEUE N closed",
        "DISCOVERED",
        "TRUSTED",
        "P_AIRA",
        "RFC-0124",
        "RFC-0138",
        "#247",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0123 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    assert!(readme.contains("RFC-0123") || readme.contains("AIRA-RFC-0123"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#247"));
    assert!(docs.contains("QUEUE N closed") || docs.contains("DONE @ RFC-0123"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_247() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #247 |"));
    assert!(status.contains("RFC-0123"));
    assert!(status.contains("QUEUE N closed") || status.contains("**DONE** @ this PR"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("QUEUE N closed") || text.contains("Phase N") && text.contains("DONE"),
        "NEXT_PROBLEM must reflect Phase N closed"
    );
    assert!(
        !text.contains("перший OPEN = `#247`") && !text.contains("first OPEN `#247`"),
        "NEXT_PROBLEM must not keep #247 as first-OPEN"
    );
}

#[test]
fn phase_n_local_file_rendezvous_kind() {
    assert_eq!(aira_peer::RENDEZVOUS_KIND_LOCAL_FILE, "local-file");
}
