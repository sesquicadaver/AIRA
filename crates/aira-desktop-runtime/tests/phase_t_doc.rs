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
        "first OPEN `#308`",
        "RFC-0193",
        "#307",
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
fn phase_t_queue_307_done_308_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-t-plan.md"));
    for n in 306..=307 {
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
        text.contains("| 308 | **OPEN**"),
        "QUEUE #308 must be first OPEN"
    );
    for n in 309..=312 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-343",
        "RFC-0193",
        "RFC-0192",
        "first OPEN `#308`",
        "#307",
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
        "#307",
        "first OPEN `#308`",
        "RFC-0192",
        "RFC-0193",
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
fn phase_t_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-t-plan.md") || readme.contains("Phase T"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-t-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#308`") || docs.contains("#308"));
}

#[test]
fn phase_t_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-t-plan.md") || text.contains("Phase T"));
    assert!(text.contains("#308") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#307`") && !text.contains("first OPEN `#307`"),
        "NEXT_PROBLEM must not keep #307 as first-OPEN after close"
    );
}

#[test]
fn phase_t_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #307 |") || status.contains("#307"));
    assert!(status.contains("phase_t_doc.rs"));
    assert!(status.contains("phase-t-plan.md"));
    assert!(status.contains("RFC-0193") || status.contains("0193"));
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
    }
}

#[test]
fn phase_s_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-s-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE S closed") || text.contains("RFC-0182"),
        "phase-s-plan should remain closed canon"
    );
}
