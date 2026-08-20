# Living Spec — Analyze-117 / #82

| ТЗ | Модуль | Тести |
|----|--------|-------|
| P1 peer supervise | `peer.rs` + `process.rs` | `peer_lifecycle` |
| PID/lock peer | `aira-peer.pid.json` / lock | start/stop clears |
| P0 no peer | ensure_peer no-op | `p0_does_not_start_peer` |
| Loopback-only bind | require_loopback_bind | fail non-loopback |
| RFC | RFC-0031 | — |
