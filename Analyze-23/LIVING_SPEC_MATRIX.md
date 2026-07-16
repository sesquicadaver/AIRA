# Living Spec Matrix — Analyze-23

| ТЗ | Модуль | Тести |
|----|--------|-------|
| TrustEntry / TrustStore JSON | `aira-object::crypto` | `trust_store_peer_verify_without_signing_key` |
| register_trust_store / ensure_trust_defaults | `aira-object` | same + LocalSession paths |
| NodePaths::trust_json | `aira-flow::local` | open/submit call ensure_trust |
| CLI trust list/add/remove | `aira-cli` | compile + smoke |
| Docs | `docs/crypto.md`, `docs/local-node.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
