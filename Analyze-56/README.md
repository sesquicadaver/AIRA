# Analyze-56 — Separate health listener (no client cert)

**QUEUE:** #21  
**Decision:** **A** (`--health-listen` plain HTTP `/health` when mTLS)  
**Status:** CLOSED (`6a956b5`)

## What shipped

- Opt-in `--health-listen <addr>` on `aira-node --http`
- Fail-closed without `--tls-client-ca`
- Plain HTTP bind serving **only** `GET /health` (`health_router`)
- Main `--listen` remains HTTPS/mTLS unchanged
- Fail closed on non-loopback health bind until QUEUE #34; docs in `docs/local-node.md`

## Out

HTTPS health without mTLS; auto-port; public bind (#34); #22+.
