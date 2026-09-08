# Living Spec Matrix — Analyze-319 / QUEUE `#284`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| port/bind checks before TrustStore/book writes | `invite::validate_peer_invite` + `import_invite` | `non_prime_addr_leaves_trust_untouched` |
| invite failure ≠ silent trust mutation | `import_invite` (preflight + rollback) | same |
| commit/rollback or explicit partial | trust snapshot rollback on book.save Err | unit path in `import_invite` |
| RFC-0173 | `specs/rfc/AIRA-RFC-0173-…` | `phase_q_rfc_0173_present` |
| tip → `#285` | QUEUE / phase-q-plan / `phase_q_doc` | `phase_q_queue_284_done_285_open` |
