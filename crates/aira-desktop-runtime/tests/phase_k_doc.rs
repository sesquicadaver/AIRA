//! Phase K wiring contract (#209), generate-local schema (#210), execution-llm mock (#211),
//! Reduction generate-local bind (#212), plane register execution-llm (#213),
//! activate gate (#214).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfc_0104_hits() -> Vec<String> {
    let rfc_dir = repo_root().join("specs/rfc");
    std::fs::read_dir(&rfc_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("RFC-0104") || n.contains("rfc-0104"))
        .collect()
}

#[test]
fn phase_k_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-k-plan.md")).unwrap();
    for needle in [
        "Phase K",
        "#209",
        "#210",
        "#216",
        "K0 govern",
        "text.generate.local",
        "execution-llm",
        "AIRA-RFC-0104",
        "confirmed free",
        "IN PROGRESS",
        "GPU marketplace",
        "LLM runtime (Core як inference host)",
        "Calculate 2 + 2",
    ] {
        assert!(text.contains(needle), "phase-k-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-k-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_k_queue_wiring_209_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-k-plan.md"),
        "QUEUE missing phase-k-plan"
    );
    assert!(
        text.contains("| 209 | **DONE**"),
        "QUEUE #209 must be DONE after wiring"
    );
    assert!(
        text.contains("| 210 | **DONE**"),
        "QUEUE #210 must be DONE after generate-local schema"
    );
    assert!(
        text.contains("| 211 | **DONE**"),
        "QUEUE #211 must be DONE after execution-llm mock"
    );
    assert!(
        text.contains("| 212 | **DONE**"),
        "QUEUE #212 must be DONE after Reduction generate-local bind"
    );
    assert!(
        text.contains("| 213 | **DONE**"),
        "QUEUE #213 must be DONE after plane register"
    );
    assert!(
        text.contains("| 214 | **DONE**"),
        "QUEUE #214 must be DONE after activate gate"
    );
    assert!(
        !text.contains("| 210 | **OPEN**"),
        "QUEUE #210 must not stay OPEN after generate-local schema"
    );
    assert!(
        !text.contains("| 211 | **OPEN**"),
        "QUEUE #211 must not stay OPEN after execution-llm mock"
    );
    assert!(
        !text.contains("| 212 | **OPEN**"),
        "QUEUE #212 must not stay OPEN after Reduction generate-local bind"
    );
    assert!(
        !text.contains("| 213 | **OPEN**"),
        "QUEUE #213 must not stay OPEN after plane register"
    );
    assert!(
        !text.contains("| 214 | **OPEN**"),
        "QUEUE #214 must not stay OPEN after activate gate"
    );
    for n in 215..=216 {
        let open = format!("| {n} | **OPEN**");
        assert!(text.contains(&open), "QUEUE #{n} must be OPEN after #214");
        let done = format!("| {n} | **DONE**");
        assert!(!text.contains(&done), "QUEUE #{n} must not be DONE at #214");
    }
    assert!(
        !text.contains("| 209 | **OPEN**"),
        "QUEUE #209 must not stay OPEN after wiring"
    );
    for needle in [
        "K0 govern",
        "Analyze-244",
        "RFC-0104",
        "text.generate.local",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_k_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-k-plan.md"));
    assert!(readme.contains("#209"));
    assert!(readme.contains("#210"));
    assert!(readme.contains("#211"));
    assert!(readme.contains("#212"));
    assert!(readme.contains("#213"));
    assert!(readme.contains("#214"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-k-plan.md"));
    assert!(docs.contains("#209"));
    assert!(docs.contains("#216"));
}

#[test]
fn phase_j_points_to_active_phase_k() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-j-plan.md")).unwrap();
    assert!(text.contains("phase-k-plan.md"));
    assert!(text.contains("#209"));
}

#[test]
fn phase_k_rfc_0104_id_free() {
    let hits = rfc_0104_hits();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
}

#[test]
fn phase_k_status_row_209() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase K gates"));
    assert!(status.contains("phase_k_doc.rs"));
    assert!(status.contains("| #209 | Phase K wiring + contract"));
    assert!(status.contains("phase-k-plan.md"));
}

