# Living Spec Matrix — Analyze-314 / QUEUE `#279`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| Окремі local/external/relay observations | `reachability_state.rs` clocks + `status_observation_at` | `local_bind_does_not_refresh_*` |
| Bind не оновлює external checked_at | `mark_local_bind` | peer + `local_bind_does_not_make_stale_direct_current` |
| Future clock ≠ Current без skew | `NETWORK_OBSERVATION_MAX_SKEW_SECS` | `future_clock_*` |
| RFC-0168 | `specs/rfc/AIRA-RFC-0168-…` | `phase_q_rfc_0168_present` |
