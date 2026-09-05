//! Phase N contract smoke (#231–#237). Per-atom tests land with #238–#247.

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
        "#237",
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
fn phase_n_queue_237_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=237 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 238..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #237"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #237"
        );
    }
    for needle in [
        "N6 publish/query",
        "Analyze-272",
        "RFC-0129",
        "перший OPEN `#238`",
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
fn phase_n_rfc_0129_publish_present() {
    let text = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0129-rendezvous-publish-query.md"),
    )
    .unwrap();
    for needle in [
        "RendezvousClient",
        "TTL",
        "rendezvous.json",
        "#237",
        "#238",
        "sequence",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0129 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#237"));
    assert!(docs.contains("#238"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_237() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #237 | Publish/query"));
    assert!(status.contains("RFC-0129"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#237`") && !text.contains("first OPEN `#237`"),
        "NEXT_PROBLEM must not keep #237 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#238`") || text.contains("first OPEN `#238`"),
        "NEXT_PROBLEM must point at first OPEN #238"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_publish_module_contract() {
    assert_eq!(
        aira_peer::RENDEZVOUS_STATE_SCHEMA,
        "aira:peer:rendezvous-state:0.1"
    );
    assert_eq!(aira_peer::RENDEZVOUS_MIN_TTL_SECS, 60);
    assert_eq!(aira_peer::RENDEZVOUS_MAX_TTL_SECS, 7 * 24 * 60 * 60);
    let ttl = aira_peer::presence_ttl_secs("2026-09-05T12:00:00Z", "2026-09-05T13:00:00Z").unwrap();
    assert_eq!(ttl, 3600);
}
