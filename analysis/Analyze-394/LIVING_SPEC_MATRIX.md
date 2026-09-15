# Living Spec Matrix — Analyze-394 / RFC-0241

| ID | Requirement | Module | Test | Status |
|----|-------------|--------|------|--------|
| LS394-001 | Optional llm_* on settings 0.1 | `schemas/desktop/settings.schema.json` | schema fixtures + `desktop_settings_schema_loads` | **DONE** |
| LS394-002 | process requires ollama model | `settings::normalize_llm_settings` | `process_without_model_rejected` | **DONE** |
| LS394-003 | node spawn gets AIRA_LLM_* | `apply_node_llm_env` / `process::start` | `apply_node_llm_env_sets_process` | **DONE** |
| LS394-004 | parse ollama list | `ollama::parse_ollama_list_stdout` | `parse_skips_header_and_keeps_names` | **DONE** |
| LS394-005 | LLM change → RestartNeeded | `settings_apply` | `llm_backend_change_needs_restart` | **DONE** |
| LS394-006 | executor from node facts | `model_status` / `collect_status_snapshot` | `process_executor_is_not_reference_mock` | **DONE** |
| LS394-007 | no VERIFIED claim | docs/help + UI copy | help model.select | **DONE** |
| LS394-008 | host-ollama Phase D tip | `install_host_ollama_bind` | `host_ollama_bind_is_production_trust_ready_not_fixture` | **DONE** |
