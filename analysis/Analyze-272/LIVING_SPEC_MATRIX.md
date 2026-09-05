# Analyze-272 — Publish/query (QUEUE #237)

## BRIEF

TTL/sequence/size/query policy over RendezvousProvider; local rendezvous.json; EVM call encoding. RFC-0129.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| publish/update/query_active/query_identity | `RendezvousClient` | `publish_query_update_with_ttl_and_state` | **DONE** |
| TTL bounds | `admit_record` | `rejects_ttl_out_of_bounds` | **DONE** |
| sequence via provider | update path | unit + mock | **DONE** |
| rendezvous.json | `RendezvousLocalState` | publish test | **DONE** |
| EVM call encode | `encode_evm_publish_call` | publish returns call | **DONE** |
| Works on EVM local double | client over EVM | `works_over_evm_local_double` | **DONE** |
| RFC-D | `AIRA-RFC-0129-…` | `phase_n_rfc_0129_*` | **DONE** |
| Reachability Probe | — | — | **OUT** (`#238`) |

## TODO_FIXME

- [x] RendezvousClient + policy + state
- [x] RFC-0129 + QUEUE `#237` DONE
- deferred: Reachability → `#238`
