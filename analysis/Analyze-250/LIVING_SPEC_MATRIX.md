# Living spec — Analyze-250 (QUEUE #215)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Missing binary → CapsuleFailed | `ProcessBackend::resolve_program` | `missing_process_binary_is_capsule_failed`; `missing_process_binary_on_plane_is_capsule_failed` | **DONE** |
| MockBackend still plane/CI default | `OperationalPlane::open` `with_mock_backend` | `default_plane_keeps_mock_backend`; `mock_backend_completes_valid_generate_local`; `non_math_prompt_completes_via_execution_llm_mock` | **DONE** |
| Process backend selectable, not default CI | `with_process_backend`; `bind_process_backend`; `AIRA_LLM_BACKEND` | `backend_from_env_defaults_to_mock_not_process`; `phase_k_process_backend_215` | **DONE** |
| Activate gate before spawn | `check_activate` then `generate` | `missing_process_binary_does_not_skip_activate_gate` | **DONE** |
| Fixed argv, no shell | `Command::new` + args; never `sh -c` | `phase_k_process_backend_215` | **DONE** |
| network=none / no WAN | payload validate; no sockets in adapter | RFC-0110; payload `validate` | **DONE** |
| C1 2+2 stays execution-basic | CapsuleCreated `math.eval.safe` | `calculate_two_plus_two_stays_execution_basic`; `c1.pipeline.calculate_2_plus_2` | **DONE** |
| CSU ↛ CSU; no Core inference | no inventory/acquisition Cargo dep | `phase_k_process_backend_215` | **DONE** |
| RFC-D | `AIRA-RFC-0110-process-backend.md` | `phase_k_process_backend_215` | **DONE** |
| RFC-0104 reserved | no `AIRA-RFC-0104*` yet | `phase_k_rfc_0104_id_free` | **DONE** |
| QUEUE K | `#215` DONE, `#216` OPEN | `phase_k_queue_wiring_209_done` | **DONE** |
| Desktop Work generate | Work tab + RFC-0104 | — | **OUT** (`#216`) |
