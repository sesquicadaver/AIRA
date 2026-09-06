//! Phase N-fix contract smoke (#248–#254). `#248` DONE @ RFC-0140; first OPEN `#249`.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0139_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0139") || n.contains("rfc-0139"))
        .collect()
}

#[test]
fn phase_n_fix_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-fix-plan.md")).unwrap();
    for needle in [
        "Phase N-fix",
        "#248",
        "#254",
        "Live EVM",
        "ab ovo",
        "Reachability",
        "expires_at",
        "next_candidate_port",
        "AIRA-RFC-0139",
        "aira-core",
        "QUEUE N closed",
    ] {
        assert!(text.contains(needle), "phase-n-fix-plan missing: {needle}");
    }
}

#[test]
fn phase_n_fix_queue_248_done_249_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-fix-plan.md"));
    assert!(
        text.contains("| 248 | **DONE**"),
        "QUEUE #248 must be DONE"
    );
    for n in 249..=254 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "перший OPEN `#249`",
        "Analyze-284",
        "RFC-0139",
        "RFC-0140",
        "QUEUE N closed",
        "N-fix",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_n_fix_rfc_0139_id_free() {
    let hits = rfc_0139_hits();
    assert!(
        hits.is_empty(),
        "RFC-0139 must stay file-free until #254, found {hits:?}"
    );
}

#[test]
fn phase_n_fix_rfc_0140_present() {
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0140-live-evm-rendezvous.md");
    assert!(rfc.is_file(), "RFC-0140 missing at {}", rfc.display());
    let text = std::fs::read_to_string(&rfc).unwrap();
    assert!(text.contains("#248"));
    assert!(text.contains("JSON-RPC"));
}

#[test]
fn phase_n_fix_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-fix-plan.md"));
    assert!(readme.contains("first OPEN `#249`") || readme.contains("перший OPEN `#249`"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#249"));
    assert!(docs.contains("phase-n-fix-plan.md"));
}

#[test]
fn phase_n_points_to_n_fix() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-plan.md")).unwrap();
    assert!(text.contains("phase-n-fix-plan.md"));
}

#[test]
fn phase_n_fix_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("перший OPEN = `#249`") || text.contains("first OPEN `#249`"),
        "NEXT_PROBLEM must point at first OPEN #249"
    );
    assert!(
        !text.contains("перший OPEN = `#248`") && !text.contains("first OPEN `#248`"),
        "NEXT_PROBLEM must not keep #248 as first-OPEN"
    );
}

#[test]
fn phase_n_fix_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #248 |"));
    assert!(status.contains("RFC-0140") || status.contains("JSON-RPC"));
    assert!(status.contains("phase-n-fix-plan.md") || status.contains("N-fix"));
}
