//! Phase P contract smoke (#266 wiring … #274 RFC-0156 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_p_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-p-plan.md")).unwrap();
    for needle in [
        "Phase P",
        "#266",
        "#274",
        "runtime-honest",
        "DataQuality",
        "Applied",
        "Model triple",
        "Reachability",
        "AIRA-RFC-0156",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE O closed",
        "GPU marketplace",
        "Calculate 2 + 2",
        "first OPEN `#268`",
        "#267",
        "RFC-0157",
    ] {
        assert!(text.contains(needle), "phase-p-plan missing: {needle}");
    }
}

#[test]
fn phase_p_queue_267_done_268_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-p-plan.md"));
    assert!(text.contains("| 266 | **DONE**"), "QUEUE #266 must be DONE");
    assert!(text.contains("| 267 | **DONE**"), "QUEUE #267 must be DONE");
    assert!(
        !text.contains("| 267 | **OPEN**"),
        "QUEUE #267 must not stay OPEN"
    );
    assert!(
        text.contains("| 268 | **OPEN**"),
        "QUEUE #268 must be first OPEN"
    );
    for n in 269..=274 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-302",
        "RFC-0157",
        "RFC-0156",
        "first OPEN `#268`",
        "QUEUE O closed",
        "desktop-ux.md",
        "aira-current.md",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_p_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-p-plan.md",
        "#267",
        "RFC-0157",
        "RFC-0156",
        "runtime-honest",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_p_rfc_0156_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0156") || n.contains("rfc-0156"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0156 must stay file-free until #274; found {hits:?}"
    );
}

#[test]
fn phase_p_rfc_0157_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0157-snapshot-freshness.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0157 missing");
    for needle in ["#267", "Stale", "loaded_at", "observed_at", "listener"] {
        assert!(text.contains(needle), "RFC-0157 missing: {needle}");
    }
}

#[test]
fn phase_p_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-p-plan.md"));
    assert!(readme.contains("#268") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-p-plan.md"));
}

#[test]
fn phase_o_points_to_phase_p() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-o-plan.md")).unwrap();
    assert!(text.contains("phase-p-plan.md") || text.contains("Phase P"));
}

#[test]
fn phase_p_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-p-plan.md") || text.contains("Phase P"));
    assert!(text.contains("#268") || text.contains("перший OPEN"));
    assert!(
        !text.contains("немає OPEN") || text.contains("Phase P"),
        "NEXT_PROBLEM must point at Phase P backlog"
    );
}

#[test]
fn phase_p_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #267 |") || status.contains("#267"));
    assert!(status.contains("RFC-0157") || status.contains("0157"));
    assert!(status.contains("phase_p_doc.rs"));
    assert!(status.contains("phase-p-plan.md"));
}
