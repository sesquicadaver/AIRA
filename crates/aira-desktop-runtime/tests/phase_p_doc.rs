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
        "first OPEN `#274`",
        "#273",
        "RFC-0163",
        "RFC-0161",
    ] {
        assert!(text.contains(needle), "phase-p-plan missing: {needle}");
    }
}

#[test]
fn phase_p_queue_273_done_274_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-p-plan.md"));
    for n in 266..=273 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(
        !text.contains("| 273 | **OPEN**"),
        "QUEUE #273 must not stay OPEN"
    );
    assert!(
        text.contains("| 274 | **OPEN**"),
        "QUEUE #274 must be first OPEN"
    );
    for needle in [
        "Analyze-308",
        "RFC-0163",
        "RFC-0156",
        "first OPEN `#274`",
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
        "#273",
        "RFC-0163",
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
fn phase_p_rfc_0162_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0162-lifecycle-nonblocking.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0162 missing");
    for needle in ["#272", "update()", "refresh", "Start"] {
        assert!(text.contains(needle), "RFC-0162 missing: {needle}");
    }
}

#[test]
fn phase_p_rfc_0163_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0163-help-f1-routing.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0163 missing");
    for needle in ["#273", "help_search", "focus", "last_problem"] {
        assert!(text.contains(needle), "RFC-0163 missing: {needle}");
    }
}

#[test]
fn phase_p_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-p-plan.md"));
    assert!(readme.contains("#274") || readme.contains("first OPEN"));
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
    assert!(text.contains("#274") || text.contains("перший OPEN"));
    assert!(
        !text.contains("немає OPEN") || text.contains("Phase P"),
        "NEXT_PROBLEM must point at Phase P backlog"
    );
}

#[test]
fn phase_p_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #273 |") || status.contains("#273"));
    assert!(status.contains("RFC-0163") || status.contains("0163"));
    assert!(status.contains("phase_p_doc.rs"));
    assert!(status.contains("phase-p-plan.md"));
}
