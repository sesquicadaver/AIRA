# Living Spec Matrix — Analyze-345 / QUEUE `#309`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| Pending/Ready/Failed на версію | `activate_gate` | `observe_ui_mismatch_fail_is_sticky_without_rehash_storm` |
| fail без rehash storm | `activate_gate` | same + `observe_fail_clears_when_version_changes_and_matches` |
| admit не послаблюється | `check_activated` | sticky test still full-hashes admit |
| RFC-0195 | specs/rfc | `phase_t_rfc_0195_present` |
| tip → `#310` | QUEUE / tips | `phase_t_queue_309_done_310_open` |
