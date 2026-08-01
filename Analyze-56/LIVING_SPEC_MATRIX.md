# LIVING_SPEC_MATRIX — Analyze-56

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Opt-in `--health-listen` | `main.rs` / clap | CLI parse via serve path |
| Fail without mTLS | `resolve_health_listen` | `health_listen_requires_mtls` |
| Parse loopback addr | same | `health_listen_parses_loopback` |
| Reject non-loopback | same | `health_listen_rejects_non_loopback` |
| Router only `/health` | `health_router` | `health_router_only_health` |
| Docs | `docs/local-node.md`, `docs/crypto.md` | manual |
| QUEUE #21 | `QUEUE.md` | DONE |
