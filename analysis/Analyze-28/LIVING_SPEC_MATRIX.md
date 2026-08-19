# Living Spec Matrix — Analyze-28

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RevokedEntry.grace_until | `aira-object::crypto` | `trust_rotate_grace_allows_old_until` |
| rotate(..., grace_until) | `TrustStore` | same + rotate without until |
| to_keyring_at / sync grace | crypto | same |
| CLI `--until` | `aira-cli` | compile |
| Docs | `docs/crypto.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
