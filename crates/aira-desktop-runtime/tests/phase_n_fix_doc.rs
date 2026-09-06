//! Phase N-fix contract smoke (#248–#254). `#248`–`#251` DONE @ RFC-0140…0143; first OPEN `#252`.

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
fn phase_n_fix_queue_251_done_252_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-fix-plan.md"));
    for n in 248..=251 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    for n in 252..=254 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "перший OPEN `#252`",
        "Analyze-287",
        "RFC-0139",
        "RFC-0143",
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
fn phase_n_fix_rfc_0140_through_0143_present() {
    for name in [
        "AIRA-RFC-0140-live-evm-rendezvous.md",
        "AIRA-RFC-0141-ab-ovo-two-process.md",
        "AIRA-RFC-0142-reachability-session-bind.md",
        "AIRA-RFC-0143-presence-expiry-before-promote.md",
    ] {
        let path = repo_root().join("specs/rfc").join(name);
        assert!(path.is_file(), "missing {name}");
    }
    let text = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0143-presence-expiry-before-promote.md"),
    )
    .unwrap();
    assert!(text.contains("#251"));
    assert!(text.contains("expir") || text.contains("Expir"));
}

#[test]
fn phase_n_fix_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-fix-plan.md"));
    assert!(readme.contains("first OPEN `#252`") || readme.contains("перший OPEN `#252`"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#252"));
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
        text.contains("перший OPEN = `#252`") || text.contains("first OPEN `#252`"),
        "NEXT_PROBLEM must point at first OPEN #252"
    );
    assert!(
        !text.contains("перший OPEN = `#251`") && !text.contains("first OPEN `#251`"),
        "NEXT_PROBLEM must not keep #251 as first-OPEN"
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
    assert!(status.contains("| #251 |"));
    assert!(status.contains("RFC-0143") || status.contains("expir"));
    assert!(status.contains("phase-n-fix-plan.md") || status.contains("N-fix"));
}
