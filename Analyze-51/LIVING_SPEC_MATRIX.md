# Living Spec Matrix — Analyze-51

| ТЗ / QUEUE | Модуль | Тести |
|------------|--------|-------|
| QUEUE #15 mTLS require | `tls::build_server_config` + CLI | `mtls_*` |
| Fail-closed CA | `load_client_ca_roots` | `client_ca_empty_fails_closed` + UltraQA U1–U3 |
| Bearer independent | `http` middleware | `http_bearer_still_enforced_alongside_mtls_config` |
| Docs | local-node / crypto | manual |
