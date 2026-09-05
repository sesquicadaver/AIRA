//! Phase N contract smoke (#231–#246). Per-atom tests land with #247.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0123_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0123") || n.contains("rfc-0123"))
        .collect()
}

#[test]
fn phase_n_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-plan.md")).unwrap();
    for needle in [
        "Phase N",
        "Global Node Rendezvous",
        "#231",
        "#246",
        "#247",
        "NAT/relay",
        "P_AIRA",
        "AIRA-RFC-0123",
        "confirmed free",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_246_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=246 {
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
        text.contains("| 247 | **OPEN**"),
        "QUEUE #247 must be OPEN after #246"
    );
    assert!(
        !text.contains("| 247 | **DONE**"),
        "QUEUE #247 must not be DONE at #246"
    );
    for needle in [
        "N15 NAT/relay",
        "Analyze-281",
        "RFC-0138",
        "перший OPEN `#247`",
        "QUEUE M closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_n_rfc_0123_id_free() {
    let hits = rfc_0123_hits();
    assert!(
        hits.is_empty(),
        "RFC-0123 must stay file-free until #247, found {hits:?}"
    );
}

#[test]
fn phase_n_rfc_0138_nat_relay_present() {
    let text = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0138-nat-relay-integration.md"),
    )
    .unwrap();
    for needle in [
        "inbound",
        "relay",
        "configure_inbound_blocked_via_relay",
        "#246",
        "#247",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0138 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#246"));
    assert!(docs.contains("#247"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_246() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #246 | NAT/relay"));
    assert!(status.contains("RFC-0138"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#246`") && !text.contains("first OPEN `#246`"),
        "NEXT_PROBLEM must not keep #246 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#247`") || text.contains("first OPEN `#247`"),
        "NEXT_PROBLEM must point at first OPEN #247"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_local_file_rendezvous_kind() {
    assert_eq!(aira_peer::RENDEZVOUS_KIND_LOCAL_FILE, "local-file");
}
