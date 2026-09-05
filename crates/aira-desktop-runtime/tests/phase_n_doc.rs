//! Phase N contract smoke (#231–#236). Per-atom tests land with #237–#247.

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
        "#236",
        "#247",
        "Presence Record",
        "RendezvousProvider",
        "EvmRendezvousProvider",
        "preferred_port",
        "P_AIRA",
        "1491",
        "aira:network:public:v1",
        "AIRA-RFC-0123",
        "confirmed free",
        "GPU marketplace",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_236_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=236 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 237..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #236"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #236"
        );
    }
    for needle in [
        "N5 EVM adapter",
        "Analyze-271",
        "RFC-0128",
        "перший OPEN `#237`",
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
fn phase_n_rfc_0128_evm_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0128-evm-rendezvous.md"))
            .unwrap();
    for needle in [
        "EvmRendezvousProvider",
        "80002",
        "137",
        "local double",
        "#236",
        "#237",
        "aira-core",
        "Ed25519",
    ] {
        assert!(text.contains(needle), "RFC-0128 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#236"));
    assert!(docs.contains("#237"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_236() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #236 | EVM ledger adapter"));
    assert!(status.contains("RFC-0128"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#236`") && !text.contains("first OPEN `#236`"),
        "NEXT_PROBLEM must not keep #236 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#237`") || text.contains("first OPEN `#237`"),
        "NEXT_PROBLEM must point at first OPEN #237"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_evm_module_contract() {
    assert_eq!(aira_peer::RENDEZVOUS_KIND_EVM, "evm");
    assert_eq!(aira_peer::EVM_CHAIN_AMOY, 80002);
    assert_eq!(aira_peer::EVM_CHAIN_POLYGON, 137);
    let evm = aira_peer::EvmRendezvousProvider::local_double();
    assert_eq!(aira_peer::RendezvousProvider::provider_kind(&evm), "evm");
    assert_eq!(evm.config().chain_id, aira_peer::EVM_CHAIN_LOCAL_DOUBLE);
}
