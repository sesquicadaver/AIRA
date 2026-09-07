//! Phase O contract smoke (#255 wiring … #265 RFC-0146 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_o_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-o-plan.md")).unwrap();
    for needle in [
        "Phase O",
        "#255",
        "#265",
        "SystemSnapshot",
        "F1",
        "AIRA-RFC-0146",
        "AIRA-RFC-0147",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE N-fix closed",
        "GPU marketplace",
        "Calculate 2 + 2",
        "first OPEN `#257`",
    ] {
        assert!(text.contains(needle), "phase-o-plan missing: {needle}");
    }
}

#[test]
fn phase_o_queue_256_done_257_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-o-plan.md"),
        "QUEUE missing phase-o-plan"
    );
    assert!(text.contains("| 255 | **DONE**"), "QUEUE #255 must be DONE");
    assert!(text.contains("| 256 | **DONE**"), "QUEUE #256 must be DONE");
    assert!(
        !text.contains("| 256 | **OPEN**"),
        "QUEUE #256 must not stay OPEN"
    );
    assert!(
        text.contains("| 257 | **OPEN**"),
        "QUEUE #257 must be first OPEN"
    );
    for n in 258..=265 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-291",
        "RFC-0147",
        "RFC-0146",
        "first OPEN `#257`",
        "desktop-ux.md",
        "QUEUE N-fix closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_o_desktop_ux_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-o-plan.md",
        "Робота",
        "Стан системи",
        "Параметри",
        "Довідка",
        "F1",
        "SystemSnapshot",
        "UNKNOWN",
        "AddressBook",
        "help_id",
        "#255",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_o_rfc_0147_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0147-system-snapshot-honesty.md");
    assert!(path.is_file(), "RFC-0147 missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#256"));
    assert!(text.contains("SystemSnapshot") || text.contains("UNKNOWN"));
    assert!(text.contains("AddressBook") || text.contains("live_session"));
}

#[test]
fn phase_o_rfc_0146_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0146") || n.contains("rfc-0146"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0146 must stay file-free until #265; found {hits:?}"
    );
}

#[test]
fn phase_o_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-o-plan.md"));
    assert!(
        readme.contains("#257") || readme.contains("first OPEN"),
        "README must point at Phase O first OPEN"
    );
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-o-plan.md"));
    assert!(docs.contains("#255") || docs.contains("Phase O"));
}

#[test]
fn phase_n_fix_points_to_phase_o() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-fix-plan.md")).unwrap();
    assert!(text.contains("phase-o-plan.md"));
}

#[test]
fn phase_o_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("phase-o-plan.md") || text.contains("Phase O"),
        "NEXT_PROBLEM must mention Phase O"
    );
    assert!(
        text.contains("QUEUE N-fix closed") || text.contains("no OPEN N-fix"),
        "NEXT_PROBLEM must retain N-fix closed"
    );
    assert!(
        text.contains("#257") || text.contains("перший OPEN"),
        "NEXT_PROBLEM must show first OPEN #257"
    );
    assert!(
        !text.contains("перший OPEN `#256`") && !text.contains("first OPEN `#256`"),
        "NEXT_PROBLEM must not keep #256 as first-OPEN"
    );
}

#[test]
fn phase_o_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #256 |") || status.contains("#256"));
    assert!(status.contains("RFC-0147") || status.contains("0147"));
    assert!(status.contains("phase-o-plan.md") || status.contains("Phase O"));
    assert!(status.contains("phase_o_doc.rs"));
}
