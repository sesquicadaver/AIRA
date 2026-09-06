//! Phase N-fix contract smoke (#248–#254). All DONE @ RFC-0140…0145 + RFC-0139; QUEUE N-fix closed.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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
        "QUEUE N-fix closed",
    ] {
        assert!(text.contains(needle), "phase-n-fix-plan missing: {needle}");
    }
}

#[test]
fn phase_n_fix_queue_all_done_closed() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-fix-plan.md"));
    for n in 248..=254 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(
        !text.contains("| 254 | **OPEN**"),
        "QUEUE #254 must not stay OPEN"
    );
    for needle in [
        "QUEUE N-fix closed",
        "no OPEN N-fix",
        "Analyze-289",
        "RFC-0139",
        "RFC-0145",
        "QUEUE N closed",
        "N-fix",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_n_fix_rfc_0139_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0139-phase-n-fix-honesty-close.md");
    assert!(path.is_file(), "RFC-0139 consolidating file missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#254"));
    assert!(text.contains("PARTIAL") || text.contains("honesty"));
    assert!(text.contains("created_at") || text.contains("expires_at"));
}

#[test]
fn phase_n_fix_rfc_0140_through_0145_present() {
    for name in [
        "AIRA-RFC-0140-live-evm-rendezvous.md",
        "AIRA-RFC-0141-ab-ovo-two-process.md",
        "AIRA-RFC-0142-reachability-session-bind.md",
        "AIRA-RFC-0143-presence-expiry-before-promote.md",
        "AIRA-RFC-0144-inbound-firewall-honesty.md",
        "AIRA-RFC-0145-next-candidate-port-wrap.md",
    ] {
        let path = repo_root().join("specs/rfc").join(name);
        assert!(path.is_file(), "missing {name}");
    }
}

#[test]
fn phase_n_fix_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-fix-plan.md"));
    assert!(
        readme.contains("QUEUE N-fix closed") || readme.contains("N-fix `#248`–`#254` **DONE**"),
        "README must show N-fix closed"
    );
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-n-fix-plan.md"));
    assert!(docs.contains("RFC-0139") || docs.contains("#254"));
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
        text.contains("QUEUE N-fix closed") || text.contains("no OPEN N-fix"),
        "NEXT_PROBLEM must show N-fix closed"
    );
    assert!(
        !text.contains("перший OPEN = `#254`") && !text.contains("first OPEN `#254`"),
        "NEXT_PROBLEM must not keep #254 as first-OPEN"
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
    assert!(status.contains("| #254 |"));
    assert!(status.contains("RFC-0139"));
    assert!(status.contains("PARTIAL") || status.contains("honesty"));
    assert!(status.contains("phase-n-fix-plan.md") || status.contains("N-fix"));
}
