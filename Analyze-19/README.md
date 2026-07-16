# Analyze-19 — Local HTTP API (Roadmap M11)

**Scope:** Post-MVP after Issue Set #80 — `AIRA MVP Implementation Roadmap` §18 Milestone M11

## Ralplan (APPROVED — consensus)

### Principles
1. Local-only bind (`127.0.0.1`); no auth/multi-tenant/federation
2. Endpoints map to existing `LocalSession` / registry / conformance / discovery
3. Extend `aira-node` with `--http` / `--listen` (keep `--text` one-shot)
4. JSON request/response; deterministic errors
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### Decision Drivers
1. Roadmap M11 is the next concrete unimplemented milestone after alpha
2. Reuse CLI semantics so HTTP is a thin transport, not a second runtime
3. Default bind loopback only — reject accidental public exposure in docs/defaults

### ADR
- **Decision:** HTTP surface lives in `aira-node` (`src/http.rs`) behind `--http`
- **Why:** Node binary already owns root/config/CSU load; avoids a new crate for MVP
- **Alternatives:** separate `aira-http` crate (deferred); embed in `aira-cli` (wrong UX)
- **Consequences:** `aira-node` gains axum/tokio; CLI remains primary interactive tool
- **Follow-ups:** TLS/auth, YAML config, SQLite event log — not this cycle

### Endpoints
```text
POST /v1/problems
GET  /v1/problems/{id}
GET  /v1/results/{id}
GET  /v1/artifacts/{id}
GET  /v1/events
GET  /v1/capabilities
GET  /v1/csu
POST /v1/csu/register
POST /v1/conformance/run
GET  /health
```

### Acceptance
- All listed endpoints work against an initialized `.aira` root
- `POST /v1/problems` with `{"text":"Calculate 2 + 2"}` returns verified result
- Conformance run via HTTP returns report summary
- `cargo test -p aira-node` + workspace clippy PASS; originals untouched

### Out of scope
Auth hardening, TLS, public bind by default, federation API, web UI
