# Living Spec Matrix — Analyze-27

| ТЗ | Модуль | Тести |
|----|--------|-------|
| TrustStore::rotate | `aira-object::crypto` | `trust_rotate_revokes_old_trusts_new` |
| supersedes / superseded_by | TrustEntry / RevokedEntry | same |
| NotTrusted / SameIdentity | CryptoError | same |
| CLI trust rotate | `aira-cli` | compile |
| Docs | `docs/crypto.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
