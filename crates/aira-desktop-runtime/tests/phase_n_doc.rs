//! Phase N contract smoke (#231–#241). Per-atom tests land with #242–#247.

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
        "#241",
        "#247",
        "Relay integration",
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
fn phase_n_queue_241_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=241 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 242..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #241"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #241"
        );
    }
    for needle in [
        "N10 Relay integration",
        "Analyze-276",
        "RFC-0133",
        "перший OPEN `#242`",
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
fn phase_n_rfc_0133_relay_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0133-relay-integration.md"))
            .unwrap();
    for needle in [
        "plan_dial_path",
        "RelayAdvertisement",
        "select_relay_reservations",
        "RELAY_RESERVATION_TARGET",
        "#241",
        "#242",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0133 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#241"));
    assert!(docs.contains("#242"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_241() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #241 | Relay integration"));
    assert!(status.contains("RFC-0133"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#241`") && !text.contains("first OPEN `#241`"),
        "NEXT_PROBLEM must not keep #241 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#242`") || text.contains("first OPEN `#242`"),
        "NEXT_PROBLEM must point at first OPEN #242"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_relay_integrate_contract() {
    let steps = aira_peer::plan_dial_path(aira_peer::DialPathInput {
        direct_addr: Some("127.0.0.1:49157".into()),
        nat_observed_addr: None,
        relays: vec![],
    })
    .unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].kind(), "direct");
    assert_eq!(aira_peer::RELAY_RESERVATION_TARGET, 2);
}
