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
        "first OPEN `#334`",
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
        !text.contains("first OPEN `#333`"),
        "phase-w-plan must advance tip past #333"
    );
}

#[test]
fn phase_w_queue_333_done_334_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-w-plan.md"));
    for n in 331..=333 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(text.contains("| 334 | **OPEN**"), "QUEUE #334 must be OPEN");
    for n in 335..=342 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 334 | **DONE**") && !text.contains("| 342 | **DONE**"),
        "QUEUE #334/#342 must not be DONE yet"
    );
    for needle in [
        "Analyze-368",
        "Analyze-369",
        "Analyze-370",
        "RFC-0215",
        "RFC-0216",
        "RFC-0217",
        "Pack 1 residual",
        "first OPEN `#334`",
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
        "#333",
        "#334",
        "RFC-0215",
        "RFC-0217",
        "QUEUE V closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_w_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-w-plan.md") || readme.contains("Phase W"));
    assert!(readme.contains("#334") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-w-plan.md"));
    assert!(docs.contains("#333") || docs.contains("IN PROGRESS"));
}

#[test]
fn phase_w_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-w-plan.md") || text.contains("Phase W"));
    assert!(text.contains("#334") || text.contains("перший OPEN"));
    assert!(text.contains("QUEUE V closed") || text.contains("RFC-0208"));
}

#[test]
fn phase_w_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #333 |") || status.contains("#333"));
    assert!(status.contains("phase_w_doc.rs"));
    assert!(status.contains("phase-w-plan.md"));
    assert!(status.contains("RFC-0217") || status.contains("0217"));
    assert!(status.contains("#334"));
}

#[test]
fn phase_w_repair_package_1_honesty() {
    let text = std::fs::read_to_string(repo_root().join("docs/repair-package-1.md")).unwrap();
    for needle in [
        "RFC-0208",
        "PARTIAL",
        "phase-w-plan.md",
        "#333",
        "#334",
        "RFC-0215",
        "RFC-0217",
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
