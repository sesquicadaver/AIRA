# Living Spec Matrix — Analyze-353 / QUEUE `#317`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| unique policy event IDs | `aira-policy` PolicyGate | `policy_event_ids_include_run_nonce`, `two_gates_…` |
| two submits → both policy events durable | `aira-flow` persist | `two_submits_persist_distinct_policy_event_ids` |
| same-ID≠hash → conflict | `admit_persisted_event` | `persist_rejects_same_id_different_hash_equivocation` |
| RFC-0202 | specs/rfc | `phase_u_rfc_0202_present` |
| tip → `#318` | QUEUE / tips | `phase_u_queue_317_done_318_open` |
