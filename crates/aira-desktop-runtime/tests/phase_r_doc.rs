//! Phase R contract smoke (#286 wiring … #294 RFC-0174 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_r_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-r-plan.md")).unwrap();
    for needle in [
        "Phase R",
        "#286",
        "#294",
        "actionable Connection",
        "Connection next-step CTA",
        "Promote connect controls",
        "Human-primary",
        "Help connect scenario",
        "Cold-start empty profile",
        "AIRA-RFC-0174",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE Q closed",
        "first OPEN `#288`",
        "#287",
        "GPU marketplace",
        "Calculate 2 + 2",
        "UNKNOWN≠OFFLINE",
    ] {
        assert!(text.contains(needle), "phase-r-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-r-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("IN PROGRESS") || text.contains("**IN PROGRESS**"),
        "phase-r-plan must be IN PROGRESS after wiring"
    );
}

#[test]
fn phase_r_queue_287_done_288_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-r-plan.md"));
    assert!(text.contains("| 286 | **DONE**"), "QUEUE #286 must be DONE");
    assert!(text.contains("| 287 | **DONE**"), "QUEUE #287 must be DONE");
    assert!(
        !text.contains("| 287 | **OPEN**"),
        "QUEUE #287 must not stay OPEN"
    );
    assert!(
        text.contains("| 288 | **OPEN**"),
        "QUEUE #288 must be first OPEN"
    );
    for n in 289..=294 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-323",
        "RFC-0175",
        "RFC-0174",
        "first OPEN `#288`",
        "#287",
        "QUEUE Q closed",
        "desktop-ux.md",
        "Actionable Desktop Connection",
        "connection_cta",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_r_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-r-plan.md",
        "#287",
        "first OPEN `#288`",
        "RFC-0174",
        "RFC-0175",
        "connection_cta",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_r_rfc_0174_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0174") || n.contains("rfc-0174"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0174 must stay file-free until #294; found {hits:?}"
    );
}

#[test]
fn phase_r_rfc_0175_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0175-connection-next-step-cta.md");
    let text = std::fs::read_to_string(&path).unwrap();
    for needle in ["#287", "primary CTA", "connection_cta", "UNKNOWN"] {
        assert!(text.contains(needle), "RFC-0175 missing: {needle}");
    }
}

#[test]
fn phase_r_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-r-plan.md") || readme.contains("Phase R"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-r-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("#288") || docs.contains("`#288`"));
}

#[test]
fn phase_r_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-r-plan.md") || text.contains("Phase R"));
    assert!(text.contains("#288") || text.contains("перший OPEN"));
}

#[test]
fn phase_r_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #287 |") || status.contains("#287"));
    assert!(status.contains("phase_r_doc.rs"));
    assert!(status.contains("phase-r-plan.md"));
    assert!(status.contains("RFC-0175") || status.contains("0175"));
    assert!(status.contains("RFC-0174") || status.contains("0174"));
}

#[test]
fn phase_q_points_to_phase_r_or_closed() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-q-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE Q closed") || text.contains("RFC-0164"),
        "phase-q-plan should remain closed canon"
    );
}

#[test]
fn phase_r_connection_cta_module_present() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/connection_cta.rs"))
            .unwrap();
    for needle in [
        "primary_connection_cta",
        "EnablePrivateNetwork",
        "ImportInvite",
        "StopToApply",
        "RefreshStatus",
        "NoneOk",
        "#287",
    ] {
        assert!(text.contains(needle), "connection_cta missing: {needle}");
    }
}
