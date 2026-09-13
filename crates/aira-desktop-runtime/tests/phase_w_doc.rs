//! Phase W contract smoke (#331 wiring … #342 RFC-0215 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_w_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-w-plan.md")).unwrap();
    for needle in [
        "Phase W",
        "Pack 1 residual honesty",
        "#331",
        "#342",
        "IN PROGRESS",
        "Math capsule fidelity",
        "Strict submit decoding",
        "Constraints enforce-or-reject",
        "Activate evidence authority",
        "Backend verified binding",
        "AIRA-RFC-0215",
        "confirmed free",
        "QUEUE V closed",
        "RFC-0208",
        "RFC-0216",
        "RFC-0217",
        "RFC-0218",
        "RFC-0219",
        "first OPEN `#336`",
        "D1",
        "D6",
        "GPU marketplace",
        "Calculate 2 + 2",
        "M1–M6",
        "phase-x-plan.md",
        "Handoff",
        "public bind",
    ] {
        assert!(text.contains(needle), "phase-w-plan missing: {needle}");
    }
    assert!(
        !text.contains("НЕ АКТИВОВАНО"),
        "phase-w-plan must be activated (not НЕ АКТИВОВАНО)"
    );
    assert!(
        !text.contains("**DONE** @ [AIRA-RFC-0215"),
        "phase-w-plan must not claim RFC-0215 DONE before #342"
    );
    assert!(
        !text.contains("first OPEN `#335`"),
        "phase-w-plan must advance tip past #335"
    );
}

#[test]
fn phase_w_queue_335_done_336_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-w-plan.md"));
    for n in 331..=335 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(text.contains("| 336 | **OPEN**"), "QUEUE #336 must be OPEN");
    for n in 337..=342 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 336 | **DONE**") && !text.contains("| 342 | **DONE**"),
        "QUEUE #336/#342 must not be DONE yet"
    );
    for needle in [
        "Analyze-368",
        "Analyze-369",
        "Analyze-370",
        "Analyze-371",
        "Analyze-372",
        "RFC-0215",
        "RFC-0216",
        "RFC-0217",
        "RFC-0218",
        "RFC-0219",
        "Pack 1 residual",
        "first OPEN `#336`",
        "QUEUE V closed",
        "phase-x-plan.md",
        "Pack 2",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    for n in 343..=358 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN (Phase X QUEUED)"
        );
    }
}

#[test]
fn phase_w_rfc_0215_file_free() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0215-phase-w-pack1-residual-honesty.md");
    assert!(
        !path.exists(),
        "RFC-0215 must stay file-free until #342 close"
    );
}

#[test]
fn phase_w_rfc_0216_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0216-math-capsule-fidelity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0216 missing");
    for needle in ["#332", "extract_math_expression", "9-3", "2+2", "RFC-0215"] {
        assert!(text.contains(needle), "RFC-0216 missing: {needle}");
    }
}

#[test]
fn phase_w_rfc_0219_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0219-admission-boundary-verify.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0219 missing");
    for needle in [
        "#335",
        "verify_input_boundary",
        "statement_content_hash",
        "AdmissionBoundary",
        "RFC-0215",
    ] {
        assert!(text.contains(needle), "RFC-0219 missing: {needle}");
    }
}

#[test]
fn phase_w_admission_boundary_runtime_present() {
    let admission =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/admission.rs")).unwrap();
    assert!(admission.contains("verify_input_boundary"));
    assert!(admission.contains("#335") || admission.contains("RFC-0219"));
    let plane = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    assert!(plane.contains("AdmissionBoundary"));
    let handlers =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/handlers.rs")).unwrap();
    assert!(handlers.contains("AdmissionBoundary"));
    let context =
        std::fs::read_to_string(repo_root().join("csu/context-basic/src/lib.rs")).unwrap();
    assert!(context.contains("admission boundary"));
}

