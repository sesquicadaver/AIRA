# Living spec — Analyze-262 (QUEUE #227)

Матриця відповідності ТЗ → модуль → тести. Попередній атом: [Analyze-261](../Analyze-261/LIVING_SPEC_MATRIX.md).

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Opt-in netns | `csu/execution-llm/src/netns.rs`; `ProcessBackend::with_netns`; `AIRA_LLM_NETNS` | `netns_enabled_from_opt_in_only`; `from_env_netns_opt_in` | **DONE** |
| Child `pre_exec` unshare | `CLONE_NEWNET` after user ns maps; before Landlock/seccomp | `phase_m_netns_227` | **DONE** |
| Host loopback isolation | connect probe vs empty netns | `netns_isolates_host_loopback` | **DONE** |
| Isolate → CapsuleFailed | execution-llm CSU | `netns_isolated_connect_is_capsule_failed` | **DONE** |
| ollama exception fail-closed | `NETNS_BLOCKS_LOOPBACK` before spawn | `ollama_with_netns_is_fail_closed`; `from_env_ollama_netns_is_fail_closed` | **DONE** |
| ollama → CapsuleFailed | execution-llm CSU | `ollama_netns_is_capsule_failed` | **DONE** |
| Harmless offline path | `/bin/echo` under netns | `netns_echo_succeeds_or_fail_closed` | **DONE** |
| Fail-closed apply | `NETNS_FAILED` / `NETNS_UNSUPPORTED` | spawn map; `phase_m_netns_227` | **DONE** |
| RFC-D | `AIRA-RFC-0120-netns.md` | `phase_m_netns_227` | **DONE** |
| RFC-0117 reserved | no `AIRA-RFC-0117*` | `phase_m_rfc_0117_id_free` | **DONE** |
| QUEUE `#227` DONE | first OPEN `#228` | `phase_m_queue_wiring_224_done`; `phase_m_next_problem` | **DONE** |
| C1 2+2 | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
| missing-kernel policy | — | — | **OUT** (`#228`) |
