//! Phase U contract smoke (#313 wiring … #322 RFC-0198 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_u_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-u-plan.md")).unwrap();
    for needle in [
        "Phase U",
        "#313",
        "#322",
        "Cross-path contract",
        "Verification capsule-sourced",
        "Result-by-problem",
        "CLI identity create",
        "Policy audit",
        "Failed submit durable",
        "Submit executor honesty",
        "AddressBook selective rollback",
        "systemd/docs prime-port",
        "AIRA-RFC-0198",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE T closed",
        "first OPEN `#317`",
        "RFC-0199",
        "RFC-0200",
        "RFC-0201",
        "#314",
        "#315",
        "#316",
        "ef5f69c",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-u-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-u-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("IN PROGRESS") || text.contains("**IN PROGRESS**"),
        "phase-u-plan must be IN PROGRESS after wiring"
    );
}

#[test]
fn phase_u_queue_316_done_317_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-u-plan.md"));
    for n in 313..=316 {
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
        text.contains("| 317 | **OPEN**"),
        "QUEUE #317 must be first OPEN"
    );
    for needle in [
        "Analyze-352",
        "RFC-0201",
        "RFC-0200",
        "RFC-0199",
        "RFC-0198",
        "first OPEN `#317`",
        "#316",
        "QUEUE T closed",
        "desktop-ux.md",
        "Cross-path contract",
        "ef5f69c",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_u_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-u-plan.md",
        "#316",
        "first OPEN `#317`",
        "RFC-0198",
        "RFC-0201",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_u_rfc_0198_file_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let hits: Vec<_> = std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0198") || n.contains("rfc-0198"))
        .collect();
    assert!(
        hits.is_empty(),
        "RFC-0198 must stay file-free until #322; found {hits:?}"
    );
}

#[test]
fn phase_u_rfc_0199_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0199-verification-capsule-sourced.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0199 missing");
    for needle in ["#314", "admitted capsule", "VerificationFailed", "RFC-0198"] {
        assert!(text.contains(needle), "RFC-0199 missing: {needle}");
    }
}

#[test]
fn phase_u_rfc_0200_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0200-result-by-problem-authority.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0200 missing");
    for needle in [
        "#315",
        "ArtifactStore",
        "ProblemRecord.result",
        "RFC-0198",
        "locator",
    ] {
        assert!(text.contains(needle), "RFC-0200 missing: {needle}");
    }
}

#[test]
fn phase_u_rfc_0201_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0201-cli-identity-create-fail-closed.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0201 missing");
    for needle in [
        "#316",
        "create_or_ensure_node_identity",
        "CreateExclusive",
        "RFC-0198",
        "fail-closed",
    ] {
        assert!(text.contains(needle), "RFC-0201 missing: {needle}");
    }
}

#[test]
fn phase_u_verification_module_present() {
    let src =
        std::fs::read_to_string(repo_root().join("csu/verification-basic/src/lib.rs")).unwrap();
    for needle in [
        "#314",
        "admitted_capsule",
        "capsule_action_expression",
        "output_matches_capsule",
        "substituted_output_expression_is_not_verified",
        "missing_admitted_capsule_is_not_verified",
    ] {
        assert!(src.contains(needle), "verification-basic missing: {needle}");
    }
}

#[test]
fn phase_u_flow_result_authority_present() {
    let src = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    for needle in [
        "#315",
        "RFC-0200",
        "never trusted alone",
        "verified_artifact_id",
        "execution_artifact_id",
    ] {
        assert!(src.contains(needle), "aira-flow local.rs missing: {needle}");
    }
    let tests = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    for needle in [
        "get_result_by_problem_ignores_tampered_index_cache",
        "get_result_by_problem_rejects_result_without_artifact_ref",
    ] {
        assert!(tests.contains(needle), "aira-flow tests missing: {needle}");
    }
}

#[test]
fn phase_u_identity_create_shared() {
    let src = std::fs::read_to_string(repo_root().join("crates/aira-object/src/crypto/create.rs"))
        .unwrap();
    for needle in [
        "#316",
        "RFC-0201",
        "create_or_ensure_node_identity",
        "CreateExclusive",
        "Ensure",
        "IdentityAlreadyExists",
        "exclusive_create_then_second_create_fails_unchanged",
    ] {
        assert!(
            src.contains(needle),
            "identity create module missing: {needle}"
        );
    }
    let cli = std::fs::read_to_string(repo_root().join("crates/aira-cli/src/commands/identity.rs"))
        .unwrap();
    assert!(
        cli.contains("CreateExclusive"),
        "CLI identity create must use CreateExclusive"
    );
    assert!(
        !cli.contains("std::fs::write(paths.identity_key()"),
        "CLI must not overwrite identity key via fs::write"
    );
}

#[test]
fn phase_u_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-u-plan.md") || readme.contains("Phase U"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-u-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#317`") || docs.contains("#317"));
}

#[test]
fn phase_u_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-u-plan.md") || text.contains("Phase U"));
    assert!(text.contains("#317") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#316`") && !text.contains("first OPEN `#316`"),
        "NEXT_PROBLEM must not keep #316 as first-OPEN after close"
    );
}

#[test]
fn phase_u_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #316 |") || status.contains("#316"));
    assert!(status.contains("phase_u_doc.rs"));
    assert!(status.contains("phase-u-plan.md"));
    assert!(status.contains("RFC-0201") || status.contains("0201"));
}

#[test]
fn phase_t_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-t-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE T closed") || text.contains("RFC-0192"),
        "phase-t-plan should remain closed canon"
    );
}
