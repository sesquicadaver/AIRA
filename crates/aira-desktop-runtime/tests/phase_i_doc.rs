//! Phase I wiring contract (#184): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_i_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-i-plan.md")).unwrap();
    for needle in [
        "Phase I",
        "#184",
        "#185",
        "#198",
        "I0 govern + status honesty",
        "Handle integrity",
        "Semantic verify",
        "PolicyGate",
        "Durable reuse",
        "AIRA-RFC-0078",
        "confirmed free",
        "IN PROGRESS",
        "first OPEN `#185`",
        "GPU marketplace",
    ] {
        assert!(text.contains(needle), "phase-i-plan missing: {needle}");
    }
}

#[test]
fn phase_i_queue_wiring_184_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-i-plan.md"),
        "QUEUE missing phase-i-plan"
    );
    assert!(
        text.contains("| 184 | **DONE**"),
        "QUEUE #184 must be DONE after wiring"
    );
    assert!(
        text.contains("| 185 | **OPEN**"),
        "QUEUE #185 must be first remaining OPEN"
    );
    assert!(
        !text.contains("| 184 | **OPEN**"),
        "QUEUE #184 must not stay OPEN after wiring"
    );
    for n in 186..=198 {
        let needle = format!("| {n} | **OPEN**");
        assert!(text.contains(&needle), "QUEUE missing OPEN row: {needle}");
    }
    for needle in [
        "I0 govern + status honesty",
        "I1 P0 Core/CSU semantics",
        "RFC-0078",
        "Analyze-219",
        "Analyze-220",
        "Analyze-233",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_i_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-i-plan.md"));
    assert!(readme.contains("#185"));
    assert!(readme.contains("#184"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-i-plan.md"));
    assert!(docs.contains("#184"));
    assert!(docs.contains("IN PROGRESS"));
    assert!(docs.contains("#185"));
}

#[test]
fn phase_h_points_to_active_phase_i() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-h-plan.md")).unwrap();
    assert!(text.contains("phase-i-plan.md"));
    assert!(text.contains("#184"));
    assert!(text.contains("first OPEN `#185`"));
}

#[test]
fn phase_i_rfc_0078_id_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let entries = std::fs::read_dir(&rfc_dir).expect("specs/rfc");
    for entry in entries {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            !name.starts_with("AIRA-RFC-0078"),
            "RFC-0078 reserved for #198; unexpected file: {name}"
        );
    }
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase I gates"));
    assert!(status.contains("RFC-0078"));
    assert!(status.contains("phase_i_doc.rs"));
    assert!(status.contains("#185"));
}
