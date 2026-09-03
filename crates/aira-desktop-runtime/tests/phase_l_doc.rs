//! Phase L wiring contract (#217). Per-atom tests land with #218–#223.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0111_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0111") || n.contains("rfc-0111"))
        .collect()
}

#[test]
fn phase_l_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-l-plan.md")).unwrap();
    for needle in [
        "Phase L",
        "#217",
        "#218",
        "#223",
        "L0 govern",
        "Activate evidence",
        "env_clear",
        "execution_artifact_id",
        "AIRA-mediated",
        "AIRA-RFC-0111",
        "confirmed free",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
        "Landlock",
    ] {
        assert!(text.contains(needle), "phase-l-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-l-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_l_queue_wiring_217_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-l-plan.md"),
        "QUEUE missing phase-l-plan"
    );
    assert!(
        text.contains("| 217 | **DONE**"),
        "QUEUE #217 must be DONE after wiring"
    );
    assert!(
        !text.contains("| 217 | **OPEN**"),
        "QUEUE #217 must not stay OPEN after wiring"
    );
    for n in 222..=223 {
        let open = format!("| {n} | **OPEN**");
        let done = format!("| {n} | **DONE**");
        assert!(text.contains(&open), "QUEUE #{n} must be OPEN after wiring");
        assert!(
            !text.contains(&done),
            "QUEUE #{n} must not be DONE at wiring"
        );
    }
    assert!(
        text.contains("| 221 | **DONE**"),
        "QUEUE #221 must be DONE after ProblemRecord split"
    );
    assert!(
        !text.contains("| 221 | **OPEN**"),
        "QUEUE #221 must not stay OPEN after ProblemRecord split"
    );
    assert!(
        text.contains("| 218 | **DONE**"),
        "QUEUE #218 must be DONE after activate evidence"
    );
    assert!(
        !text.contains("| 218 | **OPEN**"),
        "QUEUE #218 must not stay OPEN after activate evidence"
    );
    assert!(
        text.contains("| 219 | **DONE**"),
        "QUEUE #219 must be DONE after child env whitelist"
    );
    assert!(
        !text.contains("| 219 | **OPEN**"),
        "QUEUE #219 must not stay OPEN after child env whitelist"
    );
    assert!(
        text.contains("| 220 | **DONE**"),
        "QUEUE #220 must be DONE after bounded pipes"
    );
    assert!(
        !text.contains("| 220 | **OPEN**"),
        "QUEUE #220 must not stay OPEN after bounded pipes"
    );
    for needle in ["L0 govern", "Analyze-252", "RFC-0111", "Activate evidence"] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_l_rfc_0111_id_free() {
    let hits = rfc_0111_hits();
    assert!(
        hits.is_empty(),
        "RFC-0111 must stay file-free until #223, found {hits:?}"
    );
}

#[test]
fn phase_l_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-l-plan.md"));
    assert!(readme.contains("#217"));
    assert!(readme.contains("#223"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-l-plan.md"));
    assert!(docs.contains("#217"));
    assert!(docs.contains("#223"));
}

#[test]
fn phase_k_points_to_phase_l() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-k-plan.md")).unwrap();
    assert!(text.contains("phase-l-plan.md"));
    assert!(text.contains("#217"));
}

#[test]
fn phase_l_status_row_217() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase L gates"));
    assert!(status.contains("phase_l_doc.rs"));
    assert!(status.contains("| #217 | Phase L wiring + contract"));
    assert!(status.contains("phase-l-plan.md"));
}

#[test]
fn phase_l_next_problem_not_phase_g_pointer() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("RESOLVED") || text.contains("provenance"),
        "NEXT_PROBLEM must stay provenance"
    );
    assert!(
        !text.contains("перший OPEN = `#134`"),
        "NEXT_PROBLEM must not keep stale Phase G OPEN pointer"
    );
    assert!(
        !text.contains("Active backlog: [`QUEUE.md`](QUEUE.md) Phase G"),
        "NEXT_PROBLEM must not claim Phase G is the active backlog"
    );
    assert!(
        text.contains("phase-l-plan.md") || text.contains("QUEUE.md"),
        "NEXT_PROBLEM must point at QUEUE / Phase L"
    );
}

