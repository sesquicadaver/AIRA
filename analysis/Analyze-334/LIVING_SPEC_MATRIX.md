# Living Spec Matrix — Analyze-334 / QUEUE `#298`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| mint only if both absent | `bootstrap::ensure_local_identity` | existing two-root + legacy |
| orphan json → error, no mint | same | `incomplete_identity_pair_is_fail_closed` |
| orphan secret → error, no mint | same | same |
| secret 0600 fail-closed | `write_secret_create_new` | `minted_identity_secret_is_owner_rw_only` |
| RFC-0185 | `specs/rfc/AIRA-RFC-0185-…` | `phase_s_doc` |
| tip → first OPEN `#299` | QUEUE / plan / tips | `phase_s_queue_298_done_299_open` |
