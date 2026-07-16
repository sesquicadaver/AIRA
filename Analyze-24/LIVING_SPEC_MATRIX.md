# Living Spec Matrix — Analyze-24

| ТЗ | Модуль | Тести |
|----|--------|-------|
| sync_trust_verifiers prune + re-register | `aira-object::crypto` | `trust_store_peer_verify_without_signing_key` |
| ensure_trust_defaults uses sync | `aira-object` | same + LocalSession |
| CLI trust remove → sync | `aira-cli` | smoke / compile |
| Docs | `docs/crypto.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
