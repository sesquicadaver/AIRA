# Living spec — Analyze-230 (QUEUE #195)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Unique nonce | `alloc_run_nonce` UUIDv7 | `alloc_run_nonce_concurrent_is_unique` | **DONE** |
| No racy file | ignore `run-counter` | `two_submits_allocate_distinct_problem_ids` | **DONE** |
| RFC | `AIRA-RFC-0093-run-nonce-uuidv7.md` | `phase_i_run_nonce_195` | **DONE** |
| Instance crypto | process OnceLock signer | — | **OUT** (`#196`) |
