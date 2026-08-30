//! Phase I wiring contract (#184): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_i_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-i-plan.md")).unwrap();
    for needle in [
        "Phase I",
        "#184",
        "#185",
        "#198",
        "I0 govern + status honesty",
        "Handle integrity",
        "Semantic verify",
        "PolicyGate",
        "Durable reuse",
        "AIRA-RFC-0078",
        "confirmed free",
        "IN PROGRESS",
        "first OPEN `#197`",
        "GPU marketplace",
    ] {
        assert!(text.contains(needle), "phase-i-plan missing: {needle}");
    }
}

#[test]
fn phase_i_queue_wiring_184_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-i-plan.md"),
        "QUEUE missing phase-i-plan"
    );
    assert!(
        text.contains("| 184 | **DONE**"),
        "QUEUE #184 must be DONE after wiring"
    );
    assert!(
        text.contains("| 185 | **DONE**"),
        "QUEUE #185 must be DONE after status honesty"
    );
    assert!(
        text.contains("| 186 | **DONE**"),
        "QUEUE #186 must be DONE after Handle integrity"
    );
    assert!(
        text.contains("| 187 | **DONE**"),
        "QUEUE #187 must be DONE after semantic verify"
    );
    assert!(
        text.contains("| 188 | **DONE**"),
        "QUEUE #188 must be DONE after PolicyGate invoke"
    );
    assert!(
        text.contains("| 189 | **DONE**"),
        "QUEUE #189 must be DONE after durable reuse"
    );
    assert!(
        text.contains("| 190 | **DONE**"),
        "QUEUE #190 must be DONE after fail-closed signing"
    );
    assert!(
        text.contains("| 191 | **DONE**"),
        "QUEUE #191 must be DONE after atomic persist"
    );
    assert!(
        text.contains("| 192 | **DONE**"),
        "QUEUE #192 must be DONE after artifact recovery"
    );
    assert!(
        text.contains("| 193 | **DONE**"),
        "QUEUE #193 must be DONE after runtime Clock"
    );
    assert!(
        text.contains("| 194 | **DONE**"),
        "QUEUE #194 must be DONE after envelope freshness"
    );
    assert!(
        text.contains("| 195 | **DONE**"),
        "QUEUE #195 must be DONE after run nonce"
    );
    assert!(
        text.contains("| 196 | **DONE**"),
        "QUEUE #196 must be DONE after instance-scoped crypto"
    );
    assert!(
        text.contains("| 197 | **OPEN**"),
        "QUEUE #197 must be first remaining OPEN"
    );
    assert!(
        !text.contains("| 184 | **OPEN**"),
        "QUEUE #184 must not stay OPEN after wiring"
    );
    assert!(
        !text.contains("| 185 | **OPEN**"),
        "QUEUE #185 must not stay OPEN after honesty rollup"
    );
    assert!(
        !text.contains("| 186 | **OPEN**"),
        "QUEUE #186 must not stay OPEN after Handle integrity"
    );
    assert!(
        !text.contains("| 187 | **OPEN**"),
        "QUEUE #187 must not stay OPEN after semantic verify"
    );
    assert!(
        !text.contains("| 188 | **OPEN**"),
        "QUEUE #188 must not stay OPEN after PolicyGate invoke"
    );
    assert!(
        !text.contains("| 189 | **OPEN**"),
        "QUEUE #189 must not stay OPEN after durable reuse"
    );
    assert!(
        !text.contains("| 190 | **OPEN**"),
        "QUEUE #190 must not stay OPEN after fail-closed signing"
    );
    assert!(
        !text.contains("| 191 | **OPEN**"),
        "QUEUE #191 must not stay OPEN after atomic persist"
    );
    assert!(
        !text.contains("| 192 | **OPEN**"),
        "QUEUE #192 must not stay OPEN after artifact recovery"
    );
    assert!(
        !text.contains("| 193 | **OPEN**"),
        "QUEUE #193 must not stay OPEN after runtime Clock"
    );
    assert!(
        !text.contains("| 194 | **OPEN**"),
        "QUEUE #194 must not stay OPEN after envelope freshness"
    );
    assert!(
        !text.contains("| 195 | **OPEN**"),
        "QUEUE #195 must not stay OPEN after run nonce"
    );
    assert!(
        !text.contains("| 196 | **OPEN**"),
        "QUEUE #196 must not stay OPEN after instance-scoped crypto"
    );
    for n in 197..=198 {
        let needle = format!("| {n} | **OPEN**");
        assert!(text.contains(&needle), "QUEUE missing OPEN row: {needle}");
    }
    for needle in [
        "I0 govern + status honesty",
        "I1 P0 Core/CSU semantics",
        "RFC-0078",
        "Analyze-219",
        "Analyze-222",
        "Analyze-223",
        "Analyze-224",
        "Analyze-225",
        "Analyze-226",
        "Analyze-227",
        "Analyze-228",
        "Analyze-229",
        "Analyze-230",
        "Analyze-231",
        "Analyze-233",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_i_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-i-plan.md"));
    assert!(readme.contains("#197"));
    assert!(readme.contains("#196"));
    assert!(readme.contains("#195"));
    assert!(readme.contains("#194"));
    assert!(readme.contains("#193"));
    assert!(readme.contains("#192"));
    assert!(readme.contains("#191"));
    assert!(readme.contains("#190"));
    assert!(readme.contains("#189"));
    assert!(readme.contains("#188"));
    assert!(readme.contains("#187"));
    assert!(readme.contains("#186"));
    assert!(readme.contains("#185"));
    assert!(readme.contains("#184"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-i-plan.md"));
    assert!(docs.contains("#184"));
    assert!(docs.contains("IN PROGRESS"));
    assert!(docs.contains("#197"));
    assert!(docs.contains("#196"));
    assert!(docs.contains("#195"));
    assert!(docs.contains("#194"));
    assert!(docs.contains("#193"));
    assert!(docs.contains("#192"));
    assert!(docs.contains("#191"));
    assert!(docs.contains("#190"));
    assert!(docs.contains("#189"));
    assert!(docs.contains("#188"));
}

#[test]
fn phase_h_points_to_active_phase_i() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-h-plan.md")).unwrap();
    assert!(text.contains("phase-i-plan.md"));
    assert!(text.contains("#184"));
    assert!(text.contains("first OPEN `#197`"));
}

#[test]
fn phase_i_rfc_0078_id_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    let entries = std::fs::read_dir(&rfc_dir).expect("specs/rfc");
    for entry in entries {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            !name.starts_with("AIRA-RFC-0078"),
            "RFC-0078 reserved for #198; unexpected file: {name}"
        );
    }
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase I gates"));
    assert!(status.contains("RFC-0078"));
    assert!(status.contains("phase_i_doc.rs"));
    assert!(status.contains("#185"));
}

