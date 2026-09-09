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
        "first OPEN `#298`",
        "#297",
        "RFC-0184",
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
fn phase_s_queue_297_done_298_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-s-plan.md"));
    assert!(text.contains("| 295 | **DONE**"), "QUEUE #295 must be DONE");
    assert!(text.contains("| 296 | **DONE**"), "QUEUE #296 must be DONE");
    assert!(text.contains("| 297 | **DONE**"), "QUEUE #297 must be DONE");
    assert!(
        !text.contains("| 297 | **OPEN**"),
        "QUEUE #297 must not stay OPEN"
    );
    assert!(
        text.contains("| 298 | **OPEN**"),
        "QUEUE #298 must be first OPEN"
    );
    for n in 299..=305 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    for needle in [
        "Analyze-333",
        "RFC-0184",
        "RFC-0182",
        "first OPEN `#298`",
        "#297",
        "QUEUE R closed",
        "desktop-ux.md",
        "Cross-boundary integrity",
        "descriptor",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_s_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-s-plan.md",
        "#297",
        "first OPEN `#298`",
        "RFC-0182",
        "RFC-0184",
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
fn phase_s_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-s-plan.md") || readme.contains("Phase S"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-s-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
}

#[test]
fn phase_s_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-s-plan.md") || text.contains("Phase S"));
    assert!(text.contains("#298") || text.contains("перший OPEN"));
}

#[test]
fn phase_s_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #297 |") || status.contains("#297"));
    assert!(status.contains("phase_s_doc.rs"));
    assert!(status.contains("phase-s-plan.md"));
    assert!(status.contains("RFC-0184") || status.contains("0184"));
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
fn phase_r_points_to_phase_s_or_closed() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-r-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE R closed") || text.contains("RFC-0174"),
        "phase-r-plan should remain closed canon"
    );
}
