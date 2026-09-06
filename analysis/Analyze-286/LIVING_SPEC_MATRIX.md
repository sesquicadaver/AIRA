# Analyze-286 — Expiry before promote (QUEUE #251)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Promote rejects expired | `promote_presence_to_address_book(..., as_of)` | `rejects_expired_presence_even_when_trusted` | **DONE** |
| Ab ovo via query_identity | `discover_admit_promote` | `expired_presence_via_query_identity_cannot_promote` | **DONE** |
| Book stays empty | AddressBook | same | **DONE** |
| RFC-D | `AIRA-RFC-0143-…` | `phase_n_fix_*` | **DONE** |
| netns NAT | — | — | **OUT** (`#252`) |