#[test]
fn phase_w_rfc_0218_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0218-constraints-enforce-or-reject.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0218 missing");
    for needle in [
        "#334",
        "enforce_or_reject",
        "remote_required",
        "UnsupportedConstraint",
        "RFC-0215",
    ] {
        assert!(text.contains(needle), "RFC-0218 missing: {needle}");
    }
}

#[test]
fn phase_w_constraints_runtime_present() {
    let admission =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/admission.rs")).unwrap();
    assert!(admission.contains("enforce_or_reject"));
    assert!(admission.contains("#334") || admission.contains("RFC-0218"));
    let plane = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    assert!(plane.contains("UnsupportedConstraint"));
    let handlers =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/handlers.rs")).unwrap();
    assert!(handlers.contains("UnsupportedConstraint"));
}

#[test]
fn phase_w_rfc_0217_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0217-strict-submit-decoding.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0217 missing");
    for needle in [
        "#333",
        "deny_unknown_fields",
        "admisson",
        "model_reff",
        "RFC-0215",
        "4xx",
    ] {
        assert!(text.contains(needle), "RFC-0217 missing: {needle}");
    }
}

#[test]
fn phase_w_math_runtime_present() {
    let reduction =
        std::fs::read_to_string(repo_root().join("csu/reduction-basic/src/lib.rs")).unwrap();
    for needle in [
        "#332",
        "extract_math_expression",
        "refusing default 2+2",
        "CapsuleFailed",
    ] {
        assert!(
            reduction.contains(needle),
            "reduction-basic missing: {needle}"
        );
    }
    assert!(
        !reduction.contains("\"2+2\".into()"),
        "reduction must not invent default 2+2"
    );
    let execution =
        std::fs::read_to_string(repo_root().join("csu/execution-basic/src/lib.rs")).unwrap();
    assert!(execution.contains("no default 2+2"));
    assert!(!execution.contains("unwrap_or(\"2+2\")"));
}

#[test]
fn phase_w_strict_submit_runtime_present() {
    let admission =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/admission.rs")).unwrap();
    assert!(admission.contains("deny_unknown_fields"));
    assert!(admission.contains("#333") || admission.contains("RFC-0217"));
    let handlers =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/handlers.rs")).unwrap();
    assert!(handlers.contains("deny_unknown_fields"));
    assert!(handlers.contains("ProblemSubmitBody"));
    let http =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/mod.rs")).unwrap();
    assert!(http.contains("http_post_problem_unknown_top_level_field_is_4xx"));
    assert!(http.contains("http_post_problem_unknown_admission_field_is_4xx"));
}

#[test]
fn phase_w_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-w-plan.md",
        "#335",
        "#336",
        "RFC-0215",
        "RFC-0219",
        "QUEUE V closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_w_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-w-plan.md") || readme.contains("Phase W"));
    assert!(readme.contains("#336") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-w-plan.md"));
    assert!(docs.contains("#333") || docs.contains("IN PROGRESS"));
}

#[test]
fn phase_w_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-w-plan.md") || text.contains("Phase W"));
    assert!(text.contains("#336") || text.contains("перший OPEN"));
    assert!(text.contains("QUEUE V closed") || text.contains("RFC-0208"));
}

#[test]
fn phase_w_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #335 |") || status.contains("#335"));
    assert!(status.contains("phase_w_doc.rs"));
    assert!(status.contains("phase-w-plan.md"));
    assert!(status.contains("RFC-0219") || status.contains("0219"));
    assert!(status.contains("#336"));
}

#[test]
fn phase_w_repair_package_1_honesty() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-1.md")).unwrap();
    for needle in [
        "RFC-0208",
        "PARTIAL",
        "phase-w-plan.md",
        "#335",
        "#336",
        "RFC-0215",
        "RFC-0219",
    ] {
        assert!(text.contains(needle), "repair-package-1 missing: {needle}");
    }
}

#[test]
fn phase_v_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-v-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE V closed") || text.contains("RFC-0208"),
        "phase-v-plan should remain closed canon"
    );
    assert!(
        text.contains("**DONE** @ [AIRA-RFC-0208"),
        "Phase V must stay DONE @ RFC-0208"
    );
}
