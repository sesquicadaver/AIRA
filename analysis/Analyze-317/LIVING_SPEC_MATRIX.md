# Living Spec Matrix — Analyze-317 / QUEUE `#282`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| одна revision на Start/Stop | `lifecycle_revision` + `try_spawn_lifecycle` | `lifecycle_bumps_one_revision_and_invalidates_refresh` |
| refresh ≠ post-transition | refresh gate + `poll_refresh` drop | `refresh_rejected_while_lifecycle_inflight`, `poll_refresh_drops_while_lifecycle_inflight` |
| Quit-during-Start → Stop | `quit_followup_after_lifecycle` | `quit_followup_chains_stop_after_start` |
| Applied з worker snapshot | `StartOutcome.used_settings` | `applied_uses_worker_snapshot_not_later_ui_settings` |
| RFC-0171 | `specs/rfc/AIRA-RFC-0171-…` | `phase_q_rfc_0171_present` |
