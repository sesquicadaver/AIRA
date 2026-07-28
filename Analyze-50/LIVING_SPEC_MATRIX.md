# Living Spec Matrix — Analyze-50

| ТЗ / QUEUE | Модуль | Тести |
|------------|--------|-------|
| QUEUE #16 remote dual-key | `TrustStore::rekey` + previous_* | `trust_rekey_grace_allows_old_same_id` |
| apply rekey | `apply_trust_delta` | `notify_rekey_with_grace_keeps_old_pubkey` |
| mTLS Out | QUEUE #15 | n/a |
