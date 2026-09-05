# Analyze-269 — Presence Record (QUEUE #234)

## BRIEF

`NodePresenceRecord` schema + canonical Ed25519 sign/verify + mutation tests. RFC-0126. Not RendezvousProvider (`#235`).

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Schema + fixtures | `presence-record.schema.json` | `fixture_manifest_passes` | **DONE** |
| Canonical sign/verify | `presence.rs` | `sign_and_verify_roundtrip` | **DONE** |
| Mutation tests | `presence.rs` | `mutation_breaks_verify` | **DONE** |
| P_AIRA on endpoints | `validate_shape` | `rejects_non_prime_direct_port` | **DONE** |
| RFC-D | `AIRA-RFC-0126-presence-record.md` | `phase_n_rfc_0126_*` | **DONE** |
| RendezvousProvider | — | — | **OUT** (`#235`) |

## TODO_FIXME

- [x] presence module + schema
- [x] RFC-0126 + QUEUE `#234` DONE
- deferred: RendezvousProvider → `#235`
