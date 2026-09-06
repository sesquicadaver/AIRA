//! Phase N-fix contract smoke (#248–#254). `#248`–`#253` DONE @ RFC-0140…0145; first OPEN `#254`.

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
fn phase_n_fix_queue_253_done_254_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-n-fix-plan.md"));
    for n in 248..=253 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(
        text.contains("| 254 | **OPEN**"),
        "QUEUE #254 must be OPEN"
    );
    for needle in [
        "перший OPEN `#254`",
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
fn phase_n_fix_rfc_0139_id_free() {
    let hits = rfc_0139_hits();
    assert!(
        hits.is_empty(),
        "RFC-0139 must stay file-free until #254, found {hits:?}"
    );
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
    let text = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0145-next-candidate-port-wrap.md"),
    )
    .unwrap();
    assert!(text.contains("#253"));
    assert!(text.contains("wrap") || text.contains("P_AIRA"));
}

#[test]
fn phase_n_fix_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-n-fix-plan.md"));
    assert!(readme.contains("first OPEN `#254`") || readme.contains("перший OPEN `#254`"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("#254"));
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
        text.contains("перший OPEN = `#254`") || text.contains("first OPEN `#254`"),
        "NEXT_PROBLEM must point at first OPEN #254"
    );
    assert!(
        !text.contains("перший OPEN = `#253`") && !text.contains("first OPEN `#253`"),
        "NEXT_PROBLEM must not keep #253 as first-OPEN"
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
    assert!(status.contains("| #253 |"));
    assert!(status.contains("RFC-0145") || status.contains("wrap"));
    assert!(status.contains("phase-n-fix-plan.md") || status.contains("N-fix"));
}
