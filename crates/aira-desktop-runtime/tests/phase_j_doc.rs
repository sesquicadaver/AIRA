//! Phase J wiring contract (#199): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_j_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-j-plan.md")).unwrap();
    for needle in [
        "Phase J",
        "#199",
        "#200",
        "#208",
        "J0 govern + Book II ceiling honesty",
        "object_store_access",
        "B1-010",
        "Book II",
        "AIRA-RFC-0096",
        "confirmed free",
        "**IN PROGRESS**",
        "first OPEN `#200`",
        "GPU marketplace",
    ] {
        assert!(text.contains(needle), "phase-j-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-j-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_j_queue_wiring_199_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-j-plan.md"),
        "QUEUE missing phase-j-plan"
    );
    assert!(
        text.contains("| 199 | **DONE**"),
        "QUEUE #199 must be DONE after wiring"
    );
    assert!(
        text.contains("| 200 | **OPEN**"),
        "QUEUE first OPEN must be #200"
    );
    assert!(
        !text.contains("| 199 | **OPEN**"),
        "QUEUE #199 must not stay OPEN after wiring"
    );
    for n in 201..=208 {
        let open = format!("| {n} | **OPEN**");
        assert!(text.contains(&open), "QUEUE missing {open}");
        let done = format!("| {n} | **DONE**");
        assert!(
            !text.contains(&done),
            "QUEUE {n} must not be DONE at wiring"
        );
    }
    for needle in [
        "J0 govern + Book II ceiling honesty",
        "J1 Book I remainder",
        "RFC-0096",
        "first OPEN `#200`",
        "Analyze-234",
        "Analyze-235",
        "Analyze-236",
        "Analyze-237",
        "Analyze-238",
        "Analyze-239",
        "Analyze-240",
        "Analyze-241",
        "Analyze-242",
        "Analyze-243",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_j_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-j-plan.md"));
    assert!(readme.contains("#208"));
    assert!(readme.contains("#207"));
    assert!(readme.contains("#206"));
    assert!(readme.contains("#205"));
    assert!(readme.contains("#204"));
    assert!(readme.contains("#203"));
    assert!(readme.contains("#202"));
    assert!(readme.contains("#201"));
    assert!(readme.contains("#200"));
    assert!(readme.contains("#199"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-j-plan.md"));
    assert!(docs.contains("#199"));
    assert!(docs.contains("first OPEN `#200`"));
    assert!(docs.contains("#208"));
    assert!(docs.contains("**IN PROGRESS**"));
}

#[test]
fn phase_i_points_to_active_phase_j() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-i-plan.md")).unwrap();
    assert!(text.contains("phase-j-plan.md"));
    assert!(text.contains("#199"));
    assert!(text.contains("first OPEN `#200`"));
}

#[test]
fn phase_j_rfc_0096_id_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    for entry in std::fs::read_dir(&rfc_dir).expect("specs/rfc") {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            !name.starts_with("AIRA-RFC-0096"),
            "RFC-0096 reserved for J close; unexpected file: {name}"
        );
    }
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase J gates"));
    assert!(status.contains("RFC-0096"));
    assert!(status.contains("phase_j_doc.rs"));
    assert!(status.contains("#200"));
    assert!(status.contains("| #199 | Phase J wiring + contract"));
}
