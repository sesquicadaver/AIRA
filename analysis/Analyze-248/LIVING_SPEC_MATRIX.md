# Living spec — Analyze-248 (QUEUE #213)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Plane register execution-llm | `OperationalPlane::open_with_ready_nonce` | `non_math_prompt_completes_via_execution_llm_mock`; `phase_k_plane_register_213` | **DONE** |
| C1 2+2 stays execution-basic | CapsuleCreated `math.eval.safe` | `calculate_two_plus_two_stays_execution_basic`; `c1.pipeline.calculate_2_plus_2` | **DONE** |
| Generate completes via MockBackend | CapsuleCompleted + ExecutionArtifact | `non_math_prompt_completes_via_execution_llm_mock` | **DONE** |
| No fake VERIFIED | no VRA / VerificationCompleted | `non_math_prompt_completes_via_execution_llm_mock`; `generate_local_output_is_not_verified` | **DONE** |
| Fan-out skip | execution-basic / execution-llm | `generate_local_action_is_skipped_for_plane_fan_out`; `math_eval_capsule_is_skipped_for_plane_fan_out` | **DONE** |
| RFC-D | `AIRA-RFC-0108-plane-register-execution-llm.md` | `phase_k_plane_register_213` | **DONE** |
| RFC-0104 reserved | no `AIRA-RFC-0104*` yet | `phase_k_rfc_0104_id_free` | **DONE** |
| QUEUE K | `#213` DONE, `#214`–`#216` OPEN | `phase_k_queue_wiring_209_done` | **DONE** |
| Activate gate | execution-llm `TODO(#214)` | — | **OUT** (`#214`) |
