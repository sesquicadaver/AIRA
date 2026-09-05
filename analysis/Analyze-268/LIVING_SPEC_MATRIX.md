# Analyze-268 — Deterministic port selection (QUEUE #233)

## BRIEF

Implement `preferred_port(identity, transport_class)` over `P_AIRA` with collision walk + finite wrap. RFC-0125. Do not implement Presence (`#234`).

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| same identity → same preferred | `preferred_port` | `same_identity_same_preferred_port` | **DONE** |
| collision → next prime | `select_available_port` | `collision_walks_next_prime` | **DONE** |
| full wrap finite | `select_available_port` | `full_wrap_is_finite_and_errors` | **DONE** |
| RFC-D | `AIRA-RFC-0125-preferred-port.md` | `phase_n_rfc_0125_*` | **DONE** |
| QUEUE advance | `#233` DONE; first OPEN `#234` | `phase_n_queue_233_done` | **DONE** |
| Presence | — | — | **OUT** (`#234`) |

## TODO_FIXME

- [x] preferred_port API + tests
- [x] RFC-0125 + QUEUE `#233` DONE
- deferred: Presence → `#234`
