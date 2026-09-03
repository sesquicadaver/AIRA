# Living spec — Analyze-255 (QUEUE #220)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Cap during read | `read_bounded` | `read_bounded_overflow_during_read` | **DONE** |
| stdout overflow fail-closed | `ProcessBackend::generate` | `stdout_overflow_during_read_is_fail_closed` | **DONE** |
| stderr overflow fail-closed | `ProcessBackend::generate` | `stderr_overflow_during_read_is_fail_closed` | **DONE** |
| CapsuleFailed not VERIFIED | `execution-llm` | `stdout_overflow_is_capsule_failed` | **DONE** |
| C1 2+2 unchanged | execution-basic | `calculate_two_plus_two_stays_execution_basic` | **DONE** |
| RFC-0111 reserved | no file | `phase_l_rfc_0111_id_free` | **DONE** |
