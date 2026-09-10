# Living Spec Matrix — Analyze-352 / QUEUE `#316`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| shared create op | `aira-object` `create_or_ensure_node_identity` | `exclusive_create_then_second_create_fails_unchanged` |
| повторний create = error; bytes unchanged | `aira-object` CreateExclusive | same |
| incomplete pair no mint | `aira-object` | `incomplete_pair_rejected_without_mint` |
| Desktop ensure no-op / incomplete | `aira-desktop-runtime` bootstrap | `two_roots…`, `incomplete_identity_pair_is_fail_closed` |
| RFC-0201 | specs/rfc | `phase_u_rfc_0201_present` |
| tip → `#317` | QUEUE / tips | `phase_u_queue_316_done_317_open` |
