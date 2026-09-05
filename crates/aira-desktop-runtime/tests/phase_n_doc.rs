//! Phase N contract smoke (#231–#234). Per-atom tests land with #235–#247.

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
        "#234",
        "#247",
        "Presence Record",
        "preferred_port",
        "P_AIRA",
        "1491",
        "aira:network:public:v1",
        "EvmRendezvousProvider",
        "AIRA-RFC-0123",
        "confirmed free",
        "GPU marketplace",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_234_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=234 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 235..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #234"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #234"
        );
    }
    for needle in [
        "N3 Presence",
        "Analyze-269",
        "RFC-0126",
        "Presence Record",
        "перший OPEN `#235`",
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
fn phase_n_rfc_0126_presence_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0126-presence-record.md"))
            .unwrap();
    for needle in [
        "NodePresenceRecord",
        "presence-record:0.1",
        "canonical",
        "Ed25519",
        "#234",
        "#235",
        "DISCOVERED",
        "TRUSTED",
        "P_AIRA",
    ] {
        assert!(text.contains(needle), "RFC-0126 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#234"));
    assert!(docs.contains("#235"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_234() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #234 | Presence Record"));
    assert!(status.contains("RFC-0126"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#234`") && !text.contains("first OPEN `#234`"),
        "NEXT_PROBLEM must not keep #234 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#235`") || text.contains("first OPEN `#235`"),
        "NEXT_PROBLEM must point at first OPEN #235"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_presence_module_contract() {
    assert_eq!(
        aira_peer::PRESENCE_SCHEMA,
        "aira:schema:peer:presence-record:0.1"
    );
    assert_eq!(aira_peer::PUBLIC_NETWORK_ID, "aira:network:public:v1");
    let path = repo_root().join("schemas/peer/presence-record.schema.json");
    assert!(path.is_file(), "missing presence schema");
}