#[test]
fn phase_k_generate_local_210() {
    let schema = repo_root().join("schemas/execution/generate-local.schema.json");
    let text = std::fs::read_to_string(&schema).expect("generate-local schema");
    assert!(text.contains("\"$id\": \"aira:schema:execution:generate-local:0.1\""));
    assert!(text.contains("text.generate.local"));
    assert!(text.contains("\"additionalProperties\": false"));
    assert!(repo_root()
        .join("fixtures/valid/execution/generate-local.json")
        .is_file());
    assert!(repo_root()
        .join("fixtures/invalid/execution/generate-local-missing-prompt.json")
        .is_file());
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0105-generate-local-payload-schema.md");
    assert!(rfc.is_file(), "RFC-0105 must exist for #210");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("aira:schema:execution:generate-local:0.1"));
    assert!(rfc_text.contains("text.generate.local"));
    let hits = rfc_0104_hits();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 210 | **DONE**"));
    assert!(!queue.contains("| 210 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("aira:schema:execution:generate-local:0.1"));
    assert!(status.contains("| #210 | Capsule `text.generate.local`"));
}

#[test]
fn phase_k_execution_llm_211() {
    let cargo = repo_root().join("csu/execution-llm/Cargo.toml");
    let lib = repo_root().join("csu/execution-llm/src/lib.rs");
    assert!(cargo.is_file(), "csu/execution-llm Cargo.toml missing");
    assert!(lib.is_file(), "csu/execution-llm src/lib.rs missing");
    let cargo_text = std::fs::read_to_string(&cargo).unwrap();
    assert!(
        cargo_text.contains("name = \"aira-csu-execution-llm\""),
        "workspace crate name must be aira-csu-execution-llm"
    );
    assert!(
        !cargo_text.contains("model-inventory"),
        "execution-llm must not depend on inventory CSU"
    );
    assert!(
        !cargo_text.contains("model-acquisition"),
        "execution-llm must not depend on acquisition CSU"
    );
    let lib_text = std::fs::read_to_string(&lib).unwrap();
    for needle in [
        "struct MockBackend",
        "trait GenerateBackend",
        "struct ExecutionLlmCsu",
        "text.generate.local",
        "aira:schema:execution:generate-local:0.1",
        "fn mock_backend_completes_valid_generate_local",
        "fn missing_backend_is_capsule_failed",
        "trait ModelActivateGate",
        "deny_unknown_fields",
    ] {
        assert!(
            lib_text.contains(needle),
            "execution-llm lib missing: {needle}"
        );
    }
    assert!(
        !lib_text.contains("std::process"),
        "MockBackend path must not shell out"
    );
    assert!(
        !lib_text.contains("Command::"),
        "MockBackend path must not spawn Command"
    );
    let ws = std::fs::read_to_string(repo_root().join("Cargo.toml")).unwrap();
    assert!(
        ws.contains("csu/execution-llm"),
        "workspace must list csu/execution-llm"
    );
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0106-execution-llm-mock.md");
    assert!(rfc.is_file(), "RFC-0106 must exist for #211");
    let hits = rfc_0104_hits();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 211 | **DONE**"));
    assert!(!queue.contains("| 211 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #211 | `execution-llm` CSU + mock"));
    assert!(status.contains("**DONE** @ this PR"));
    let flow = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    assert!(
        !flow.contains("execution-llm") && !flow.contains("ExecutionLlmCsu"),
        "LocalSession must not independently construct execution-llm (plane.rs is the register site)"
    );
}

#[test]
fn phase_k_reduction_bind_212() {
    let cargo = repo_root().join("csu/reduction-basic/Cargo.toml");
    let lib = repo_root().join("csu/reduction-basic/src/lib.rs");
    assert!(cargo.is_file(), "reduction-basic Cargo.toml missing");
    assert!(lib.is_file(), "reduction-basic src/lib.rs missing");
    let cargo_text = std::fs::read_to_string(&cargo).unwrap();
    assert!(
        cargo_text.contains("name = \"aira-csu-reduction-basic\""),
        "workspace crate name must be aira-csu-reduction-basic"
    );
    assert!(
        !cargo_text.contains("execution-llm"),
        "reduction-basic must not Cargo-dep execution-llm (CSU ↛ CSU)"
    );
    let lib_text = std::fs::read_to_string(&lib).unwrap();
    for needle in [
        "fn catalog_action",
        "text.generate.local",
        "aira:schema:execution:generate-local:0.1",
        "fn calculate_2_plus_2_binds_math_eval_safe",
        "fn non_math_prompt_binds_generate_local",
        "math.eval.safe",
        "text.echo",
        "text.uppercase",
    ] {
        assert!(
            lib_text.contains(needle),
            "reduction-basic lib missing: {needle}"
        );
    }
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0107-reduction-generate-local.md");
    assert!(rfc.is_file(), "RFC-0107 must exist for #212");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("text.generate.local"));
    assert!(rfc_text.contains("math.eval.safe"));
    let hits = rfc_0104_hits();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 212 | **DONE**"));
    assert!(queue.contains("| 213 | **DONE**"));
    assert!(!queue.contains("| 212 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #212 | Reduction bind"));
    assert!(status.contains("RFC-0107"));
    let flow = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    assert!(
        !flow.contains("execution-llm") && !flow.contains("ExecutionLlmCsu"),
        "LocalSession must not independently construct execution-llm (plane.rs is the register site)"
    );
}

