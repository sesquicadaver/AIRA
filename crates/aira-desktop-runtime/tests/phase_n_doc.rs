//! Phase N contract smoke (#231–#238). Per-atom tests land with #239–#247.

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
        "#238",
        "#247",
        "Reachability Probe",
        "EvmRendezvousProvider",
        "P_AIRA",
        "1491",
        "AIRA-RFC-0123",
        "confirmed free",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_238_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=238 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 239..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #238"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #238"
        );
    }
    for needle in [
        "N7 Reachability Probe",
        "Analyze-273",
        "RFC-0130",
        "перший OPEN `#239`",
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
fn phase_n_rfc_0130_probe_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0130-reachability-probe.md"))
            .unwrap();
    for needle in [
        "ReachabilityChallenge",
        "hairpin",
        "#238",
        "#239",
        "probe",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0130 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#238"));
    assert!(docs.contains("#239"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_238() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #238 | Reachability Probe"));
    assert!(status.contains("RFC-0130"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#238`") && !text.contains("first OPEN `#238`"),
        "NEXT_PROBLEM must not keep #238 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#239`") || text.contains("first OPEN `#239`"),
        "NEXT_PROBLEM must point at first OPEN #239"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_reachability_module_contract() {
    assert_eq!(
        aira_peer::REACHABILITY_CHALLENGE_SCHEMA,
        "aira:schema:peer:reachability-challenge:0.1"
    );
    assert_eq!(
        aira_peer::REACHABILITY_RESULT_SCHEMA,
        "aira:schema:peer:reachability-result:0.1"
    );
    let path = repo_root().join("schemas/peer/reachability-challenge.schema.json");
    assert!(path.is_file(), "missing reachability challenge schema");
}
