//! Phase S contract smoke (#295 wiring … #305 RFC-0182 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_s_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-s-plan.md")).unwrap();
    for needle in [
        "Phase S",
        "#295",
        "#305",
        "Cross-boundary integrity",
        "Reachability key-bound",
        "Model descriptor root-scoped",
        "Identity incomplete-pair",
        "Connect Help boundary",
        "Opt-in peer dial",
        "CTA Stop→Start",
        "AIRA-RFC-0182",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE R closed",
        "first OPEN `#301`",
        "#300",
        "RFC-0187",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-s-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-s-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("IN PROGRESS") || text.contains("**IN PROGRESS**"),
        "phase-s-plan must be IN PROGRESS after wiring"
    );
}

#[test]
fn phase_s_queue_300_done_301_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-s-plan.md"));
    for n in 295..=300 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(
        !text.contains("| 300 | **OPEN**"),
        "QUEUE #300 must not stay OPEN"
    );
    assert!(
        text.contains("| 301 | **OPEN**"),
        "QUEUE #301 must be first OPEN"
    );
    for n in 302..=305 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-336",
        "RFC-0187",
        "RFC-0182",
        "first OPEN `#301`",
        "#300",
        "QUEUE R closed",
        "desktop-ux.md",
        "Cross-boundary integrity",
        "RFC-0187",
        "opt-in",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_s_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-s-plan.md",
        "#300",
        "first OPEN `#301`",
        "RFC-0182",
        "RFC-0187",
        "run_opt_in_peer_dial",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_s_rfc_0182_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0182") || n.contains("rfc-0182"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0182 must stay file-free until #305; found {hits:?}"
    );
}

#[test]
fn phase_s_rfc_0183_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0183-reachability-key-bound-admission.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0183 missing");
    for needle in ["#296", "target_public_key", "foreign-key", "RFC-0182"] {
        assert!(text.contains(needle), "RFC-0183 missing: {needle}");
    }
}

#[test]
fn phase_s_rfc_0184_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0184-model-descriptor-root-scoped-verify.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0184 missing");
    for needle in [
        "#297",
        "ArtifactDescriptor",
        "bind_thread_crypto",
        "RFC-0182",
    ] {
        assert!(text.contains(needle), "RFC-0184 missing: {needle}");
    }
}

#[test]
fn phase_s_rfc_0185_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0185-identity-incomplete-pair-fail-closed.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0185 missing");
    for needle in ["#298", "incomplete", "local.ed25519", "RFC-0182"] {
        assert!(text.contains(needle), "RFC-0185 missing: {needle}");
    }
}

#[test]
fn phase_s_rfc_0186_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0186-connect-help-boundary-honesty.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0186 missing");
    for needle in [
        "#299",
        "setup ≠ remote",
        "loopback",
        "dial",
        "RFC-0182",
        "conn_boundary_guidance",
    ] {
        assert!(text.contains(needle), "RFC-0186 missing: {needle}");
    }
}

#[test]
fn phase_s_rfc_0187_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0187-opt-in-peer-dial-session-evidence.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0187 missing");
    for needle in [
        "#300",
        "run_opt_in_peer_dial",
        "dial_session_evidence",
        "RFC-0182",
        "DIRECT",
    ] {
        assert!(text.contains(needle), "RFC-0187 missing: {needle}");
    }
}

#[test]
fn phase_s_help_connect_boundary() {
    for rel in [
        "docs/help/en/network.connect.md",
        "docs/help/uk/network.connect.md",
    ] {
        let text = std::fs::read_to_string(repo_root().join(rel)).unwrap();
        let has_setup = text.contains("Setup ≠ remote")
            || text.contains("setup ≠ remote")
            || text.contains("Налаштування ≠ віддалена");
        let has_loopback = text.contains("Loopback ≠")
            || text.contains("loopback ≠")
            || text.contains("Loopback ≠ адреса");
        assert!(has_setup, "{rel} must state setup ≠ remote session");
        assert!(has_loopback, "{rel} must state loopback ≠ dial");
        assert!(
            text.contains("Boundary") || text.contains("Межа"),
            "{rel} must have Boundary/Межа section"
        );
        assert!(
            text.contains("Peer dial") || text.contains("Dial до піра") || text.contains("opt-in"),
            "{rel} must mention opt-in dial path"
        );
    }
}

#[test]
fn phase_s_cta_boundary_module_present() {
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    for needle in [
        "#299",
        "conn_boundary_guidance",
        "Setup is not a remote session",
        "peer_dial",
        "#300",
    ] {
        assert!(i18n.contains(needle), "i18n missing: {needle}");
    }
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    for needle in ["#299", "conn_boundary_guidance", "peer_dial", "dial_run"] {
        assert!(ui.contains(needle), "ui missing: {needle}");
    }
}

#[test]
fn phase_s_peer_dial_module_present() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/peer_dial.rs"))
            .unwrap();
    for needle in [
        "#300",
        "run_opt_in_peer_dial",
        "DialSessionEvidence",
        "explicit dial address",
        "opt_in_dial_persists_evidence_without_direct_status",
    ] {
        assert!(text.contains(needle), "peer_dial missing: {needle}");
    }
    let peer_doc = std::fs::read_to_string(repo_root().join("docs/desktop-peer.md")).unwrap();
    assert!(peer_doc.contains("#300") || peer_doc.contains("RFC-0187"));
}

#[test]
fn phase_s_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-s-plan.md") || readme.contains("Phase S"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-s-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#301`") || docs.contains("#301"));
}

#[test]
fn phase_s_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-s-plan.md") || text.contains("Phase S"));
    assert!(text.contains("#301") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#300`") && !text.contains("first OPEN `#300`"),
        "NEXT_PROBLEM must not keep #300 as first-OPEN after close"
    );
}

#[test]
fn phase_s_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #300 |") || status.contains("#300"));
    assert!(status.contains("phase_s_doc.rs"));
    assert!(status.contains("phase-s-plan.md"));
    assert!(status.contains("RFC-0187") || status.contains("0187"));
}

#[test]
fn phase_s_key_bound_module_present() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-peer/src/reachability_state.rs"))
            .unwrap();
    for needle in [
        "#296",
        "target_public_key = current root key",
        "apply_rejects_same_identity_foreign_key_package",
    ] {
        assert!(
            text.contains(needle),
            "reachability_state missing: {needle}"
        );
    }
}

#[test]
fn phase_s_descriptor_reopen_module_present() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/activate_gate.rs")).unwrap();
    for needle in [
        "#297",
        "bind_thread_crypto",
        "verification_crypto",
        "AIRA_297_REOPEN_CHILD",
        "install_disk_identity_activate_fixture",
    ] {
        assert!(text.contains(needle), "activate_gate missing: {needle}");
    }
}

#[test]
fn phase_s_identity_incomplete_pair_module_present() {
    let text =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/bootstrap.rs"))
            .unwrap();
    for needle in [
        "#298",
        "identity incomplete",
        "write_secret_create_new",
        "incomplete_identity_pair_is_fail_closed",
    ] {
        assert!(text.contains(needle), "bootstrap missing: {needle}");
    }
}

#[test]
fn phase_r_points_to_phase_s_or_closed() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-r-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE R closed") || text.contains("RFC-0174"),
        "phase-r-plan should remain closed canon"
    );
}
