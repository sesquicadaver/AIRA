# Living spec — Analyze-246 (QUEUE #211)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| execution-llm CSU | `csu/execution-llm` `aira:csu:execution.llm` | `manifest_is_execution_type`; `phase_k_execution_llm_211` | **DONE** |
| MockBackend completes generate-local | `MockBackend` + `with_mock_backend` | `mock_backend_completes_valid_generate_local` | **DONE** |
| Fail-closed without backend | `ExecutionLlmCsu::new` has no backend | `missing_backend_is_capsule_failed` | **DONE** |
| Strict payload | serde `deny_unknown_fields` + schema id/action const | `wrong_action_is_capsule_failed`; `extra_properties_fail_closed`; `missing_prompt_is_capsule_failed` | **DONE** |
| Not fake VERIFIED | outputs are CapsuleCompleted/Failed + ExecutionArtifact | assertions in named tests | **DONE** |
| RFC-D | `AIRA-RFC-0106-execution-llm-mock.md` | `phase_k_execution_llm_211` | **DONE** |
| RFC-0104 reserved | no `AIRA-RFC-0104*` yet | `phase_k_rfc_0104_id_free` | **DONE** |
| QUEUE K | `#211` DONE, `#212`–`#216` OPEN | `phase_k_queue_wiring_209_done` | **DONE** |
| Plane register | OperationalPlane | — | **OUT** (`#213`) |
| Reduction bind | reduction-basic | — | **OUT** (`#212`) |
