# Living Spec Matrix — Analyze-69

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Public HTTP bind opt-in | `aira-node` `--allow-public-bind` | `listen_rejects_public_without_flag`; `http_listen_public_without_flag_exits` |
| Fail-closed default | `assert_bind_allowed` | `listen_rejects_unspecified_v6_without_flag` |
| Loopback unchanged | `--listen` / `--health-listen` | `listen_allows_loopback_without_flag`; `health_listen_parses_loopback` |
| Health same policy | `resolve_health_listen` | `health_listen_rejects_non_loopback_without_flag`; allow-with-flag |
| Flag-only extra gate | public + no TLS | warning path (not fail); TLS/Bearer independent |
| Docs | `docs/local-node.md`, systemd example stays loopback | operator path |
