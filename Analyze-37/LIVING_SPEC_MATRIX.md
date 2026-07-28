# Living Spec Matrix — Analyze-37

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Multi-key verify same key_ref | `Keyring` | `node_rotate_grace_allows_old_until` |
| Cutover without `--until` | `rotate_node_signing_secret` | `node_signing_secret_rotate_cutover` |
| Grace until future | identity JSON + load | `node_rotate_grace_allows_old_until` |
| Expired grace drops old | `load_node_identity` | same test (past until) |
| Bad `--until` fail-closed | rotate | `node_rotate_rejects_bad_grace_until` |
| CLI `--until` | `aira-cli` | clippy build |
