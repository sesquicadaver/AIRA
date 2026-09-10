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
        "first OPEN `#321`",
        "RFC-0199",
        "RFC-0200",
        "RFC-0201",
        "RFC-0202",
        "RFC-0203",
        "RFC-0204",
        "RFC-0205",
        "#314",
        "#315",
        "#316",
        "#317",
        "#318",
        "#319",
        "#320",
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
fn phase_u_queue_320_done_321_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-u-plan.md"));
    for n in 313..=320 {
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
        text.contains("| 321 | **OPEN**"),
        "QUEUE #321 must be first OPEN"
    );
    for needle in [
        "Analyze-356",
        "RFC-0205",
        "Analyze-355",
        "RFC-0204",
        "Analyze-354",
        "RFC-0203",
        "RFC-0202",
        "RFC-0201",
        "RFC-0200",
        "RFC-0199",
        "RFC-0198",
        "first OPEN `#321`",
        "#320",
        "#319",
        "#318",
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
        "#318",
        "first OPEN `#321`",
        "RFC-0198",
        "RFC-0203",
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
fn phase_u_rfc_0202_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0202-policy-audit-uniqueness.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0202 missing");
    for needle in [
        "#317",
        "PolicyEvaluated",
        "same-id",
        "fail-closed",
        "RFC-0198",
    ] {
        assert!(text.contains(needle), "RFC-0202 missing: {needle}");
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
fn phase_u_policy_audit_uniqueness_present() {
    let gate = std::fs::read_to_string(repo_root().join("crates/aira-policy/src/gate.rs")).unwrap();
    for needle in ["#317", "with_run_nonce", "aira:event:policy", "run_nonce"] {
        assert!(gate.contains(needle), "policy gate missing: {needle}");
    }
    let local = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    for needle in [
        "admit_persisted_event",
        "same-id different hash #317",
        "RFC-0202",
    ] {
        assert!(local.contains(needle), "flow persist missing: {needle}");
    }
    let tests = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    for needle in [
        "two_submits_persist_distinct_policy_event_ids",
        "persist_rejects_same_id_different_hash_equivocation",
    ] {
        assert!(tests.contains(needle), "flow tests missing: {needle}");
    }
}

#[test]
fn phase_u_rfc_0203_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0203-failed-submit-durable.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0203 missing");
    for needle in [
        "#318",
        "persist_failed_submit",
        "failed",
        "FailureEvidence",
        "RFC-0198",
    ] {
        assert!(text.contains(needle), "RFC-0203 missing: {needle}");
    }
}

#[test]
fn phase_u_failed_submit_durable_present() {
    let src = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    for needle in [
        "#318",
        "RFC-0203",
        "persist_failed_submit",
        "persist_plane_events",
        "status: \"failed\"",
    ] {
        assert!(src.contains(needle), "local.rs missing: {needle}");
    }
    let tests = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    assert!(
        tests.contains("failed_submit_persists_problem_and_failure_after_reopen"),
        "flow tests missing failed_submit persist"
    );
}

#[test]
fn phase_u_rfc_0204_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0204-submit-executor-honesty.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0204 missing");
    for needle in [
        "#319",
        "with_backend_from_env",
        "staff_executor_kind",
        "reference",
        "RFC-0198",
    ] {
        assert!(text.contains(needle), "RFC-0204 missing: {needle}");
    }
}

#[test]
fn phase_u_executor_honesty_present() {
    let plane = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    for needle in [
        "#319",
        "RFC-0204",
        "with_backend_from_env",
        "enable_activated_mock_llm",
    ] {
        assert!(plane.contains(needle), "plane.rs missing: {needle}");
    }
    let flow = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    for needle in [
        "staff_executor_kind",
        "staff_bind_activate_gate_honors_process_env",
        "enable_activated_mock_llm_ignores_process_env",
    ] {
        assert!(flow.contains(needle), "flow lib missing: {needle}");
    }
    let cli = std::fs::read_to_string(repo_root().join("crates/aira-cli/src/commands/problem.rs"))
        .unwrap();
    assert!(cli.contains("executor"), "CLI must label executor");
    assert!(
        cli.contains("mode reference") || cli.contains("(reference)"),
        "CLI must label reference mock"
    );
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(
        i18n.contains("strip_model_ready_reference"),
        "Desktop strip must label reference mock"
    );
    let ms = std::fs::read_to_string(
        repo_root().join("crates/aira-desktop-runtime/src/model_status.rs"),
    )
    .unwrap();
    assert!(
        ms.contains("executor_kind"),
        "model_status must expose executor_kind"
    );
}

#[test]
fn phase_u_rfc_0205_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0205-addressbook-selective-rollback.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0205 missing");
    for needle in [
        "#320",
        "rollback_own_dial_candidate",
        "parallel",
        "RFC-0198",
    ] {
        assert!(text.contains(needle), "RFC-0205 missing: {needle}");
    }
}

#[test]
fn phase_u_addressbook_selective_rollback_present() {
    let src =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/peer_dial.rs"))
            .unwrap();
    for needle in [
        "#320",
        "RFC-0205",
        "rollback_own_dial_candidate",
        "selective_rollback_preserves_parallel_peer_upsert",
        "address book candidate rolled back",
    ] {
        assert!(src.contains(needle), "peer_dial missing: {needle}");
    }
    let book =
        std::fs::read_to_string(repo_root().join("crates/aira-peer/src/address_book.rs")).unwrap();
    assert!(book.contains("fn remove"), "AddressBook::remove required");
}

#[test]
fn phase_u_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-u-plan.md") || readme.contains("Phase U"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-u-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("first OPEN"));
    assert!(docs.contains("first OPEN `#321`") || docs.contains("#321"));
}

#[test]
fn phase_u_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-u-plan.md") || text.contains("Phase U"));
    assert!(text.contains("#321") || text.contains("перший OPEN"));
    assert!(
        !text.contains("перший OPEN `#320`") && !text.contains("first OPEN `#320`"),
        "NEXT_PROBLEM must not keep #320 as first-OPEN after close"
    );
}

#[test]
fn phase_u_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #320 |") || status.contains("#320"));
    assert!(status.contains("phase_u_doc.rs"));
    assert!(status.contains("phase-u-plan.md"));
    assert!(status.contains("RFC-0205") || status.contains("0205"));
}

#[test]
fn phase_t_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-t-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE T closed") || text.contains("RFC-0192"),
        "phase-t-plan should remain closed canon"
    );
}
