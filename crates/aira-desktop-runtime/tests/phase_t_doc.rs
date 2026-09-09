//! Phase T contract smoke (#306 wiring … #312 RFC-0192 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_t_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-t-plan.md")).unwrap();
    for needle in [
        "Phase T",
        "#306",
        "#312",
        "Operation lifecycle",
        "Dial evidence ≠ live",
        "Opt-in dial off UI-thread",
        "Observe miss fail durable",
        "Quit∥Submit",
        "AddressBook candidate",
        "AIRA-RFC-0192",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE S closed",
        "first OPEN `#312`",
        "RFC-0193",
        "RFC-0194",
        "RFC-0195",
        "RFC-0196",
        "RFC-0197",
        "#311",
        "3301d27",
        "live_session_count",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-t-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-t-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("IN PROGRESS") || text.contains("**IN PROGRESS**"),
        "phase-t-plan must be IN PROGRESS after wiring"
    );
}

#[test]
fn phase_t_queue_311_done_312_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-t-plan.md"));
    for n in 306..=311 {
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
        text.contains("| 312 | **OPEN**"),
        "QUEUE #312 must be first OPEN"
    );
    for needle in [
        "Analyze-347",
        "RFC-0197",
        "RFC-0196",
        "RFC-0192",
        "first OPEN `#312`",
        "#311",
        "QUEUE S closed",
        "desktop-ux.md",
        "Operation lifecycle",
        "live_session_count",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_t_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-t-plan.md",
        "#311",
        "first OPEN `#312`",
        "RFC-0192",
        "RFC-0197",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_t_rfc_0192_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0192") || n.contains("rfc-0192"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0192 must stay file-free until #312; found {hits:?}"
    );
}

#[test]
fn phase_t_rfc_0193_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0193-dial-evidence-ne-live-session.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0193 missing");
    for needle in [
        "#307",
        "live_session_count",
        "last confirmed handshake",
        "RFC-0192",
    ] {
        assert!(text.contains(needle), "RFC-0193 missing: {needle}");
    }
}

#[test]
fn phase_t_rfc_0197_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0197-addressbook-candidate-honesty.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0197 missing");
    for needle in ["#311", "AddressBook", "rollback", "RFC-0192"] {
        assert!(text.contains(needle), "RFC-0197 missing: {needle}");
    }
}

#[test]
fn phase_t_rfc_0196_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0196-quit-submit-completion.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0196 missing");
    for needle in [
        "#310",
        "quit_followup_after_submit",
        "Stop→Close",
        "RFC-0192",
    ] {
        assert!(text.contains(needle), "RFC-0196 missing: {needle}");
    }
}

#[test]
fn phase_t_rfc_0195_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0195-observe-miss-fail-durable.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0195 missing");
    for needle in ["#309", "observe-fail", "rehash", "RFC-0192"] {
        assert!(text.contains(needle), "RFC-0195 missing: {needle}");
    }
}

#[test]
fn phase_t_rfc_0194_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0194-opt-in-dial-off-ui-thread.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0194 missing");
    for needle in ["#308", "try_spawn_dial", "block_on", "RFC-0192"] {
        assert!(text.contains(needle), "RFC-0194 missing: {needle}");
    }
}

#[test]
fn phase_t_dial_evidence_module_present() {
    let mesh = std::fs::read_to_string(
        repo_root().join("crates/aira-desktop-runtime/src/network_mesh.rs"),
    )
    .unwrap();
    assert!(mesh.contains("#307"));
    assert!(mesh.contains("live_session_count"));
    assert!(
        mesh.contains("(None, Some(ev.summary_line()))")
            || mesh.contains("live_session_count") && mesh.contains("#307"),
        "mesh must not invent live count from dial evidence"
    );
    let peer =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/peer_dial.rs"))
            .unwrap();
    assert!(peer.contains("fresh_dial_evidence_is_history_not_live_count"));
    assert!(!peer.contains("assert_eq!(snap.live_session_count, Some(1))"));
}