#[test]
fn phase_l_activate_evidence_218() {
    let gate = repo_root().join("crates/aira-flow/src/activate_gate.rs");
    let text = std::fs::read_to_string(&gate).expect("activate_gate.rs");
    for needle in [
        "struct ActivatedPointerGate",
        "fn install_fixture",
        "content_hash mismatch",
        "evidence artifact missing",
        "fn forged_model_ref_only_pointer_is_denied",
        "fn cache_hash_mismatch_is_denied",
        "fn fixture_pointer_allows_generate",
        "activated!=true",
    ] {
        assert!(text.contains(needle), "activate_gate.rs missing: {needle}");
    }
    let flow = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    assert!(flow.contains("fn forged_model_ref_pointer_is_capsule_failed"));
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0112-activate-evidence-gate.md");
    assert!(rfc.is_file(), "RFC-0112 must exist for #218");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("AIRA-RFC-0112"));
    assert!(rfc_text.contains("content_hash"));
    assert!(rfc_text.contains("## 5. Non-Goals"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 218 | **DONE**"));
    assert!(!queue.contains("| 218 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #218 | Activate evidence"));
    assert!(status.contains("RFC-0112"));
    let hits = rfc_0111_hits();
    assert!(
        hits.is_empty(),
        "RFC-0111 must stay file-free, found {hits:?}"
    );
}

#[test]
fn phase_l_child_env_219() {
    let process = repo_root().join("csu/execution-llm/src/process.rs");
    let text = std::fs::read_to_string(&process).expect("process.rs");
    for needle in [
        "env_clear",
        "CHILD_ENV_ALLOWLIST",
        "AIRA_HTTP_TOKEN",
        "fn child_env_pairs_never_include_http_token",
        "fn process_child_does_not_inherit_http_token",
        "fn apply_child_env",
    ] {
        assert!(text.contains(needle), "process.rs missing: {needle}");
    }
    assert!(text.contains("\"PATH\"") && text.contains("\"HOME\"") && text.contains("\"LANG\""));
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0113-child-env-whitelist.md");
    assert!(rfc.is_file(), "RFC-0113 must exist for #219");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("AIRA-RFC-0113"));
    assert!(rfc_text.contains("AIRA_HTTP_TOKEN"));
    assert!(rfc_text.contains("## 5. Non-Goals"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 219 | **DONE**"));
    assert!(!queue.contains("| 219 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #219 | Child env whitelist"));
    assert!(status.contains("RFC-0113"));
    let hits = rfc_0111_hits();
    assert!(
        hits.is_empty(),
        "RFC-0111 must stay file-free, found {hits:?}"
    );
}

#[test]
fn phase_l_bounded_pipes_220() {
    let process = repo_root().join("csu/execution-llm/src/process.rs");
    let text = std::fs::read_to_string(&process).expect("process.rs");
    for needle in [
        "fn read_bounded",
        "PIPE_OVERFLOW",
        "PIPE_STDOUT_LIMIT",
        "PIPE_STDERR_LIMIT",
        "fn read_bounded_overflow_during_read",
        "fn stdout_overflow_during_read_is_fail_closed",
        "fn stderr_overflow_during_read_is_fail_closed",
    ] {
        assert!(text.contains(needle), "process.rs missing: {needle}");
    }
    let lib = std::fs::read_to_string(repo_root().join("csu/execution-llm/src/lib.rs")).unwrap();
    assert!(lib.contains("fn stdout_overflow_is_capsule_failed"));
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0114-bounded-pipes.md");
    assert!(rfc.is_file(), "RFC-0114 must exist for #220");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("AIRA-RFC-0114"));
    assert!(rfc_text.contains("during"));
    assert!(rfc_text.contains("## 5. Non-Goals"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 220 | **DONE**"));
    assert!(!queue.contains("| 220 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #220 | Bounded pipes"));
    assert!(status.contains("RFC-0114"));
    let hits = rfc_0111_hits();
    assert!(
        hits.is_empty(),
        "RFC-0111 must stay file-free, found {hits:?}"
    );
}

#[test]
fn phase_l_problem_record_221() {
    let local = repo_root().join("crates/aira-flow/src/local.rs");
    let text = std::fs::read_to_string(&local).expect("local.rs");
    for needle in [
        "pub execution_artifact_id: Option<String>",
        "fn split_executed_verified_lie",
        "verified_artifact_id: None,",
        "execution_artifact_id: Some(execution_artifact_id.as_str().to_string())",
    ] {
        assert!(text.contains(needle), "local.rs missing: {needle}");
    }
    let flow = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    assert!(flow.contains("fn local_session_generate_persists_execution_not_verified"));
    assert!(flow.contains("fn problem_status_remaps_legacy_executed_verified_lie"));
    let http =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/mod.rs")).unwrap();
    assert!(http.contains("fn http_get_problem_generate_parity_not_verified"));
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0115-problem-record-split.md");
    assert!(rfc.is_file(), "RFC-0115 must exist for #221");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("AIRA-RFC-0115"));
    assert!(rfc_text.contains("execution_artifact_id"));
    assert!(rfc_text.contains("## 5. Non-Goals"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 221 | **DONE**"));
    assert!(!queue.contains("| 221 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #221 | ProblemRecord split"));
    assert!(status.contains("RFC-0115"));
    let hits = rfc_0111_hits();
    assert!(
        hits.is_empty(),
        "RFC-0111 must stay file-free, found {hits:?}"
    );
}
