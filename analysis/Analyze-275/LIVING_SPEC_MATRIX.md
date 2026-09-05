# Analyze-275 — AddressBook promotion (QUEUE #240)

## BRIEF

Valid Presence + trust → AddressBook only; no auto-trust. RFC-0132.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Presence verify + trust gate | `promote_presence_to_address_book` | promote + untrusted | **DONE** |
| No TrustStore upsert | promote path | `rejects_untrusted_and_does_not_auto_trust` | **DONE** |
| Revoked reject | trust_policy_allows | `rejects_revoked_*` | **DONE** |
| Relay via | dial_target_from_presence | `relay_only_presence_sets_via` | **DONE** |
| RFC-D | `AIRA-RFC-0132-…` | `phase_n_rfc_0132_*` | **DONE** |
| Relay integration | — | — | **OUT** (`#241`) |

## TODO_FIXME

- [x] promote API + tests
- [x] RFC-0132 + QUEUE `#240` DONE
- deferred: relay → `#241`
