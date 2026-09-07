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
        "AIRA-RFC-0150",
        "AIRA-RFC-0151",
        "AIRA-RFC-0152",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE N-fix closed",
        "GPU marketplace",
        "Calculate 2 + 2",
        "first OPEN `#262`",
    ] {
        assert!(text.contains(needle), "phase-o-plan missing: {needle}");
    }
}

#[test]
fn phase_o_queue_261_done_262_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-o-plan.md"));
    for n in 255..=261 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    assert!(
        text.contains("| 262 | **OPEN**"),
        "QUEUE #262 must be first OPEN"
    );
    for n in 263..=265 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-296",
        "RFC-0152",
        "RFC-0151",
        "RFC-0146",
        "first OPEN `#262`",
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
        "lexicon",
        "#261",
        "RFC-0152",
        "Програма",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_o_rfc_0150_present() {
    assert!(repo_root()
        .join("specs/rfc/AIRA-RFC-0150-shell-ia-help-f1.md")
        .is_file());
}

#[test]
fn phase_o_rfc_0151_present() {
    assert!(repo_root()
        .join("specs/rfc/AIRA-RFC-0151-work-screen-provenance.md")
        .is_file());
}

#[test]
fn phase_o_rfc_0152_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0152-system-status-screen.md");
    assert!(path.is_file(), "RFC-0152 missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#261"));
    assert!(text.contains("Program") || text.contains("Connection"));
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
    assert!(readme.contains("#262") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-o-plan.md"));
}

#[test]
fn phase_n_fix_points_to_phase_o() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-fix-plan.md")).unwrap();
    assert!(text.contains("phase-o-plan.md"));
}

#[test]
fn phase_o_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-o-plan.md") || text.contains("Phase O"));
    assert!(text.contains("QUEUE N-fix closed") || text.contains("no OPEN N-fix"));
    assert!(text.contains("#262") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#261`") && !text.contains("first OPEN `#261`"),
        "NEXT_PROBLEM must not keep #261 as first-OPEN"
    );
}

#[test]
fn phase_o_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #261 |") || status.contains("#261"));
    assert!(status.contains("RFC-0152") || status.contains("0152"));
    assert!(status.contains("phase_o_doc.rs"));
}
