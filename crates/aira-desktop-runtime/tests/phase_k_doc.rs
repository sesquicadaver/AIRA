//! Phase K wiring contract (#209): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_k_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-k-plan.md")).unwrap();
    for needle in [
        "Phase K",
        "#209",
        "#210",
        "#216",
        "K0 govern",
        "text.generate.local",
        "execution-llm",
        "AIRA-RFC-0104",
        "confirmed free",
        "IN PROGRESS",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
    ] {
        assert!(text.contains(needle), "phase-k-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-k-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_k_queue_wiring_209_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-k-plan.md"),
        "QUEUE missing phase-k-plan"
    );
    assert!(
        text.contains("| 209 | **DONE**"),
        "QUEUE #209 must be DONE after wiring"
    );
    for n in 210..=216 {
        let open = format!("| {n} | **OPEN**");
        assert!(text.contains(&open), "QUEUE #{n} must be OPEN after wiring");
        let done = format!("| {n} | **DONE**");
        assert!(
            !text.contains(&done),
            "QUEUE #{n} must not be DONE at wiring"
        );
    }
    assert!(
        !text.contains("| 209 | **OPEN**"),
        "QUEUE #209 must not stay OPEN after wiring"
    );
    for needle in [
        "K0 govern",
        "Analyze-244",
        "RFC-0104",
        "text.generate.local",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_k_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-k-plan.md"));
    assert!(readme.contains("#209"));
    assert!(readme.contains("#210"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-k-plan.md"));
    assert!(docs.contains("#209"));
    assert!(docs.contains("#216"));
}

#[test]
fn phase_j_points_to_active_phase_k() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-j-plan.md")).unwrap();
    assert!(text.contains("phase-k-plan.md"));
    assert!(text.contains("#209"));
}

#[test]
fn phase_k_rfc_0104_id_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0104") || n.contains("rfc-0104"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
}

#[test]
fn phase_k_status_row_209() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase K gates"));
    assert!(status.contains("phase_k_doc.rs"));
    assert!(status.contains("| #209 | Phase K wiring + contract"));
    assert!(status.contains("phase-k-plan.md"));
}