#[test]
fn phase_i_status_honesty_185() {
    let text = std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "Opaque Handle",
        "**PARTIAL**",
        "Handle::new",
        "storage_token()",
        "object_id == handle.object_ref",
        "b66bcf1",
        "#185",
        "#186",
        "Reduction / reuse before execute",
        "LocalSession::submit_problem",
        "vec![]",
        "#189",
        "Verification CSU",
        "is_finite()",
        "VERIFIED",
        "#187",
        "| #185 | Status honesty rollup",
        "**DONE** @ this PR",
    ] {
        assert!(
            text.contains(needle),
            "implementation-status honesty missing: {needle}"
        );
    }
}

#[test]
fn phase_i_handle_integrity_186() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0084",
        "object_store_access",
        "HandleBindMismatch",
        "handle_cross_object_token_bind_rejects",
        "pub(crate)",
        "| #186 | Handle integrity",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #186 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0084-handle-integrity.md"))
            .expect("RFC-0084");
    for needle in ["#186", "HandleBindMismatch", "object_store_access"] {
        assert!(rfc.contains(needle), "RFC-0084 missing: {needle}");
    }
}

#[test]
fn phase_i_semantic_verify_187() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0085",
        "wrong_finite_math_result_is_not_verified",
        "math_expression_from_capsule_artifact",
        "| #187 | Semantic verify math.eval.safe",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #187 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0085-semantic-verify-math.md"),
    )
    .expect("RFC-0085");
    for needle in ["#187", "math.eval.safe", "VerificationFailed"] {
        assert!(rfc.contains(needle), "RFC-0085 missing: {needle}");
    }
}