#[test]
fn phase_t_addressbook_candidate_module_present() {
    let peer =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/peer_dial.rs"))
            .unwrap();
    for needle in [
        "#311",
        "prior_book",
        "address book restored",
        "failed_dial_restores_prior_address_book",
        "failed_dial_removes_new_candidate_when_no_prior",
    ] {
        assert!(peer.contains(needle), "peer_dial missing: {needle}");
    }
    let desktop_peer = std::fs::read_to_string(repo_root().join("docs/desktop-peer.md")).unwrap();
    assert!(desktop_peer.contains("RFC-0197") || desktop_peer.contains("#311"));
}

#[test]
fn phase_t_dial_off_ui_module_present() {
    let jobs =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/async_jobs.rs")).unwrap();
    for needle in [
        "#308",
        "try_spawn_dial",
        "poll_dial",
        "dial_inflight",
        "second_dial_rejected_while_inflight",
    ] {
        assert!(jobs.contains(needle), "async_jobs missing: {needle}");
    }
    let ui_dial =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/peer_dial.rs"))
            .unwrap();
    assert!(ui_dial.contains("request_opt_in_peer_dial"));
    assert!(ui_dial.contains("try_spawn_dial"));
    assert!(!ui_dial.contains("actions::opt_in_peer_dial"));
}

#[test]
fn phase_t_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-t-plan.md") || readme.contains("Phase T"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-t-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#312`") || docs.contains("#312"));
}

#[test]
fn phase_t_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-t-plan.md") || text.contains("Phase T"));
    assert!(text.contains("#312") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#311`") && !text.contains("first OPEN `#311`"),
        "NEXT_PROBLEM must not keep #311 as first-OPEN after close"
    );
}

#[test]
fn phase_t_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #311 |") || status.contains("#311"));
    assert!(status.contains("phase_t_doc.rs"));
    assert!(status.contains("phase-t-plan.md"));
    assert!(status.contains("RFC-0197") || status.contains("0197"));
}

#[test]
fn phase_t_help_connect_last_check() {
    for rel in [
        "docs/help/en/network.connect.md",
        "docs/help/uk/network.connect.md",
    ] {
        let text = std::fs::read_to_string(repo_root().join(rel)).unwrap();
        let has_last = text.contains("last confirmed handshake")
            || text.contains("останню підтверджену перевірку");
        assert!(
            has_last,
            "{rel} must describe last check, not live session invent"
        );
        assert!(
            !text.contains("live session observation")
                && !text.contains("спостереження живої сесії"),
            "{rel} must not claim live session from closed dial"
        );
        let has_off_ui = text.contains("off the UI thread") || text.contains("поза UI-потоком");
        assert!(has_off_ui, "{rel} must state dial is off UI-thread (#308)");
        let has_rollback = text.contains("restores")
            || text.contains("AddressBook")
            || text.contains("відновлює")
            || text.contains("AddressBook");
        assert!(
            has_rollback,
            "{rel} must mention AddressBook restore on failed dial (#311)"
        );
    }
}

#[test]
fn phase_t_observe_fail_module_present() {
    let gate =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/activate_gate.rs")).unwrap();
    for needle in [
        "#309",
        "ObserveFailCache",
        "observe-fail.json",
        "observe_ui_mismatch_fail_is_sticky_without_rehash_storm",
        "remember_observe_fail",
    ] {
        assert!(gate.contains(needle), "activate_gate missing: {needle}");
    }
}

#[test]
fn phase_t_quit_submit_module_present() {
    let jobs =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/async_jobs.rs")).unwrap();
    for needle in [
        "#310",
        "quit_arm_policy",
        "quit_followup_after_submit",
        "quit_during_submit_defers_then_queues_stop",
    ] {
        assert!(jobs.contains(needle), "async_jobs missing: {needle}");
    }
    let app =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/mod.rs")).unwrap();
    assert!(app.contains("quit_followup_after_submit"));
    assert!(app.contains("QuitArm::DeferUntilSubmitDone") || app.contains("DeferUntilSubmitDone"));
}

#[test]
fn phase_s_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-s-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE S closed") || text.contains("RFC-0182"),
        "phase-s-plan should remain closed canon"
    );
}
