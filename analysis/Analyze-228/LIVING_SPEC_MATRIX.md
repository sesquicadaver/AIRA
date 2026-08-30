# Living spec — Analyze-228 (QUEUE #193)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Clock trait | `aira_object::Clock` | `system_clock_is_not_the_mvp_fixed_timestamp` | **DONE** |
| FixedClock | `FixedClock::parse` / `set_clock` | `fixed_clock_now_is_the_installed_time`; `local_session_fixed_clock_stamps_artifacts` | **DONE** |
| Operational stamps | CSU `mvp_timestamp` → `now()` | `local_session_artifacts_are_not_all_mvp_fixed_timestamp` | **DONE** |
| RFC | `AIRA-RFC-0091-runtime-clock.md` | `phase_i_runtime_clock_193` | **DONE** |
| Envelope replay | expires_at / skew | — | **OUT** (`#194`) |
