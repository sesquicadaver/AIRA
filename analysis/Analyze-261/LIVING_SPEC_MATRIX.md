# Living spec — Analyze-261 (QUEUE #226)

Матриця відповідності ТЗ → модуль → тести. Попередній атом: [Analyze-260](../Analyze-260/LIVING_SPEC_MATRIX.md).

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Opt-in seccomp | `csu/execution-llm/src/seccomp.rs`; `ProcessBackend::with_seccomp`; `AIRA_LLM_SECCOMP` | `seccomp_enabled_from_opt_in_only`; `from_env_seccomp_opt_in` | **DONE** |
| Child `pre_exec` filter | `SECCOMP_SET_MODE_FILTER`; `PR_SET_NO_NEW_PRIVS` | `phase_m_seccomp_226` | **DONE** |
| Forbidden syscall fail-closed | socket probe vs deny-list | `seccomp_forbidden_syscall_is_fail_closed` | **DONE** |
| Deny → CapsuleFailed | execution-llm CSU | `seccomp_forbidden_syscall_is_capsule_failed` | **DONE** |
| Harmless path still works | `/bin/echo` under seccomp | `seccomp_echo_succeeds` | **DONE** |
| Fail-closed install | `SECCOMP_FAILED` / `SECCOMP_UNSUPPORTED` | spawn map; `phase_m_seccomp_226` | **DONE** |
| RFC-D | `AIRA-RFC-0119-seccomp.md` | `phase_m_seccomp_226` | **DONE** |
| RFC-0117 reserved | no `AIRA-RFC-0117*` | `phase_m_rfc_0117_id_free` | **DONE** |
| QUEUE `#226` DONE | first OPEN `#227` | `phase_m_queue_wiring_224_done`; `phase_m_next_problem` | **DONE** |
| C1 2+2 | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
| netns | — | — | **OUT** (`#227`) |
