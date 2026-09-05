//! Phase N wiring contract (#231). Per-atom tests land with #232–#247.

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
        "#232",
        "#247",
        "N0 govern",
        "Prime Port",
        "P_AIRA",
        "1491",
        "49157",
        "65521",
        "aira:network:public:v1",
        "EvmRendezvousProvider",
        "Polygon Amoy",
        "DISCOVERED",
        "TRUSTED",
        "AIRA-RFC-0123",
        "confirmed free",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
        "aira-core",
    ] {
        assert!(text.contains(needle), "phase-n-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-n-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_n_queue_wiring_231_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-n-plan.md"),
        "QUEUE missing phase-n-plan"
    );
    assert!(
        text.contains("| 231 | **DONE**"),
        "QUEUE #231 must be DONE after wiring"
    );
    assert!(
        !text.contains("| 231 | **OPEN**"),
        "QUEUE #231 must not stay OPEN after wiring"
    );
    for n in 232..=247 {
        let open = format!("| {n} | **OPEN**");
        let done = format!("| {n} | **DONE**");
        assert!(text.contains(&open), "QUEUE #{n} must be OPEN after #231");
        assert!(!text.contains(&done), "QUEUE #{n} must not be DONE at #231");
    }
    for needle in [
        "N0 govern",
        "Analyze-266",
        "RFC-0123",
        "Prime Port",
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
fn phase_n_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-plan.md"));
    assert!(readme.contains("#231"));
    assert!(readme.contains("#247"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-n-plan.md"));
    assert!(docs.contains("#231"));
    assert!(docs.contains("#247"));
}

#[test]
fn phase_m_points_to_phase_n() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    assert!(text.contains("phase-n-plan.md"));
    assert!(text.contains("#231"));
}

#[test]
fn phase_n_status_row_231() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase N gates"));
    assert!(status.contains("phase_n_doc.rs"));
    assert!(status.contains("| #231 | Phase N wiring + contract"));
    assert!(status.contains("phase-n-plan.md"));
}

#[test]
fn phase_n_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("RESOLVED") || text.contains("provenance"),
        "NEXT_PROBLEM must stay provenance"
    );
    assert!(
        !text.contains("немає OPEN (QUEUE M closed)") && !text.contains("перший OPEN = `#231`"),
        "NEXT_PROBLEM must not keep M-closed-only or #231 as first-OPEN after wiring"
    );
    assert!(
        text.contains("перший OPEN = `#232`") || text.contains("first OPEN `#232`"),
        "NEXT_PROBLEM must point at first OPEN #232"
    );
    assert!(
        text.contains("QUEUE M closed"),
        "NEXT_PROBLEM must keep QUEUE M closed"
    );
    assert!(
        text.contains("phase-n-plan.md") || text.contains("QUEUE.md"),
        "NEXT_PROBLEM must point at QUEUE / Phase N"
    );
}
