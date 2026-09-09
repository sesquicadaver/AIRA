# Living Spec Matrix — Analyze-337 / QUEUE `#301`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| stopped + P1 + unknown → Start | `connection_cta` | `stopped_peer_profile_unknown_is_start_not_refresh` |
| stopped + local-only → Start | same | `stopped_peer_profile_local_only_is_start` |
| failed + offline → Start | same | `failed_peer_profile_is_start_not_refresh` |
| running unknown still Refresh | same | `running_unknown_still_refresh` |
| RFC-0188 | specs/rfc | `phase_s_doc` |
| tip → `#302` | QUEUE / tips | `phase_s_queue_301_done_302_open` |
