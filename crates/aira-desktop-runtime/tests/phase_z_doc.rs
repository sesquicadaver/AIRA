//! Phase Z contract smoke (#390 wiring … #397 RFC-0244 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_z_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-z-plan.md")).unwrap();
    for needle in [
        "Phase Z",
        "chrome density",
        "progressive disclosure",
        "#390",
        "#397",
        "IN PROGRESS",
        "first OPEN `#391`",
        "AIRA-RFC-0244",
        "confirmed free",
        "desired_rows",
        "Як це працює",
        "conn_boundary",
        "cold_start",
        "Instruct ≫ Explain",
        "network.connect",
        "GPU marketplace",
        "aira-core",
        "public bind",
        "phase-r-plan.md",
        "phase-s-plan.md",
        "desktop-ux.md",
    ] {
        assert!(text.contains(needle), "phase-z-plan missing: {needle}");
    }
    assert!(
        !text.contains("**DONE** @ [AIRA-RFC-0244"),
        "phase-z-plan must not claim RFC-0244 DONE before #397"
    );
    assert!(
        !text.contains("first OPEN `#390`"),
        "phase-z-plan must advance tip past #390"
    );
    assert!(
        !text.contains("QUEUE Z closed"),
        "phase-z-plan must not claim QUEUE Z closed at wiring"
    );
}

#[test]
fn phase_z_queue_390_done_391_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-z-plan.md"));
    assert!(text.contains("| 390 | **DONE**"), "QUEUE #390 must be DONE");
    assert!(text.contains("| 391 | **OPEN**"), "QUEUE #391 must be OPEN");
    for n in 391..=397 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 390 | **OPEN**"),
        "QUEUE #390 must not stay OPEN"
    );
    for needle in [
        "Analyze-398",
        "RFC-0244",
        "phase_z_doc",
        "first OPEN `#391`",
        "**Перший OPEN:** `#391`",
        "chrome density",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("**Перший OPEN:** `#390`"),
        "QUEUE tip must not keep #390 as first-OPEN"
    );
}

#[test]
fn phase_z_rfc_0244_file_free() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0244-phase-z-desktop-chrome-density.md");
    assert!(
        !path.exists(),
        "RFC-0244 must stay file-free until #397 close"
    );
}

#[test]
fn phase_z_desktop_ux_density_contract() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-z-plan.md",
        "#390",
        "#391",
        "RFC-0244",
        "Density / progressive disclosure",
        "Instruct ≫ Explain",
        "перший OPEN `#391`",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_z_wiring_does_not_change_work_layout_yet() {
    let ui = std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(
        ui.contains("desired_rows(4)"),
        "#390 must not implement Work layout (#391)"
    );
    assert!(
        ui.contains("work_how_it_works"),
        "#390 must not remove how-it-works (#392)"
    );
    assert!(
        ui.contains("conn_boundary_guidance"),
        "#390 must not remove boundary chrome (#394)"
    );
}

#[test]
fn phase_z_entry_tips_share_391() {
    let root = repo_root();
    let tip = "перший OPEN `#391`";
    let tip_en = "first OPEN `#391`";
    for rel in [
        "docs/demo.md",
        "docs/crypto.md",
        "docs/implementation-status.md",
        "NEXT_PROBLEM.md",
    ] {
        let text = std::fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            text.contains(tip) || text.contains(tip_en),
            "{rel} must cite live tip {tip} / {tip_en}"
        );
        assert!(
            text.contains("phase-z-plan.md") || text.contains("Phase Z") || text.contains("#391"),
            "{rel} must point at Phase Z / #391"
        );
    }
}

#[test]
fn phase_z_docs_index() {
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-z-plan.md"));
    assert!(docs.contains("#391") || docs.contains("IN PROGRESS"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("phase_z_doc.rs") || status.contains("phase_z_doc"));
    assert!(status.contains("phase-z-plan.md"));
    assert!(status.contains("#391"));
}

#[test]
fn phase_z_analyze_398_present() {
    let root = repo_root();
    for rel in [
        "analysis/Analyze-398/BRIEF.md",
        "analysis/Analyze-398/LIVING_SPEC_MATRIX.md",
        "analysis/Analyze-398/README.md",
        "analysis/Analyze-398/todo/TODO_FIXME.md",
    ] {
        assert!(
            root.join(rel).is_file(),
            "missing Analyze-398 artifact: {rel}"
        );
    }
    let brief = std::fs::read_to_string(root.join("analysis/Analyze-398/BRIEF.md")).unwrap();
    assert!(brief.contains("#390"));
    assert!(brief.contains("#391") || brief.contains("layout"));
}
