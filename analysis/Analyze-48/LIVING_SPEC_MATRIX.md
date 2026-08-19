# Living Spec Matrix — Analyze-48

| ТЗ / QUEUE | Модуль | Тести |
|------------|--------|-------|
| QUEUE #13 HTTP authn (bearer micro) | `aira-node` `http::bearer_gate` + CLI | `http_bearer_*`, `bearer_credential_*` |
| `/health` exempt | `bearer_gate` path check | `http_bearer_health_exempt` |
| Default open API | no middleware when token unset | existing `http_*` suite |
| Docs | `docs/local-node.md`, `docs/crypto.md` | manual |
| mTLS Out | QUEUE #15 | n/a |
