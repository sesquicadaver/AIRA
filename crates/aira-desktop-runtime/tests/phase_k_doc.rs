//! Phase K wiring contract (#209), generate-local schema (#210), execution-llm mock (#211),
//! Reduction generate-local bind (#212), plane register execution-llm (#213),
//! activate gate (#214), process backend (#215), Desktop generate + RFC-0104 (#216).

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
        "**DONE**",
        "QUEUE K closed",
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
        text.contains("| 215 | **DONE**"),
        "QUEUE #215 must be DONE after process backend"
    );
    assert!(
        text.contains("| 216 | **DONE**"),
        "QUEUE #216 must be DONE after Desktop RFC-0104"
    );
    assert!(
        text.contains("QUEUE K closed"),
        "QUEUE K must be closed after #216"
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
    assert!(
        !text.contains("| 215 | **OPEN**"),
        "QUEUE #215 must not stay OPEN after process backend"
    );
    assert!(
        !text.contains("| 216 | **OPEN**"),
        "QUEUE #216 must not stay OPEN after RFC-0104"
    );
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
    assert!(readme.contains("#215"));
    assert!(readme.contains("#216"));
    assert!(readme.contains("RFC-0104") || readme.contains("AIRA-RFC-0104"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-k-plan.md"));
    assert!(docs.contains("#209"));
    assert!(docs.contains("#216"));
    assert!(docs.contains("QUEUE K closed"));
    assert!(docs.contains("RFC-0104") || docs.contains("AIRA-RFC-0104"));
}

#[test]
fn phase_j_points_to_active_phase_k() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-j-plan.md")).unwrap();
    assert!(text.contains("phase-k-plan.md"));
    assert!(text.contains("#209"));
}

#[test]
fn phase_k_rfc_0104_present() {
    let hits = rfc_0104_hits();
    assert_eq!(hits.len(), 1, "exactly one RFC-0104 file, found {hits:?}");
    let path = repo_root().join("specs/rfc").join(&hits[0]);
    let text = std::fs::read_to_string(&path).expect("RFC-0104 file");
    for needle in [
        "AIRA-RFC-0104",
        "0104",
        "Phase K",
        "#209",
        "#210",
        "#211",
        "#212",
        "#213",
        "#214",
        "#215",
        "#216",
        "QUEUE K closed",
        "GPU marketplace",
        "LLM-in-Core",
        "blockchain",
        "POST /v1/problems",
        "confirmed free",
    ] {
        assert!(text.contains(needle), "RFC-0104 missing: {needle}");
    }
    assert!(
        text.contains("## 5. Non-Goals"),
        "RFC-0104 must list anti-mission as Non-Goals, not as deliverables"
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
    let gate_text =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/activate_gate.rs")).unwrap();
    for needle in [
        "fn bind_activate_gate",
        "fn enable_activated_mock_llm",
        "fn bind_phase_d_activate_from_root",
    ] {
        assert!(plane_text.contains(needle), "plane.rs missing: {needle}");
    }
    for needle in [
        "struct ActivatedPointerGate",
        "fn install_fixture",
        "content_hash mismatch",
        "evidence artifact missing",
    ] {
        assert!(
            gate_text.contains(needle),
            "activate_gate.rs missing: {needle}"
        );
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
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 214 | **DONE**"));
    assert!(queue.contains("| 215 | **DONE**"));
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

#[test]
fn phase_k_process_backend_215() {
    let lib = repo_root().join("csu/execution-llm/src/lib.rs");
    let process = repo_root().join("csu/execution-llm/src/process.rs");
    let lib_text = std::fs::read_to_string(&lib).unwrap();
    let process_text = std::fs::read_to_string(&process).expect("process.rs missing");
    for needle in [
        "fn with_process_backend",
        "fn with_backend_from_env",
        "fn missing_process_binary_is_capsule_failed",
        "fn backend_from_env_defaults_to_mock_not_process",
        "PROCESS_BACKEND_ID",
        "MISSING_BINARY",
        "AIRA_LLM_BACKEND",
        "pub use process::",
    ] {
        assert!(
            lib_text.contains(needle),
            "execution-llm lib missing: {needle}"
        );
    }
    for needle in [
        "struct ProcessBackend",
        "Command::new",
        "fn llama_cpp",
        "fn ollama",
        "MISSING_BINARY",
        "ENV_LLM_BACKEND",
        "network=none",
        "loopback",
    ] {
        assert!(
            process_text.contains(needle),
            "execution-llm process.rs missing: {needle}"
        );
    }
    assert!(
        !process_text.contains("Command::new(\"sh\")")
            && !process_text.contains("Command::new(\"bash\")")
            && !process_text.contains(".arg(\"-c\")"),
        "process backend must not spawn a shell"
    );
    assert!(
        !lib_text.contains("std::process"),
        "MockBackend path (lib.rs) must not shell out"
    );
    assert!(
        !lib_text.contains("Command::"),
        "MockBackend path (lib.rs) must not spawn Command"
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
        "with_mock_backend",
        "fn bind_process_backend",
        "fn enable_activated_mock_llm",
    ] {
        assert!(plane_text.contains(needle), "plane.rs missing: {needle}");
    }
    assert!(
        plane_text.contains(".with_mock_backend()"),
        "default plane register must keep MockBackend"
    );
    let flow_tests =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    for needle in [
        "fn default_plane_keeps_mock_backend",
        "fn missing_process_binary_on_plane_is_capsule_failed",
        "fn calculate_two_plus_two_stays_execution_basic",
        "fn non_math_prompt_completes_via_execution_llm_mock",
    ] {
        assert!(
            flow_tests.contains(needle),
            "aira-flow tests missing: {needle}"
        );
    }
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0110-process-backend.md");
    assert!(rfc.is_file(), "RFC-0110 must exist for #215");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("ProcessBackend"));
    assert!(rfc_text.contains("CapsuleFailed"));
    assert!(rfc_text.contains("MockBackend"));
    assert!(rfc_text.contains("loopback"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 215 | **DONE**"));
    assert!(queue.contains("| 216 | **DONE**"));
    assert!(!queue.contains("| 215 | **OPEN**"));
    assert!(!queue.contains("| 216 | **OPEN**"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #215 | Process backend"));
    assert!(status.contains("RFC-0110"));
    let local = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    assert!(
        !local.contains("execution-llm") && !local.contains("ExecutionLlmCsu"),
        "LocalSession must not independently construct execution-llm (plane.rs is the register site)"
    );
}

#[test]
fn phase_k_desktop_generate_216() {
    let work =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/work_view.rs")).unwrap();
    for needle in [
        "fn format_work_result",
        "fn executed_generate_local_leads_with_result_not_verified",
        "execution_artifact_id",
        "text.generate.local",
        "executed",
        "must not fake VERIFIED",
    ] {
        assert!(work.contains(needle), "work_view.rs missing: {needle}");
    }
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(i18n.contains("text.generate.local"));
    assert!(i18n.contains("never fakes VERIFIED") || i18n.contains("не підробляє VERIFIED"));
    assert!(i18n.contains("execution-basic"));
    let http =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop-runtime/src/node_http.rs"))
            .unwrap();
    assert!(http.contains("POST /v1/problems"));
    assert!(http.contains("posts_generate_local_and_parses_executed_not_verified"));
    let node_http =
        std::fs::read_to_string(repo_root().join("crates/aira-node/src/http/mod.rs")).unwrap();
    assert!(node_http.contains("http_post_problem_generate_without_activate_is_not_verified"));
    assert!(node_http.contains("http_post_problem_generate_with_activate_is_executed_not_verified"));
    let rfc = repo_root().join("specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md");
    assert!(rfc.is_file(), "RFC-0104 must exist for #216");
    let rfc_text = std::fs::read_to_string(&rfc).unwrap();
    assert!(rfc_text.contains("AIRA-RFC-0104"));
    assert!(rfc_text.contains("#209"));
    assert!(rfc_text.contains("#216"));
    assert!(rfc_text.contains("QUEUE K closed"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 216 | **DONE**"));
    assert!(!queue.contains("| 216 | **OPEN**"));
    assert!(queue.contains("QUEUE K closed"));
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #216 | Desktop + RFC-0104"));
    assert!(status.contains("RFC-0104"));
    assert!(status.contains("Phase K `#209`–`#216` **DONE**"));
    let gui = std::fs::read_to_string(repo_root().join("docs/desktop-gui.md")).unwrap();
    assert!(gui.contains("generate-local") || gui.contains("text.generate.local"));
    let analyze = repo_root().join("analysis/Analyze-251/LIVING_SPEC_MATRIX.md");
    assert!(analyze.is_file(), "Analyze-251 living spec missing");
}
