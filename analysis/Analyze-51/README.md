# Analyze-51 — Optional mTLS (require client cert)

**Scope:** `--tls-client-ca <pem>` with HTTPS → rustls **requires** client cert (canonical mTLS). `/health` also requires cert (handshake-level). No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## Deep-interview
See `provenance/deep-interview-handoff.md` — mode **require**.

## RALPLAN-DR
Option **A** (require). Architect+Critic APPROVE — see `provenance/ralplan-consensus-gate.md`.

### Acceptance
- [x] `--tls-client-ca` requires HTTPS; error without TLS
- [x] Invalid/empty CA PEM → fail closed
- [x] mTLS ServerConfig: require client cert; ALPN set
- [x] Without `--tls-client-ca` → server-only TLS via `build_server_config`
- [x] Startup warning: mTLS requires client cert for all routes incl. `/health`
- [x] Tests: reject no-cert / wrong-CA; accept valid client cert
- [x] Bearer+mTLS coexistence unit
- [x] Docs + QUEUE #15 DONE; clippy clean

### Delivered
- `build_server_config` / `load_client_ca_roots` / `serve_https(..., client_ca)`
- CLI `--tls-client-ca`
- Handshake triad + docs

### Out
Optional client auth; CN→identity; public bind; second health listener.
