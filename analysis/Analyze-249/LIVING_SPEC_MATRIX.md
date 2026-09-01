# Living spec — Analyze-249 (QUEUE #214)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| No activate / inactive model → CapsuleFailed | `ExecutionLlmCsu::check_activate` | `inactive_model_is_capsule_failed`; `never_activated_gate_is_capsule_failed` | **DONE** |
| Activated test double + MockBackend → CapsuleCompleted | `AlwaysActivated` | `mock_backend_completes_valid_generate_local`; `activated_mock_completes_without_model_artifact_ref` | **DONE** |
| Plane fail-closed without activate + Evidence | `OperationalPlane` default; evidence-basic | `generate_without_activate_is_capsule_failed` | **DONE** |
| Plane success with injected gate | `enable_activated_mock_llm`; `ActivatedPointerGate` | `non_math_prompt_completes_via_execution_llm_mock`; `phase_d_activated_pointer_allows_mock_generate` | **DONE** |
| C1 2+2 stays execution-basic | CapsuleCreated `math.eval.safe` | `calculate_two_plus_two_stays_execution_basic`; `c1.pipeline.calculate_2_plus_2` | **DONE** |
| CSU ↛ CSU | no inventory/acquisition Cargo dep | `phase_k_activate_gate_214` | **DONE** |
| RFC-D | `AIRA-RFC-0109-activate-gate.md` | `phase_k_activate_gate_214` | **DONE** |
| RFC-0104 reserved | no `AIRA-RFC-0104*` yet | `phase_k_rfc_0104_id_free` | **DONE** |
| QUEUE K | `#214` DONE, `#215`–`#216` OPEN | `phase_k_queue_wiring_209_done` | **DONE** |
| Process backend | ollama/llama.cpp CLI | — | **OUT** (`#215`) |
