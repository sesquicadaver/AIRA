# Living Spec Matrix — Analyze-38

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Rekey shape + apply issuer-only | `trust_delta` | `trust_delta_rekey_requires_issuer_subject_match` |
| Notify before rotate updates trust | `notify` + session | `notify_rekey_updates_peer_trust` |
| CLI `--notify-peers` / `notify-rekey` | `aira-cli` | clippy build |
| No gossip | scope | CODE_REVIEW |
