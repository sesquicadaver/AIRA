# Analyze-292 — Living Spec Matrix

| ТЗ / atom | Модуль / артефакт | Тест | Статус |
|-----------|-------------------|------|--------|
| Submit off UI thread | `async_jobs::try_spawn_submit` | `second_submit_rejected_while_inflight` | **DONE** |
| Refresh off UI thread | `try_spawn_refresh` / `pump_async_jobs` | `second_refresh_*`, `refresh_generation_*` | **DONE** |
| repaint ≠ data | `STATUS_REFRESH_INTERVAL` + comments | `status_refresh_interval_is_independent_of_repaint` | **DONE** |
| Empty submit fail-closed | `run_submit_job` | `submit_empty_fails_closed_off_thread` | **DONE** |
| RFC-D | `AIRA-RFC-0148-…` | `phase_o_rfc_0148_present` | **DONE** |
