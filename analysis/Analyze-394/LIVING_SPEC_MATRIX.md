# Living Spec Matrix — Analyze-394 / RFC-0241 + RFC-0242 + RFC-0243

| ID | Requirement | Module | Test | Status |
|----|-------------|--------|------|--------|
| LS394-001 | Optional llm_* on settings 0.1 | `schemas/desktop/settings.schema.json` | schema fixtures + `desktop_settings_schema_loads` | **DONE** |
| LS394-002 | process requires ollama model | `settings::normalize_llm_settings` | `process_without_model_rejected` | **DONE** |
| LS394-003 | node spawn gets AIRA_LLM_* | `apply_node_llm_env` / `process::start` | `apply_node_llm_env_sets_process` | **DONE** |
| LS394-004 | parse ollama list | `ollama::parse_ollama_list_stdout` | `parse_skips_header_and_keeps_names` | **DONE** |
| LS394-005 | LLM change → RestartNeeded | `settings_apply` | `llm_backend_change_needs_restart` | **DONE** |
| LS394-006 | executor from node facts | `model_status` / `collect_status_snapshot` | `process_executor_is_not_reference_mock` | **DONE** |
| LS394-007 | no VERIFIED claim | docs/help + UI copy | help model.select | **DONE** |
| LS394-008 | host-ollama activate tip | `install_host_ollama_bind` | `host_ollama_bind_is_production_trust_ready_not_fixture` | **DONE** |
| LS394-009 | Work requires host LLM; no math escape | `evaluate_work_readiness` / `evaluate_host_llm_gate` | `mock_settings_block_even_math_text` | **DONE** |
| LS394-010 | C1 process Executed ≠ VERIFIED | `aira-conformance` c1 | `c1.pipeline.process_executor_executed` | **DONE** |
| LS394-011 | OP-001 math legacy banner | RFC-0242 + `csu/execution-basic/LEGACY.md` | docs | **DONE** |
| LS394-012 | Per-request host_cli_model argv | `ProcessBackend::generate` / `ExecutorFacts` | `per_request_host_cli_model_rewrites_ollama_argv` | **DONE** |
| LS394-013 | host-ollama ≠ verified | `list_model_lifecycle` | `host_ollama_lifecycle_is_available_not_verified` | **DONE** |
| LS394-014 | collision-resistant ollama refs | `host_ollama_model_ref` | slash vs underscore assert in activate_gate | **DONE** |
| LS394-015 | attach applied=pidfile | `settings_apply` / `try_attach` | `attach_applied_llm_from_pidfile_not_disk` | **DONE** |
| LS394-016 | catalog mutate vs work_inflight | `catalog_mutate_allowed` | `catalog_mutate_blocked_during_work` | **DONE** |
| LS394-017 | atomic materialize | `materialize_weights_nofollow` | materialize unit tests | **DONE** |
| LS394-018 | Quit only after Stop ok | `quit_followup_after_lifecycle` | quit_followup tests | **DONE** |
| LS394-019 | optional process timeout | `llm_process_timeout_ms` / `submit_timeout_for` | schema + settings | **DONE** |
| LS394-020 | Required A→B distinct host_cli | `ActivatedPointerGate::check_activated` | `required_host_ollama_a_then_b_admit_distinct_host_cli` | **DONE** |
| LS394-021 | Compare B fail keeps A | `WorkJobResult::compare` | `compare_b_fail_preserves_leg_a` | **DONE** |
| LS394-022 | Ollama Select ≠ activate_verified | `select_catalog_model` | `select_host_ollama_skips_activate_verified` | **DONE** |
| LS394-023 | ollama list hang → timeout | `list_ollama_models_with_timeout` | `list_ollama_models_timeout_fail_closed` | **DONE** |
| LS394-024 | collision-resistant named test | `host_ollama_model_ref` | `host_ollama_model_ref_is_collision_resistant` | **DONE** |
| LS394-025 | R1 readiness = GUI settings | `evaluate_work_readiness(&settings)` / `load_settings_readonly` | `readiness_uses_system_layout_settings_not_data_root_file` | **DONE** |
| LS394-026 | R2 trusted host_cli | `resolve_trusted_host_cli` | `tampered_pointer_host_ollama_model_is_fail_closed` | **DONE** |
| LS394-027 | R3 file weights ≠ ollama tip | `ProcessBackend::generate` | `file_weight_binding_on_ollama_backend_is_fail_closed` | **DONE** |
