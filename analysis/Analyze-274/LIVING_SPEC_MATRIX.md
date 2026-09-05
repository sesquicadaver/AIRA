# Analyze-274 — Reachability states (QUEUE #239)

## BRIEF

State machine + `peers/reachability.json`. DIRECT only after verified probe. RFC-0131.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| UNKNOWN…OFFLINE enum | `ReachabilityStatus` | unit | **DONE** |
| reachability.json persist | `ReachabilityLocalState` | `successful_probe_sets_direct_and_persists` | **DONE** |
| bind ≠ DIRECT | `mark_local_bind` | `default_unknown_and_local_bind_never_direct` | **DONE** |
| probe → DIRECT | `apply_successful_probe` | persist test | **DONE** |
| relay/outbound/offline | `apply_direct_failed` | `direct_failed_with_relay_or_outbound_or_offline` | **DONE** |
| RFC-D | `AIRA-RFC-0131-…` | `phase_n_rfc_0131_*` | **DONE** |
| AddressBook promotion | — | — | **OUT** (`#240`) |

## TODO_FIXME

- [x] state + persistence
- [x] RFC-0131 + QUEUE `#239` DONE
- deferred: AddressBook → `#240`
