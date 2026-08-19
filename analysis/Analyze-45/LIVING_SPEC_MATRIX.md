# Living Spec Matrix — Analyze-45

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Persist discovery registry | `DiscoveryRegistry::{load,save}` | `discovery_persist_roundtrip` |
| AppState seeds + persists | `aira-node` http | `http_capabilities` |
| TLS PEM pair / self-signed | `tls` | `self_signed_loads_into_rustls_config`, `resolve_requires_pair` |
| CLI HTTPS flags | `aira-node` main | clippy build |
| Docs + QUEUE #11 | docs/QUEUE | CODE_REVIEW |
| No mTLS/DHT | scope | RULES |
