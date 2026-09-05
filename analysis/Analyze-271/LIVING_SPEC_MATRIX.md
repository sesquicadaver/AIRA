# Analyze-271 — EVM ledger adapter (QUEUE #236)

## BRIEF

`EvmRendezvousProvider` + local double + Amoy/mainnet config hooks. No live RPC. RFC-0128. Not `#237`.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Local deterministic double | `EvmRendezvousProvider::local_double` | `local_double_roundtrip_and_kind` | **DONE** |
| Amoy hooks chain_id=80002 | `EvmRendezvousConfig::amoy_local_double` | `amoy_and_polygon_config_hooks` | **DONE** |
| Polygon hooks chain_id=137 | `polygon_mainnet_local_double` | `amoy_and_polygon_config_hooks` | **DONE** |
| Live RPC fail-closed | `validate` | `rejects_live_remote_without_237` | **DONE** |
| identity_hash helper | `evm_identity_hash` | roundtrip test | **DONE** |
| No aira-core ledger | Cargo / dep_firewall | unchanged | **DONE** |
| RFC-D | `AIRA-RFC-0128-evm-rendezvous.md` | `phase_n_rfc_0128_*` | **DONE** |
| Live publish/query | — | — | **OUT** (`#237`) |

## TODO_FIXME

- [x] EVM adapter + local double
- [x] RFC-0128 + QUEUE `#236` DONE
- deferred: live publish/query → `#237`
