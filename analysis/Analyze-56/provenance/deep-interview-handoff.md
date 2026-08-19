# Deep-interview handoff — Analyze-56

**Date:** 2026-07-31  
User chose **A**: opt-in `--health-listen` plain HTTP, `GET /health` only, only when mTLS is on. Matches simplicity constraint; B/C rejected.

## Crystallized

| Item | Choice |
|------|--------|
| Activation | Opt-in `--health-listen <addr>`; requires `--tls-client-ca` |
| Transport | Plain HTTP (no TLS/mTLS on health bind) |
| Routes | Only `GET /health` |
| Main API | `--listen` HTTPS/mTLS unchanged |
| Fail closed | `--health-listen` without mTLS → error |
| Default addr (docs) | `127.0.0.1:8788` |
| Non-loopback | Fail closed until QUEUE #34 (tightened vs initial “warn” after architect WATCH) |

## Acceptance

1. `--health-listen` without mTLS → fail closed
2. With mTLS + health-listen → `/health` on health addr without client cert
3. Health router has no `/v1/*`
4. Docs + QUEUE #21; tests
