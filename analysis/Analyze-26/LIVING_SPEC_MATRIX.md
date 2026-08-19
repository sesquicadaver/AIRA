# Living Spec Matrix — Analyze-26

| ТЗ | Модуль | Тести |
|----|--------|-------|
| TrustStore::unrevoke | `aira-object::crypto` | `trust_crl_unrevoke_allows_explicit_readd` |
| CryptoError::NotRevoked | `aira-object` | same |
| No auto re-trust | contract | same (verify UnknownKey until add) |
| CLI trust unrevoke | `aira-cli` | compile |
| Docs | `docs/crypto.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
