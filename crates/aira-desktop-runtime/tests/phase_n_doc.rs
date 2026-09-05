//! Phase N contract smoke (#231–#242). Per-atom tests land with #243–#247.

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
        "#242",
        "#247",
        "Presence refresh",
        "EvmRendezvousProvider",
        "P_AIRA",
        "AIRA-RFC-0123",
        "confirmed free",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_242_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=242 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 243..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #242"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #242"
        );
    }
    for needle in [
        "N11 Presence refresh",
        "Analyze-277",
        "RFC-0134",
        "перший OPEN `#243`",
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
fn phase_n_rfc_0134_refresh_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0134-presence-refresh.md"))
            .unwrap();
    for needle in [
        "refresh_and_sign_presence",
        "endpoint_change",
        "retain_unexpired_presence",
        "sequence",
        "#242",
        "#243",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0134 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#242"));
    assert!(docs.contains("#243"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_242() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #242 | Presence refresh"));
    assert!(status.contains("RFC-0134"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#242`") && !text.contains("first OPEN `#242`"),
        "NEXT_PROBLEM must not keep #242 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#243`") || text.contains("first OPEN `#243`"),
        "NEXT_PROBLEM must point at first OPEN #243"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_presence_refresh_contract() {
    assert_eq!(aira_peer::PRESENCE_REFRESH_TTL_SECS_DEFAULT, 24 * 60 * 60);
}
