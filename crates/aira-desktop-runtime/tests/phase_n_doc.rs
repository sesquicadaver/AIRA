//! Phase N contract smoke (#231–#243). Per-atom tests land with #244–#247.

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
        "#243",
        "#247",
        "CLI",
        "P_AIRA",
        "AIRA-RFC-0123",
        "confirmed free",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
}

#[test]
fn phase_n_queue_243_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    for n in 231..=243 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for n in 244..=247 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN after #243"
        );
        assert!(
            !text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must not be DONE at #243"
        );
    }
    for needle in [
        "N12 CLI",
        "Analyze-278",
        "RFC-0135",
        "перший OPEN `#244`",
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
fn phase_n_rfc_0135_cli_present() {
    let text =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0135-peer-cli-phase-n.md"))
            .unwrap();
    for needle in [
        "peer port",
        "reachability",
        "rendezvous",
        "LocalFileRendezvousProvider",
        "#243",
        "#244",
        "aira-core",
    ] {
        assert!(text.contains(needle), "RFC-0135 missing: {needle}");
    }
}

#[test]
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#243"));
    assert!(docs.contains("#244"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_status_row_243() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #243 | CLI"));
    assert!(status.contains("RFC-0135"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        !text.contains("перший OPEN = `#243`") && !text.contains("first OPEN `#243`"),
        "NEXT_PROBLEM must not keep #243 as first-OPEN"
    );
    assert!(
        text.contains("перший OPEN = `#244`") || text.contains("first OPEN `#244`"),
        "NEXT_PROBLEM must point at first OPEN #244"
    );
    assert!(text.contains("QUEUE M closed"));
}

#[test]
fn phase_n_local_file_rendezvous_kind() {
    assert_eq!(aira_peer::RENDEZVOUS_KIND_LOCAL_FILE, "local-file");
}
