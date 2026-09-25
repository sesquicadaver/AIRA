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
        "first OPEN `#392`",
        "AIRA-RFC-0244",
        "confirmed free",
        "desired_rows",
        "WORK_EDITOR_DESIRED_ROWS",
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
        !text.contains("first OPEN `#390`") && !text.contains("first OPEN `#391`"),
        "phase-z-plan must advance tip past #391"
    );
    assert!(
        !text.contains("QUEUE Z closed"),
        "phase-z-plan must not claim QUEUE Z closed before close"
    );
}

#[test]
fn phase_z_queue_391_done_392_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-z-plan.md"));
    assert!(text.contains("| 390 | **DONE**"), "QUEUE #390 must be DONE");
    assert!(text.contains("| 391 | **DONE**"), "QUEUE #391 must be DONE");
    assert!(text.contains("| 392 | **OPEN**"), "QUEUE #392 must be OPEN");
    for n in 392..=397 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 391 | **OPEN**"),
        "QUEUE #391 must not stay OPEN"
    );
    for needle in [
        "Analyze-398",
        "Analyze-399",
        "RFC-0244",
        "phase_z_doc",
        "first OPEN `#392`",
        "**Перший OPEN:** `#392`",
        "WORK_EDITOR_DESIRED_ROWS",
        "chrome density",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("**Перший OPEN:** `#391`"),
        "QUEUE tip must not keep #391 as first-OPEN"
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
        "#392",
        "RFC-0244",
        "Density / progressive disclosure",
        "Instruct ≫ Explain",
        "перший OPEN `#392`",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_z_391_work_editor_budget() {
    let work =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/work.rs")).unwrap();
    assert!(
        work.contains("WORK_EDITOR_DESIRED_ROWS: usize = 8"),
        "#391 must define WORK_EDITOR_DESIRED_ROWS >= 8"
    );
    assert!(
        work.contains("WORK_ANSWER_MIN_HEIGHT: f32 = 160.0"),
        "#391 must keep answer min height constant"
    );
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(
        ui.contains("work::WORK_EDITOR_DESIRED_ROWS"),
        "ui_work must use WORK_EDITOR_DESIRED_ROWS"
    );
    assert!(
        !ui.contains("desired_rows(4)"),
        "ui_work must not keep the 4-row editor budget"
    );
    assert!(
        ui.contains("work::WORK_ANSWER_MIN_HEIGHT"),
        "answer ScrollArea must use WORK_ANSWER_MIN_HEIGHT"
    );
    // #392 not in this atom.
    assert!(
        ui.contains("work_how_it_works"),
        "#391 must not remove how-it-works (#392)"
    );
    assert!(
        ui.contains("conn_boundary_guidance"),
        "#391 must not remove boundary chrome (#394)"
    );
}

#[test]
fn phase_z_entry_tips_share_392() {
    let root = repo_root();
    let tip = "перший OPEN `#392`";
    let tip_en = "first OPEN `#392`";
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
            text.contains("phase-z-plan.md") || text.contains("Phase Z") || text.contains("#392"),
            "{rel} must point at Phase Z / #392"
        );
    }
}

#[test]
fn phase_z_docs_index() {
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-z-plan.md"));
    assert!(docs.contains("#392") || docs.contains("IN PROGRESS"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("phase_z_doc.rs") || status.contains("phase_z_doc"));
    assert!(status.contains("phase-z-plan.md"));
    assert!(status.contains("#392"));
    assert!(status.contains("WORK_EDITOR_DESIRED_ROWS") || status.contains("#391"));
}

#[test]
fn phase_z_analyze_399_present() {
    let root = repo_root();
    for rel in [
        "analysis/Analyze-399/BRIEF.md",
        "analysis/Analyze-399/LIVING_SPEC_MATRIX.md",
        "analysis/Analyze-399/README.md",
        "analysis/Analyze-399/todo/TODO_FIXME.md",
    ] {
        assert!(
            root.join(rel).is_file(),
            "missing Analyze-399 artifact: {rel}"
        );
    }
    let brief = std::fs::read_to_string(root.join("analysis/Analyze-399/BRIEF.md")).unwrap();
    assert!(brief.contains("#391"));
    assert!(brief.contains("WORK_EDITOR") || brief.contains("layout"));
}
