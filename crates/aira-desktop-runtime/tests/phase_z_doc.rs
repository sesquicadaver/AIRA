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
        "first OPEN `#394`",
        "Analyze-401",
        "AIRA-RFC-0244",
        "confirmed free",
        "desired_rows",
        "WORK_EDITOR_DESIRED_ROWS",
        "Як це працює",
        "Analyze-400",
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
        !text.contains("first OPEN `#390`")
            && !text.contains("first OPEN `#391`")
            && !text.contains("first OPEN `#392`")
            && !text.contains("first OPEN `#393`"),
        "phase-z-plan must advance tip past #393"
    );
    assert!(
        !text.contains("QUEUE Z closed"),
        "phase-z-plan must not claim QUEUE Z closed before close"
    );
}

#[test]
fn phase_z_queue_393_done_394_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-z-plan.md"));
    for n in 390..=393 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(text.contains("| 394 | **OPEN**"), "QUEUE #394 must be OPEN");
    for n in 394..=397 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 393 | **OPEN**"),
        "QUEUE #393 must not stay OPEN"
    );
    for needle in [
        "Analyze-398",
        "Analyze-399",
        "Analyze-400",
        "Analyze-401",
        "RFC-0244",
        "phase_z_doc",
        "first OPEN `#394`",
        "**Перший OPEN:** `#394`",
        "chrome density",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("**Перший OPEN:** `#393`"),
        "QUEUE tip must not keep #393 as first-OPEN"
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
        "#393",
        "#394",
        "RFC-0244",
        "Density / progressive disclosure",
        "Instruct ≫ Explain",
        "перший OPEN `#394`",
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
}

#[test]
fn phase_z_392_no_how_it_works() {
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(
        !ui.contains("work_how_it_works"),
        "#392 must remove work_how_it_works CollapsingHeader"
    );
    assert!(
        !ui.contains("work-tech-note"),
        "#392 must remove work-tech-note collapsing id"
    );
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(
        !i18n.contains("work_how_it_works"),
        "#392 must drop orphan work_how_it_works i18n"
    );
    assert!(
        !i18n.contains("work_tech_details"),
        "#392 must drop orphan work_tech_details i18n"
    );
    assert!(
        ui.contains("conn_boundary_guidance"),
        "#392/#393 must not remove boundary chrome (#394)"
    );
    let help_en = std::fs::read_to_string(repo_root().join("docs/help/en/work.submit.md")).unwrap();
    let help_uk = std::fs::read_to_string(repo_root().join("docs/help/uk/work.submit.md")).unwrap();
    assert!(
        help_en.contains("Ctrl+Enter") || help_en.to_ascii_lowercase().contains("run"),
        "EN work.submit Help must cover submit path"
    );
    assert!(
        help_uk.contains("Ctrl+Enter") || help_uk.contains("Виконати"),
        "UK work.submit Help must cover submit path"
    );
}

#[test]
fn phase_z_393_short_unknown() {
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(
        i18n.contains("sys_conn_unknown: \"Not checked yet.\""),
        "#393 EN Unknown label must be the short status"
    );
    assert!(
        i18n.contains("sys_conn_unknown: \"Ще не перевірено.\""),
        "#393 UK Unknown label must be the short status"
    );
    assert!(
        !i18n.contains("not the same as offline"),
        "#393 must not keep the long EN Unknown parenthetical on chrome"
    );
    assert!(
        !i18n.contains("це не те саме, що офлайн"),
        "#393 must not keep the long UK Unknown parenthetical on chrome"
    );
    assert!(
        i18n.contains("conn_cold_start_guidance:"),
        "#393 must not remove cold-start guidance (#394)"
    );
    assert!(
        i18n.contains("conn_boundary_guidance:"),
        "#393 must not remove boundary guidance (#394)"
    );
    let help_en =
        std::fs::read_to_string(repo_root().join("docs/help/en/network.connect.md")).unwrap();
    let help_uk =
        std::fs::read_to_string(repo_root().join("docs/help/uk/network.connect.md")).unwrap();
    assert!(
        help_en.contains("UNKNOWN is not the same as OFFLINE"),
        "EN Help must keep UNKNOWN ≠ OFFLINE"
    );
    assert!(
        help_uk.contains("не те саме, що OFFLINE"),
        "UK Help must keep UNKNOWN ≠ OFFLINE"
    );
}

#[test]
fn phase_z_entry_tips_share_394() {
    let root = repo_root();
    let tip = "перший OPEN `#394`";
    let tip_en = "first OPEN `#394`";
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
    }
}

#[test]
fn phase_z_docs_index() {
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-z-plan.md"));
    assert!(docs.contains("#394"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("phase_z_doc"));
    assert!(status.contains("#394"));
}

#[test]
fn phase_z_analyze_400_present() {
    let root = repo_root();
    for rel in [
        "analysis/Analyze-400/BRIEF.md",
        "analysis/Analyze-400/LIVING_SPEC_MATRIX.md",
        "analysis/Analyze-400/README.md",
        "analysis/Analyze-400/todo/TODO_FIXME.md",
    ] {
        assert!(
            root.join(rel).is_file(),
            "missing Analyze-400 artifact: {rel}"
        );
    }
    let brief = std::fs::read_to_string(root.join("analysis/Analyze-400/BRIEF.md")).unwrap();
    assert!(brief.contains("#392"));
}

#[test]
fn phase_z_analyze_401_present() {
    let root = repo_root();
    for rel in [
        "analysis/Analyze-401/BRIEF.md",
        "analysis/Analyze-401/LIVING_SPEC_MATRIX.md",
        "analysis/Analyze-401/README.md",
        "analysis/Analyze-401/todo/TODO_FIXME.md",
    ] {
        assert!(
            root.join(rel).is_file(),
            "missing Analyze-401 artifact: {rel}"
        );
    }
    let brief = std::fs::read_to_string(root.join("analysis/Analyze-401/BRIEF.md")).unwrap();
    assert!(brief.contains("#393"));
}