#[test]
fn phase_k_plane_register_213() {
    let plane = repo_root().join("crates/aira-flow/src/plane.rs");
    let cargo = repo_root().join("crates/aira-flow/Cargo.toml");
    let plane_text = std::fs::read_to_string(&plane).unwrap();
    let cargo_text = std::fs::read_to_string(&cargo).unwrap();
    assert!(
        cargo_text.contains("aira-csu-execution-llm"),
        "aira-flow must depend on aira-csu-execution-llm"
    );
    for needle in [
        "ExecutionLlmCsu",
        "with_mock_backend",
        "SubmitOutcome::Executed",
        "fn latest_generate_local_output",
    ] {
        assert!(plane_text.contains(needle), "plane.rs missing: {needle}");
    }
    let flow_tests =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    for needle in [
        "fn non_math_prompt_completes_via_execution_llm_mock",
        "fn calculate_two_plus_two_stays_execution_basic",
    ] {
        assert!(
            flow_tests.contains(needle),
            "aira-flow tests missing: {needle}"
        );
    }
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0108-plane-register-execution-llm.md");
    assert!(rfc.is_file(), "RFC-0108 must exist for #213");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("execution-llm"));
    assert!(rfc_text.contains("MockBackend"));
    assert!(rfc_text.contains("math.eval.safe"));
    let hits = rfc_0104_hits();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 213 | **DONE**"));
    assert!(queue.contains("| 214 | **DONE**"));
    assert!(!queue.contains("| 213 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #213 | Plane register"));
    assert!(status.contains("RFC-0108"));
    let llm_cargo =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/Cargo.toml")).unwrap();
    assert!(
        !llm_cargo.contains("model-inventory"),
        "execution-llm must not depend on inventory CSU"
    );
    assert!(
        !llm_cargo.contains("model-acquisition"),
        "execution-llm must not depend on acquisition CSU"
    );
}

#[test]
fn phase_k_activate_gate_214() {
    let lib = repo_root().join("csu/execution-llm/src/lib.rs");
    let lib_text = std::fs::read_to_string(&lib).unwrap();
    for needle in [
        "trait ModelActivateGate",
        "struct AlwaysActivated",
        "struct NeverActivated",
        "fn with_activate_gate",
        "fn check_activate",
        "fn inactive_model_is_capsule_failed",
        "fn never_activated_gate_is_capsule_failed",
        "fn mock_backend_completes_valid_generate_local",
        "ACTIVATE_DENIED",
    ] {
        assert!(
            lib_text.contains(needle),
            "execution-llm lib missing: {needle}"
        );
    }
    assert!(
        !lib_text.contains("TODO(#214)"),
        "activate-gate placeholder TODO must be gone after #214"
    );
    let llm_cargo =
        std::fs::read_to_string(repo_root().join("csu/execution-llm/Cargo.toml")).unwrap();
    assert!(
        !llm_cargo.contains("model-inventory"),
        "execution-llm must not depend on inventory CSU"
    );
    assert!(
        !llm_cargo.contains("model-acquisition"),
        "execution-llm must not depend on acquisition CSU"
    );
    let plane_text =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    for needle in [
        "struct ActivatedPointerGate",
        "fn bind_activate_gate",
        "fn enable_activated_mock_llm",
        "fn bind_phase_d_activate_from_root",
    ] {
        assert!(plane_text.contains(needle), "plane.rs missing: {needle}");
    }
    let flow_tests =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    for needle in [
        "fn generate_without_activate_is_capsule_failed",
        "fn non_math_prompt_completes_via_execution_llm_mock",
        "fn phase_d_activated_pointer_allows_mock_generate",
        "fn calculate_two_plus_two_stays_execution_basic",
    ] {
        assert!(
            flow_tests.contains(needle),
            "aira-flow tests missing: {needle}"
        );
    }
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0109-activate-gate.md");
    assert!(rfc.is_file(), "RFC-0109 must exist for #214");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("ModelActivateGate"));
    assert!(rfc_text.contains("CapsuleFailed"));
    assert!(rfc_text.contains("Phase D"));
    let hits = rfc_0104_hits();
    assert!(
        hits.is_empty(),
        "RFC-0104 must stay free until #216, found {hits:?}"
    );
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 214 | **DONE**"));
    assert!(queue.contains("| 215 | **OPEN**"));
    assert!(!queue.contains("| 214 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #214 | Activate gate"));
    assert!(status.contains("RFC-0109"));
    let local = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    assert!(
        !local.contains("execution-llm") && !local.contains("ExecutionLlmCsu"),
        "LocalSession must not independently construct execution-llm (plane.rs is the register site)"
    );
}
