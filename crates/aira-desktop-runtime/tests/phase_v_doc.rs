//! Phase V contract smoke (#323 wiring … #330 RFC-0208 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_v_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-v-plan.md")).unwrap();
    for needle in [
        "Phase V",
        "Repair Pack 1",
        "#323",
        "#330",
        "Admission snapshot",
        "Reuse after constraints",
        "activate_verified",
        "Capsule↔Output↔Result",
        "E2E fail-closed",
        "AIRA-RFC-0208",
        "confirmed free",
        "QUEUE U closed",
        "QUEUE V closed",
        "no OPEN V atoms",
        "**DONE** @",
        "RFC-0207",
        "RFC-0209",
        "RFC-0210",
        "RFC-0211",
        "RFC-0212",
        "RFC-0213",
        "RFC-0214",
        "GPU marketplace",
        "Calculate 2 + 2",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-v-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-v-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        text.contains("**DONE** @ [AIRA-RFC-0208"),
        "phase-v-plan must claim RFC-0208 DONE after #330"
    );
    assert!(
        !text.contains("first OPEN `#324`")
            && !text.contains("first OPEN `#325`")
            && !text.contains("first OPEN `#326`")
            && !text.contains("first OPEN `#327`")
            && !text.contains("first OPEN `#328`")
            && !text.contains("first OPEN `#329`")
            && !text.contains("first OPEN `#330`"),
        "phase-v-plan must not keep a V first-OPEN after close"
    );
    assert!(
        !text.contains("**IN PROGRESS**"),
        "phase-v-plan must not stay IN PROGRESS after #330"
    );
}

#[test]
fn phase_v_queue_all_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-v-plan.md"));
    for n in 323..=330 {
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
        "Analyze-360",
        "Analyze-361",
        "Analyze-362",
        "Analyze-363",
        "Analyze-364",
        "Analyze-365",
        "Analyze-366",
        "Analyze-367",
        "RFC-0208",
        "RFC-0209",
        "RFC-0210",
        "RFC-0211",
        "RFC-0212",
        "RFC-0213",
        "RFC-0214",
        "Repair Pack 1",
        "admission integrity",
        "QUEUE V closed",
        "no OPEN V atoms",
        "QUEUE U closed",
        "desktop-ux.md",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#330`"),
        "QUEUE must not keep #330 as first-OPEN after close"
    );
}

#[test]
fn phase_v_rfc_0208_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0208-phase-v-admission-integrity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0208 missing");
    for needle in [
        "#330",
        "QUEUE V closed",
        "no OPEN V atoms",
        "RFC-0214",
        "RFC-0209",
        "admission integrity",
    ] {
        assert!(text.contains(needle), "RFC-0208 missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0209_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0209-admission-snapshot.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0209 missing");
    for needle in [
        "#324",
        "AdmissionSnapshot",
        "ProblemSubmitted",
        "RFC-0208",
        "default_for_text",
    ] {
        assert!(text.contains(needle), "RFC-0209 missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0210_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0210-submit-api-constraints.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0210 missing");
    for needle in [
        "#325",
        "AdmissionConstraints",
        "submit_problem_with_admission",
        "RFC-0208",
        "Settings",
    ] {
        assert!(text.contains(needle), "RFC-0210 missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0211_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0211-reuse-after-constraints.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0211 missing");
    for needle in [
        "#326",
        "reuse_catalog_key",
        "RequireNewExecution",
        "RFC-0208",
        "by_content_hash",
    ] {
        assert!(text.contains(needle), "RFC-0211 missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0212_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0212-activate-verified-hash.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0212 missing");
    for needle in [
        "#327",
        "ActivateHashMismatch",
        "VerifiedPointer",
        "activate_verified",
        "RFC-0208",
    ] {
        assert!(text.contains(needle), "RFC-0212 missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0213_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0213-capsule-output-result-binding.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0213 missing");
    for needle in [
        "#328",
        "ExecutorFacts",
        "model_content_hash",
        "CapsuleCompleted",
        "RFC-0208",
    ] {
        assert!(text.contains(needle), "RFC-0213 missing: {needle}");
    }
}

#[test]
fn phase_v_rfc_0214_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0214-e2e-fail-closed.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0214 missing");
    for needle in [
        "#329",
        "phase_v_pack1_e2e",
        "ActivateHashMismatch",
        "settings",
        "RFC-0208",
    ] {
        assert!(text.contains(needle), "RFC-0214 missing: {needle}");
    }
}

#[test]
fn phase_v_admission_runtime_present() {
    let admission =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/admission.rs")).unwrap();
    for needle in [
        "#324",
        "#325",
        "#326",
        "AdmissionSnapshot",
        "AdmissionConstraints",
        "from_text_and_constraints",
        "default_for_text",
        "reuse_catalog_key",
        "ADMISSION_SNAPSHOT_KIND",
        "RequireNewExecution",
    ] {
        assert!(admission.contains(needle), "admission.rs missing: {needle}");
    }
    let reuse = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/reuse.rs")).unwrap();
    for needle in [
        "reuse_catalog_key",
        "#326",
        "RequireNewExecution",
        "lookup_artifact_id",
    ] {
        assert!(reuse.contains(needle), "reuse.rs missing: {needle}");
    }
    let activate =
        std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/activate.rs")).unwrap();
    for needle in [
        "#327",
        "VerifiedPointer",
        "ActivateHashMismatch",
        "content_hash",
    ] {
        assert!(activate.contains(needle), "activate.rs missing: {needle}");
    }
    let err =
        std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/error.rs")).unwrap();
    assert!(err.contains("ActivateHashMismatch"));
    let execllm =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/src/lib.rs")).unwrap();
    for needle in [
        "#328",
        "ExecutorFacts",
        "stamp_executor_binding",
        "model_content_hash",
        "ALWAYS_ACTIVATED_MODEL_REF",
    ] {
        assert!(execllm.contains(needle), "execution-llm missing: {needle}");
    }
    let reduction =
        std::fs::read_to_string(repo_root().join("csu/reduction-basic/src/lib.rs")).unwrap();
    assert!(reduction.contains("admission_model_ref_from_context"));
    assert!(reduction.contains("model_artifact_ref"));
    let plane = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    for needle in [
        "publish_admission_snapshot",
        "last_admission",
        "submit_problem_with_admission",
        "bind_catalog_for_admission",
        "RFC-0210",
    ] {
        assert!(plane.contains(needle), "plane.rs missing: {needle}");
    }
    let local = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    assert!(local.contains("admission_snapshot"));
    assert!(local.contains("submit_problem_with_admission"));
    assert!(local.contains("record_reuse_index"));
    let handlers =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/handlers.rs")).unwrap();
    assert!(handlers.contains("AdmissionConstraints"));
    assert!(handlers.contains("submit_problem_with_admission"));
    let node_http =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/node_http.rs"))
            .unwrap();
    assert!(node_http.contains("AdmissionConstraints"));
    assert!(node_http.contains("\"admission\""));
    let e2e =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/tests/phase_v_pack1_e2e.rs"))
            .unwrap();
    for needle in [
        "#329",
        "pack1_e2e_same_text_different_model_no_reuse",
        "pack1_e2e_settings_mid_run_keeps_admission_binding",
        "pack1_e2e_verify_tamper_before_activate_rejected",
        "pack1_e2e_c1_and_generate_local_facts_hold",
        "ActivateHashMismatch",
    ] {
        assert!(e2e.contains(needle), "phase_v_pack1_e2e missing: {needle}");
    }
}

#[test]
fn phase_v_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-v-plan.md",
        "#330",
        "RFC-0208",
        "QUEUE V closed",
        "no OPEN V atoms",
        "QUEUE U closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#330`"),
        "desktop-ux must not keep #330 as first-OPEN after close"
    );
}

