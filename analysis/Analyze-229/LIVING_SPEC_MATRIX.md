# Living spec — Analyze-229 (QUEUE #194)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| expires_at | `admit_envelope` | `expired_envelope_is_rejected`; `recv_envelope_rejects_expired` | **DONE** |
| Clock skew | `|now − created_at|` | `skewed_created_at_is_rejected`; `admit_received_rejects_clock_skew` | **DONE** |
| Replay window | `message_id` + TTL | `duplicate_message_id_is_rejected_within_window`; `recv_envelope_rejects_replayed_message_id` | **DONE** |
| RFC | `AIRA-RFC-0092-envelope-freshness-replay.md` | `phase_i_envelope_freshness_194` | **DONE** |
| Run nonce | run-counter | — | **OUT** (`#195`) |
