# Analyze-276 — Relay integration (QUEUE #241)

## BRIEF

Dial path + signed prime relay ads + dual reservation SHOULD. RFC-0133.

## Living Spec Matrix

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| direct→NAT→relay order | `plan_dial_path` | `dial_path_orders_*` | **DONE** |
| Prime fail-closed | validate_aira_bind | `dial_path_rejects_non_prime` | **DONE** |
| Signed RelayAdvertisement | `RelayAdvertisement` | `relay_ad_sign_verify_and_store` | **DONE** |
| Dual reservation SHOULD | `select_relay_reservations` | `dual_reservation_*` | **DONE** |
| No auto-trust | trust_policy_allows | `untrusted_relay_ad_*` | **DONE** |
| RFC-D | `AIRA-RFC-0133-…` | `phase_n_rfc_0133_*` | **DONE** |
| NAT/relay Noise smoke | — | — | **OUT** (`#246`) |

## TODO_FIXME

- [x] relay_integrate + tests
- [x] RFC-0133 + QUEUE `#241` DONE
- deferred: Presence refresh → `#242`; NAT Noise → `#246`
