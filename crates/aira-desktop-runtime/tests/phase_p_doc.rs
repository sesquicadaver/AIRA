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
        "first OPEN `#267`",
    ] {
        assert!(text.contains(needle), "phase-p-plan missing: {needle}");
    }
}

#[test]
fn phase_p_queue_266_done_267_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-p-plan.md"));
    assert!(text.contains("| 266 | **DONE**"), "QUEUE #266 must be DONE");
    assert!(
        !text.contains("| 266 | **OPEN**"),
        "QUEUE #266 must not stay OPEN"
    );
    assert!(
        text.contains("| 267 | **OPEN**"),
        "QUEUE #267 must be first OPEN"
    );
    for n in 268..=274 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-301",
        "RFC-0156",
        "first OPEN `#267`",
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
    for needle in ["phase-p-plan.md", "#266", "RFC-0156", "runtime-honest"] {
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
fn phase_p_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-p-plan.md"));
    assert!(readme.contains("#267") || readme.contains("first OPEN"));
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
    assert!(text.contains("#267") || text.contains("перший OPEN"));
    assert!(
        !text.contains("немає OPEN") || text.contains("Phase P"),
        "NEXT_PROBLEM must point at Phase P backlog"
    );
}

#[test]
fn phase_p_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #266 |") || status.contains("#266"));
    assert!(status.contains("phase_p_doc.rs"));
    assert!(status.contains("phase-p-plan.md"));
}
