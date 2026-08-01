# Living Spec Matrix — Analyze-59

| ТЗ / QUEUE | Модуль | Тести |
|------------|--------|-------|
| #24 ≥2 parallel session recv; accept not blocked by handshake | `session::{accept_tcp,complete_accept}`; CLI `peer listen` | `hung_tcp_does_not_block_accept_loop`; `broken_handshake_does_not_kill_listener`; `two_parallel_sessions_recv`; `relay_accept_tcp_spawn_survives_hung_peer` |
| Discovery only after auth | CLI daemon spawn after `complete_accept` | covered by hung/broken (no premature side effects on fail) |
| Compat `accept()` | `session::accept` | existing peer hello/envelope tests |