#[test]
fn phase_i_policy_gate_invoke_188() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0086",
        "invoke_binds_policy_gate_check_policy_allows",
        "check_policy_fail_closed_without_bound_gate",
        "| #188 | CSU PolicyGate in invoke",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #188 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0086-csu-policy-gate-invoke.md"),
    )
    .expect("RFC-0086");
    for needle in ["#188", "check_policy", "policy gate not bound"] {
        assert!(rfc.contains(needle), "RFC-0086 missing: {needle}");
    }
}

#[test]
fn phase_i_durable_reuse_189() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0087",
        "local_session_repeat_problem_reuses_without_execution",
        "reuse-index.json",
        "| #189 | Durable reuse index",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #189 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0087-durable-reuse-index.md"))
            .expect("RFC-0087");
    for needle in ["#189", "reuse-index.json", "reuse:ready_solution"] {
        assert!(rfc.contains(needle), "RFC-0087 missing: {needle}");
    }
}

#[test]
fn phase_i_fail_closed_signing_190() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0088",
        "active_signature_does_not_fallback_to_local_test",
        "local_session_rejects_corrupt_identity",
        "| #190 | Fail-closed signing",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #190 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0088-fail-closed-signing.md"))
            .expect("RFC-0088");
    for needle in ["#190", "active_signature", "local-test"] {
        assert!(rfc.contains(needle), "RFC-0088 missing: {needle}");
    }
}

#[test]
fn phase_i_atomic_persist_191() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0089",
        "local_session_corrupt_problems_index_is_not_silent_wipe",
        "| #191 | Atomic session persist",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #191 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0089-atomic-session-persist.md"),
    )
    .expect("RFC-0089");
    for needle in ["#191", "write_json", "problems/index.json"] {
        assert!(rfc.contains(needle), "RFC-0089 missing: {needle}");
    }
}

#[test]
fn phase_i_artifact_recovery_192() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0090",
        "second_descriptor_same_content_hash_recoverable",
        "| #192 | Artifact metadata recovery",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #192 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0090-artifact-descriptor-recovery.md"),
    )
    .expect("RFC-0090");
    for needle in ["#192", "artifact_id", "content hash"] {
        assert!(rfc.contains(needle), "RFC-0090 missing: {needle}");
    }
}

#[test]
fn phase_i_runtime_clock_193() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0091",
        "system_clock_is_not_the_mvp_fixed_timestamp",
        "local_session_artifacts_are_not_all_mvp_fixed_timestamp",
        "| #193 | Runtime Clock",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #193 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0091-runtime-clock.md"))
        .expect("RFC-0091");
    for needle in ["#193", "Clock", "SystemClock"] {
        assert!(rfc.contains(needle), "RFC-0091 missing: {needle}");
    }
}

#[test]
fn phase_i_envelope_freshness_194() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0092",
        "recv_envelope_rejects_expired",
        "recv_envelope_rejects_replayed_message_id",
        "| #194 | Envelope freshness/replay",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #194 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0092-envelope-freshness-replay.md"),
    )
    .expect("RFC-0092");
    for needle in ["#194", "expires_at", "message_id"] {
        assert!(rfc.contains(needle), "RFC-0092 missing: {needle}");
    }
}

#[test]
fn phase_i_run_nonce_195() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0093",
        "alloc_run_nonce_concurrent_is_unique",
        "| #195 | Run nonce concurrency",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #195 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0093-run-nonce-uuidv7.md"))
            .expect("RFC-0093");
    for needle in ["#195", "UUIDv7", "run-counter"] {
        assert!(rfc.contains(needle), "RFC-0093 missing: {needle}");
    }
}

#[test]
fn phase_i_instance_crypto_196() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0094",
        "thread_crypto_scopes_do_not_leak",
        "| #196 | Instance-scoped crypto",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #196 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0094-instance-scoped-crypto.md"),
    )
    .expect("RFC-0094");
    for needle in ["#196", "bind_thread_crypto", "OnceLock"] {
        assert!(rfc.contains(needle), "RFC-0094 missing: {needle}");
    }
}
