# Analyze-48 — Optional HTTP bearer authn

**Scope:** Opt-in `Authorization: Bearer` for `aira-node --http` (CLI `--http-token` / `AIRA_HTTP_TOKEN`). `/health` exempt. No mTLS. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## RALPLAN-DR

### Principles
1. Auth is opt-in; default remains open loopback (local MVP DX)
2. `/health` stays unauthenticated for probes
3. Constant-time token compare; no token echo in errors
4. Reuse existing router — middleware only; TLS stack unchanged
5. One micro: bearer only; mTLS remains deferred

### Decision Drivers
1. QUEUE #13 after HTTPS (#11 / A-45)
2. Explore: bearer fits axum oneshot tests; mTLS needs Rustls client-auth rewrite
3. Anti-pattern: do not ship mTLS + bearer in one Analyze

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Optional Bearer token middleware** | Small, testable, docs already name bearer | Not transport identity |
| B. mTLS client-cert require | Stronger bind to TLS | Large; server-only PEM today |
| C. Docs-only defer again | Zero risk | Leaves QUEUE #13 open |

**Chosen: A.** B = follow-up (#15); C invalid for autopilot delivery.

### Architect
- **Antithesis:** Always-on auth vs opt-in — opt-in preserves existing tests/scripts without token.
- **Tension:** Env var vs flag-only — synthesis: clap `--http-token` with `env = AIRA_HTTP_TOKEN`.
- **Critic: APPROVE**

### Acceptance
- [x] `--http-token` / `AIRA_HTTP_TOKEN` enables auth on `/v1/*`
- [x] Missing/wrong token → 401 JSON `{error}`; correct Bearer → handlers work
- [x] `/health` OK without token when auth enabled
- [x] No token configured → backward-compatible open API
- [x] docs + QUEUE #13 DONE for bearer micro; mTLS still Out
- [x] Tests + clippy; CODE_REVIEW APPROVE/CLEAR

### Delivered
- `AppState::with_http_token` + `bearer_gate` middleware
- CLI `--http-token` / `AIRA_HTTP_TOKEN` (clap `env` feature)
- Docs + A-19 TODO bearer closed; mTLS → QUEUE #15

### Out
mTLS, multi-tenant authz, OAuth/OIDC, changing default bind, peer Noise changes.
