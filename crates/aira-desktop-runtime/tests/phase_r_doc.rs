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
        "QUEUE R closed",
        "no OPEN R atoms",
        "**DONE** @",
        "RFC-0181",
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
        !text.contains("first OPEN `#294`"),
        "phase-r-plan must not keep #294 as first-OPEN after close"
    );
}

#[test]
fn phase_r_queue_all_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-r-plan.md"));
    for n in 286..=294 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for needle in [
        "Analyze-330",
        "RFC-0174",
        "RFC-0181",
        "QUEUE R closed",
        "no OPEN R atoms",
        "QUEUE Q closed",
        "desktop-ux.md",
        "Actionable Desktop Connection",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#294`"),
        "QUEUE must not keep #294 as first-OPEN after close"
    );
}

#[test]
fn phase_r_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-r-plan.md",
        "#294",
        "RFC-0174",
        "QUEUE R closed",
        "connection_cta",
        "ui_connect_primary",
        "mesh_language",
        "problem_action",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#294`"),
        "desktop-ux must not keep #294 as first-OPEN after close"
    );
}

#[test]
fn phase_r_rfc_0174_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0174-phase-r-actionable-connection-ux.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0174 missing");
    for needle in [
        "#294",
        "QUEUE R closed",
        "no OPEN R atoms",
        "RFC-0181",
        "RFC-0175",
        "connection_cta",
    ] {
        assert!(text.contains(needle), "RFC-0174 missing: {needle}");
    }
}

#[test]
fn phase_r_rfc_0175_through_0181_present() {
    for (name, needles) in [
        (
            "AIRA-RFC-0175-connection-next-step-cta.md",
            &["#287", "primary CTA", "connection_cta"][..],
        ),
        (
            "AIRA-RFC-0176-promote-connect-controls.md",
            &["#288", "ui_connect_primary", "Technical details"][..],
        ),
        (
            "AIRA-RFC-0177-human-primary-mesh-strip.md",
            &["#289", "mesh_language", "UNKNOWN"][..],
        ),
        (
            "AIRA-RFC-0178-actionable-problem-strip.md",
            &["#290", "problem_action", "next-step"][..],
        ),
        (
            "AIRA-RFC-0179-help-connect-scenario.md",
            &["#291", "invite", "network.connect"][..],
        ),
        (
            "AIRA-RFC-0180-uk-mesh-discovery-parity.md",
            &["#292", "mesh", "discovery"][..],
        ),
        (
            "AIRA-RFC-0181-cold-start-empty-profile.md",
            &["#293", "cold-start", "P0"][..],
        ),
    ] {
        let text = std::fs::read_to_string(repo_root().join("specs/rfc").join(name)).unwrap();
        for needle in needles {
            assert!(text.contains(needle), "{name} missing: {needle}");
        }
    }
}

#[test]
fn phase_r_help_connect_scenario() {
    for rel in [
        "docs/help/en/network.connect.md",
        "docs/help/uk/network.connect.md",
    ] {
        let text = std::fs::read_to_string(repo_root().join(rel)).unwrap();
        for needle in ["P1", "P2", "invite"] {
            assert!(
                text.to_lowercase().contains(&needle.to_lowercase())
                    || text.contains(needle)
                    || (needle == "invite"
                        && (text.contains("запрошення") || text.contains("invite"))),
                "{rel} missing scenario needle: {needle}"
            );
        }
        assert!(
            text.contains("Stop") || text.contains("Стоп"),
            "{rel} must mention Stop/Стоп"
        );
        assert!(
            text.contains("Start") || text.contains("Старт"),
            "{rel} must mention Start/Старт"
        );
        assert!(
            !text.contains("open Technical details first")
                && !text.contains("спочатку розгорніть «Технічні подробиці»"),
            "{rel} must not require Technical details as the primary path"
        );
    }
}

