# Analyze-270 — RendezvousProvider (QUEUE #235)

## BRIEF

`RendezvousProvider` trait + in-memory mock. No ledger logic in `aira-core`. RFC-0127. Not EVM adapter (`#236`).

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Trait API (publish/update/query/remove) | `rendezvous.rs` | `mock_publish_query_update_expire` | **DONE** |
| Mock CI double | `MockRendezvousProvider` | unit tests | **DONE** |
| Signed admit only | mock `admit` | `mock_rejects_unsigned_and_does_not_trust` | **DONE** |
| DISCOVERED ≠ TRUSTED | no TrustStore upsert | `mock_rejects_unsigned_and_does_not_trust` | **DONE** |
| Sequence increase on update | mock update | `mock_update_requires_higher_sequence` | **DONE** |
| No aira-core ledger deps | dep_firewall / Cargo | unchanged Core deps | **DONE** |
| RFC-D | `AIRA-RFC-0127-rendezvous-provider.md` | `phase_n_rfc_0127_*` | **DONE** |
| EvmRendezvousProvider | — | — | **OUT** (`#236`) |

## TODO_FIXME

- [x] trait + mock + tests
- [x] RFC-0127 + QUEUE `#235` DONE
- deferred: EVM adapter → `#236`
