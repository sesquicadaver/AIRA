//! Phase M wiring contract (#224). Per-atom tests land with #225–#230.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0117_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0117") || n.contains("rfc-0117"))
        .collect()
}

#[test]
fn phase_m_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-m-plan.md")).unwrap();
    for needle in [
        "Phase M",
        "#224",
        "#225",
        "#230",
        "M0 govern",
        "Landlock",
        "seccomp",
        "network namespace",
        "AIRA-RFC-0117",
        "confirmed free",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
        "AIRA-mediated",
    ] {
        assert!(text.contains(needle), "phase-m-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-m-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_m_queue_wiring_224_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-m-plan.md"),
        "QUEUE missing phase-m-plan"
    );
    assert!(
        text.contains("| 224 | **DONE**"),
        "QUEUE #224 must be DONE after wiring"
    );
    assert!(
        !text.contains("| 224 | **OPEN**"),
        "QUEUE #224 must not stay OPEN after wiring"
    );
    for n in 225..=230 {
        let open = format!("| {n} | **OPEN**");
        let done = format!("| {n} | **DONE**");
        assert!(text.contains(&open), "QUEUE #{n} must be OPEN after wiring");
        assert!(
            !text.contains(&done),
            "QUEUE #{n} must not be DONE at wiring"
        );
    }
    for needle in [
        "M0 govern",
        "Analyze-259",
        "RFC-0117",
        "Landlock",
        "QUEUE L closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_m_rfc_0117_id_free() {
    let hits = rfc_0117_hits();
    assert!(
        hits.is_empty(),
        "RFC-0117 must stay file-free until #230, found {hits:?}"
    );
}

#[test]
fn phase_m_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-m-plan.md"));
    assert!(readme.contains("#224"));
    assert!(readme.contains("#230"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-m-plan.md"));
    assert!(docs.contains("#224"));
    assert!(docs.contains("#230"));
}

#[test]
fn phase_l_points_to_phase_m() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-l-plan.md")).unwrap();
    assert!(text.contains("phase-m-plan.md"));
    assert!(text.contains("#224"));
}

#[test]
fn phase_m_status_row_224() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase M gates"));
    assert!(status.contains("phase_m_doc.rs"));
    assert!(status.contains("| #224 | Phase M wiring + contract"));
    assert!(status.contains("phase-m-plan.md"));
}

#[test]
fn phase_m_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("RESOLVED") || text.contains("provenance"),
        "NEXT_PROBLEM must stay provenance"
    );
    assert!(
        !text.contains("перший OPEN = `#223`"),
        "NEXT_PROBLEM must not keep Phase L first-OPEN pointer"
    );
    assert!(
        text.contains("перший OPEN = `#225`") || text.contains("first OPEN `#225`"),
        "NEXT_PROBLEM must point at first OPEN #225"
    );
    assert!(
        text.contains("QUEUE L closed"),
        "NEXT_PROBLEM must keep QUEUE L closed"
    );
    assert!(
        text.contains("phase-m-plan.md") || text.contains("QUEUE.md"),
        "NEXT_PROBLEM must point at QUEUE / Phase M"
    );
}