#[test]
fn phase_v_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-v-plan.md") || readme.contains("Phase V"));
    assert!(readme.contains("QUEUE V closed") || readme.contains("RFC-0208"));
    assert!(
        !readme.contains("first OPEN `#330`"),
        "README must not keep #330 as first-OPEN after close"
    );
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-v-plan.md"));
    assert!(docs.contains("QUEUE V closed") || docs.contains("**DONE** @ RFC-0208"));
    assert!(docs.contains("repair-package-1.md"));
    assert!(
        !docs.contains("first OPEN `#330`"),
        "docs/README must not keep #330 as first-OPEN after close"
    );
}

#[test]
fn phase_v_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-v-plan.md") || text.contains("Phase V"));
    assert!(text.contains("QUEUE V closed") || text.contains("RFC-0208"));
    assert!(
        !text.contains("first OPEN `#330`") && !text.contains("перший OPEN `#330`"),
        "NEXT_PROBLEM must not keep #330 as first-OPEN after close"
    );
    assert!(text.contains("phase-u-plan.md") || text.contains("Phase U"));
    assert!(text.contains("QUEUE U closed") || text.contains("RFC-0198"));
}

#[test]
fn phase_v_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #330 |") || status.contains("#330"));
    assert!(status.contains("phase_v_doc.rs"));
    assert!(status.contains("phase-v-plan.md"));
    assert!(status.contains("RFC-0208") || status.contains("0208"));
    assert!(status.contains("QUEUE V closed"));
}

#[test]
fn phase_v_repair_package_1_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-1.md")).unwrap();
    for needle in [
        "RFC-0208",
        "phase-v-plan.md",
        "#330",
        "QUEUE V closed",
        "Analyze-367",
    ] {
        assert!(text.contains(needle), "repair-package-1 missing: {needle}");
    }
    assert!(
        text.contains("Structural **DONE**") || text.contains("**DONE** @ [AIRA-RFC-0208"),
        "repair-package-1 must keep Phase V structural DONE"
    );
}

#[test]
fn phase_v_pack0_still_done() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-0.md")).unwrap();
    assert!(text.contains("**DONE**"));
    assert!(text.contains("RFC-0207") || text.contains("0207"));
}

#[test]
fn phase_u_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-u-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE U closed") || text.contains("RFC-0198"),
        "phase-u-plan should remain closed canon"
    );
}
