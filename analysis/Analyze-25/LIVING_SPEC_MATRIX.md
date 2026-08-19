# Living Spec Matrix — Analyze-25

| ТЗ | Модуль | Тести |
|----|--------|-------|
| TrustStore.revoked CRL | `aira-object::crypto` | `trust_crl_revoke_blocks_readd_and_verify` |
| revoke / upsert reject | `TrustStore` | same |
| sync unloads revoked | `sync_trust_verifiers` | same |
| CLI trust revoke + list | `aira-cli` | compile |
| Docs | `docs/crypto.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
