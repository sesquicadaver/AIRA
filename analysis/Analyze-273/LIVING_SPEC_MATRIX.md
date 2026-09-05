# Analyze-273 — Reachability Probe (QUEUE #238)

## BRIEF

Peer-assisted signed challenge + attestation; hairpin forbidden; replay/expiry. RFC-0130.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Signed challenge | `ReachabilityChallenge` | roundtrip + mutation | **DONE** |
| External probe attestation | `ReachabilityAttestation` | roundtrip | **DONE** |
| No hairpin | issue/verify | `rejects_hairpin_self_probe` | **DONE** |
| Expired / wrong binding | verify | `rejects_wrong_challenge_binding_and_expired` | **DONE** |
| Replay | `ReachabilityReplayLog` | roundtrip second verify | **DONE** |
| Schema/fixtures | challenge schema | fixture_manifest | **DONE** |
| RFC-D | `AIRA-RFC-0130-…` | `phase_n_rfc_0130_*` | **DONE** |
| State machine | — | — | **OUT** (`#239`) |

## TODO_FIXME

- [x] reachability probe module
- [x] RFC-0130 + QUEUE `#238` DONE
- deferred: states → `#239`
