# Living spec — Analyze-263 (QUEUE #228)

Матриця відповідності ТЗ → модуль → тести. Попередній атом: [Analyze-262](../Analyze-262/LIVING_SPEC_MATRIX.md).

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Opt-in sandbox required | `csu/execution-llm/src/sandbox.rs`; `ProcessBackend::with_sandbox_required`; `AIRA_LLM_SANDBOX_REQUIRED` | `sandbox_required_from_opt_in_only`; `from_env_sandbox_required_opt_in` | **DONE** |
| Missing kernel fail-closed | parent Landlock ABI probe; `SANDBOX_REQUIRED` | `sandbox_required_missing_kernel_is_fail_closed`; `enforce_missing_kernel_is_fail_closed` | **DONE** |
| Missing kernel → CapsuleFailed | execution-llm CSU | `sandbox_required_missing_kernel_is_capsule_failed` | **DONE** |
| ollama exception | `SANDBOX_REQUIRED_LOOPBACK` | `sandbox_required_ollama_is_fail_closed`; `enforce_ollama_loopback_is_fail_closed` | **DONE** |
| Host can isolate | `/bin/echo` under required stack | `sandbox_required_echo_succeeds_or_fail_closed` | **DONE** |
| RFC-D | `AIRA-RFC-0121-sandbox-required.md` | `phase_m_sandbox_required_228` | **DONE** |
| RFC-0117 reserved | no `AIRA-RFC-0117*` | `phase_m_rfc_0117_id_free` | **DONE** |
| QUEUE `#228` DONE | first OPEN `#229` | `phase_m_queue_wiring_224_done`; `phase_m_next_problem` | **DONE** |
| C1 2+2 | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
| schema OS vs AIRA-mediated | — | — | **OUT** (`#229`) |
