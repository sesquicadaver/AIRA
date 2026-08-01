# Analyze-59 — Concurrent accept (handshake off accept loop)

**QUEUE:** #24  
**Status:** CLOSED  
**Decision:** Option **C** — `accept_tcp` on the accept loop; spawn `complete_accept` (+ recv/relay)

## Shipped
- `aira_peer::{accept_tcp, complete_accept}`; `accept` remains composed
- CLI daemon `--recv` and `--relay`: handshake off accept loop; discovery/register only after success
- Tests: hung TCP; broken handshake; ≥2 parallel recv; relay hung-peer smoke

## Out
systemd (#25); persistent multi-envelope recv-loop; handshake concurrency semaphore (WATCH)
