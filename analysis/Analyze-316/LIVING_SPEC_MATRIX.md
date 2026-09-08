# Living Spec Matrix — Analyze-316 / QUEUE `#281`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| apply target = current root | `apply_successful_probe` + `Keyring::load_node_identity` | `apply_rejects_wrong_root_replay_and_stale_apply_time` |
| durable replay | `peers/reachability_replay.json` | same (second apply → replayed) |
| apply-time freshness | `check_apply_time_freshness` + skew | ClockSkew / expired-at-apply |
| CLI apply-time now | `aira peer reachability check --result-json` | compile + print replay path |
| RFC-0170 | `specs/rfc/AIRA-RFC-0170-…` | `phase_q_rfc_0170_present` |
