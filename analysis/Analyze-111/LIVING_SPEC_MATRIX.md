# Living Spec — Analyze-111 / #76

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Shared lifecycle | `crates/aira-desktop-runtime` | `tests/lifecycle.rs` |
| CLI surface | `aira-cli` `desktop` | smoke start/status/stop |
| Bearer token auth | bootstrap + `aira-node --http-token` | start path |
| Idempotent attach | `process::start` | `start_idempotent_attach_and_stop` |
| Stale PID | `process::start` | `stale_pid_recovered_on_start` |
| Port conflict | `process::start` | `port_conflict_fails_closed` |
| RFC-E | `AIRA-RFC-0025` | — |
