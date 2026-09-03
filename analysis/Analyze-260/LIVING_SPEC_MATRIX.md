# Living spec — Analyze-260 (QUEUE #225)

Матриця відповідності ТЗ → модуль → тести (оновлюється з фічею). Попередній wiring: [Analyze-259](../Analyze-259/LIVING_SPEC_MATRIX.md).

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Opt-in Landlock FS | `csu/execution-llm/src/landlock.rs`; `ProcessBackend::with_landlock`; `AIRA_LLM_LANDLOCK` | `landlock_enabled_from_opt_in_only`; `from_env_landlock_opt_in` | **DONE** |
| Child `pre_exec` restrict | `process.rs` `pre_exec`; `PR_SET_NO_NEW_PRIVS` | `phase_m_landlock_225` | **DONE** |
| Deny read outside allowlist | jail vs sibling secret | `landlock_denies_read_outside_allowlist` | **DONE** |
| Deny → CapsuleFailed | execution-llm CSU | `landlock_denied_read_is_capsule_failed` | **DONE** |
| Allowlist success | echo-only jail script | `landlock_echo_in_jail_succeeds` | **DONE** |
| Fail-closed | `LANDLOCK_FAILED` / `LANDLOCK_UNSUPPORTED` | `phase_m_landlock_225`; spawn map | **DONE** |
| RFC-D | `AIRA-RFC-0118-landlock-fs.md` | `phase_m_landlock_225` | **DONE** |
| RFC-0117 reserved | no `AIRA-RFC-0117*` | `phase_m_rfc_0117_id_free` | **DONE** |
| QUEUE `#225` DONE | first OPEN `#226` | `phase_m_queue_wiring_224_done`; `phase_m_next_problem` | **DONE** |
| C1 2+2 | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
| seccomp | — | — | **OUT** (`#226`) |
