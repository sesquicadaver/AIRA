//! Phase N contract smoke (#231–#235). Per-atom tests land with #236–#247.

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
        "#235",
        "#247",
        "Presence Record",
        "RendezvousProvider",
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
fn phase_n_queue_235_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=235 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 236..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #235"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #235"
        );
    }
    for needle in [
        "N4 RendezvousProvider",
        "Analyze-270",
        "RFC-0127",
        "перший OPEN `#236`",
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
fn phase_n_rfc_0127_rendezvous_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0127-rendezvous-provider.md"))
            .unwrap();
    for needle in [
        "RendezvousProvider",
        "MockRendezvousProvider",
        "#235",
        "#236",
        "aira-core",
        "DISCOVERED",
        "TRUSTED",
        "publish_presence",
    ] {
        assert!(text.contains(needle), "RFC-0127 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#235"));
    assert!(docs.contains("#236"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_235() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #235 | RendezvousProvider"));
    assert!(status.contains("RFC-0127"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#235`") && !text.contains("first OPEN `#235`"),
        "NEXT_PROBLEM must not keep #235 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#236`") || text.contains("first OPEN `#236`"),
        "NEXT_PROBLEM must point at first OPEN #236"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_rendezvous_module_contract() {
    assert_eq!(aira_peer::RENDEZVOUS_KIND_MOCK, "mock");
    let mut mock = aira_peer::MockRendezvousProvider::new();
    assert!(mock.is_empty());
    assert_eq!(aira_peer::RendezvousProvider::provider_kind(&mock), "mock");
    let _ = &mut mock;
}
