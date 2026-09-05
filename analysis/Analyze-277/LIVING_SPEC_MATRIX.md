# Analyze-277 — Presence refresh (QUEUE #242)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| sequence++ + TTL window | `refresh_and_sign_presence` | `refresh_bumps_*` | **DONE** |
| expire stale | `retain_unexpired_presence` | `expire_stale_*` | **DONE** |
| endpoint change | `endpoint_change_and_sign_*` | `endpoint_change_*` | **DONE** |
| notify target list | `trusted_peers_to_notify` | `notify_list_*` | **DONE** |
| RFC-D | `AIRA-RFC-0134-…` | `phase_n_rfc_0134_*` | **DONE** |
| CLI dial notify | — | — | **OUT** (`#243`) |

## TODO_FIXME

- [x] presence_refresh + tests
- [x] RFC-0134 + QUEUE `#242` DONE
- deferred: CLI → `#243`
