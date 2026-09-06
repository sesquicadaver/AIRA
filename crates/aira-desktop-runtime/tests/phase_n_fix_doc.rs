//! Phase N-fix contract smoke (#248–#254). `#248` DONE @ RFC-0140; `#249` DONE @ RFC-0141; first OPEN `#250`.

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
fn phase_n_fix_queue_249_done_250_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-fix-plan.md"));
    assert!(text.contains("| 248 | **DONE**"), "QUEUE #248 must be DONE");
    assert!(text.contains("| 249 | **DONE**"), "QUEUE #249 must be DONE");
    for n in 250..=254 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "перший OPEN `#250`",
        "Analyze-285",
        "RFC-0139",
        "RFC-0141",
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
fn phase_n_fix_rfc_0140_and_0141_present() {
    let rfc140 = repo_root().join("specs/rfc/AIRA-RFC-0140-live-evm-rendezvous.md");
    let rfc141 = repo_root().join("specs/rfc/AIRA-RFC-0141-ab-ovo-two-process.md");
    assert!(rfc140.is_file(), "RFC-0140 missing");
    assert!(rfc141.is_file(), "RFC-0141 missing");
    let text = std::fs::read_to_string(&rfc141).unwrap();
    assert!(text.contains("#249"));
    assert!(text.contains("two") || text.contains("Two"));
}

#[test]
fn phase_n_fix_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-fix-plan.md"));
    assert!(readme.contains("first OPEN `#250`") || readme.contains("перший OPEN `#250`"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#250"));
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
        text.contains("перший OPEN = `#250`") || text.contains("first OPEN `#250`"),
        "NEXT_PROBLEM must point at first OPEN #250"
    );
    assert!(
        !text.contains("перший OPEN = `#249`") && !text.contains("first OPEN `#249`"),
        "NEXT_PROBLEM must not keep #249 as first-OPEN"
    );
    assert!(
        text.contains("QUEUE L closed"),
        "NEXT_PROBLEM must retain QUEUE L closed"
    );
}

#[test]
fn phase_n_fix_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #249 |"));
    assert!(status.contains("RFC-0141") || status.contains("two-process"));
    assert!(status.contains("phase-n-fix-plan.md") || status.contains("N-fix"));
}