#[test]
fn phase_r_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-r-plan.md") || readme.contains("Phase R"));
    assert!(readme.contains("RFC-0174") || readme.contains("QUEUE R closed"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-r-plan.md"));
    assert!(docs.contains("QUEUE R closed") || docs.contains("DONE @ RFC-0174"));
    assert!(docs.contains("#294") || docs.contains("`#294`"));
    assert!(
        !docs.contains("first OPEN `#294`"),
        "docs index must not keep #294 as first-OPEN after close"
    );
}

#[test]
fn phase_r_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-r-plan.md") || text.contains("Phase R"));
    assert!(text.contains("QUEUE R closed") || text.contains("RFC-0174"));
    assert!(
        !text.contains("перший OPEN `#294`") && !text.contains("first OPEN `#294`"),
        "NEXT_PROBLEM must not keep #294 as first-OPEN after close"
    );
}

#[test]
fn phase_r_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #294 |") || status.contains("#294"));
    assert!(status.contains("phase_r_doc.rs"));
    assert!(status.contains("phase-r-plan.md"));
    assert!(status.contains("RFC-0174") || status.contains("0174"));
    assert!(status.contains("QUEUE R closed"));
    assert!(
        !status.contains("| #294 | RFC-0174 + close | consolidating; QUEUE R closed | **OPEN** |"),
        "status #294 must be DONE"
    );
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

#[test]
fn phase_r_connect_primary_outside_tech() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    let conn = text
        .find("fn ui_sys_connection")
        .expect("ui_sys_connection");
    let primary_call = text[conn..]
        .find("self.ui_connect_primary")
        .expect("ui_connect_primary call");
    let tech = text[conn..]
        .find("sys-connection-tech")
        .expect("sys-connection-tech");
    assert!(
        primary_call < tech,
        "ui_connect_primary must render before Connection Technical details"
    );
    assert!(
        text.contains("fn ui_connect_primary"),
        "ui_connect_primary definition required"
    );
    assert!(
        text.contains("fn ui_network_advanced"),
        "ui_network_advanced keeps P3/federation/discovery under tech"
    );
    assert!(
        !text.contains("fn ui_network_ops"),
        "ui_network_ops must be split for #288"
    );
}

#[test]
fn phase_r_mesh_language_human_primary() {
    let mesh =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/mesh_language.rs"))
            .unwrap();
    for needle in [
        "strip_network_phrase",
        "StripNetworkPhrase",
        "NotChecked",
        "#289",
        "ConnectionConclusion",
    ] {
        assert!(mesh.contains(needle), "mesh_language missing: {needle}");
    }
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(
        ui.contains("strip_network_from_top_level"),
        "strip must use shared mesh_language mapping"
    );
    assert!(
        ui.contains("Phase R `#289`"),
        "mesh status must document human-primary #289"
    );
}

#[test]
fn phase_r_problem_action_human_primary() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/problem_action.rs"))
            .unwrap();
    for needle in [
        "action_next_step",
        "ProblemActionView",
        "strip_work_from_problem",
        "#290",
        "help:",
        "try:",
    ] {
        assert!(text.contains(needle), "problem_action missing: {needle}");
    }
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(
        ui.contains("ProblemActionView::from_problem"),
        "problem footer/events must use ProblemActionView"
    );
    assert!(
        ui.contains("strip_work_from_problem"),
        "status strip must use actionable strip hint"
    );
}

#[test]
fn phase_r_uk_mesh_i18n_parity_source() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    for needle in [
        "uk_mesh_discovery_labels_not_english_shell",
        "Виявлення (P6 Dev)",
        "Ідентичність:",
        "Доступність:",
        "mesh_identity: \"Ідентичність:\"",
        "#292",
    ] {
        assert!(text.contains(needle), "i18n missing: {needle}");
    }
}

#[test]
fn phase_r_cold_start_module_present() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/cold_start.rs")).unwrap();
    for needle in [
        "is_cold_start_empty_profile",
        "cold_start_forbids_connected_claim",
        "#293",
        "P0",
    ] {
        assert!(text.contains(needle), "cold_start missing: {needle}");
    }
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(
        ui.contains("conn_cold_start_guidance") || ui.contains("is_cold_start_empty_profile"),
        "Connection UI must render cold-start guidance"
    );
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(i18n.contains("conn_cold_start_guidance"));
    assert!(i18n.contains("Local-only profile so far"));
}
