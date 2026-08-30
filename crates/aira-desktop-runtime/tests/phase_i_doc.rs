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
        "first OPEN `#189`",
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
        text.contains("| 189 | **OPEN**"),
        "QUEUE #189 must be first remaining OPEN"
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
    for n in 189..=198 {
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
        "Analyze-233",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_i_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-i-plan.md"));
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
    assert!(docs.contains("#189"));
    assert!(docs.contains("#188"));
}

#[test]
fn phase_h_points_to_active_phase_i() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-h-plan.md")).unwrap();
    assert!(text.contains("phase-i-plan.md"));
    assert!(text.contains("#184"));
    assert!(text.contains("first OPEN `#189`"));
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
