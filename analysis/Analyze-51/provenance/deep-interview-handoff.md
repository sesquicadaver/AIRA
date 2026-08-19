# Deep-interview handoff — Analyze-51 mTLS

## Interview-complete rationale
User re-invoked `/autopilot` without selecting A/B. QUEUE #15 names **mTLS client-cert** — by definition mutual TLS means **require** a client certificate when enabled. Ambiguity below quick threshold (~0.15). Non-goals and decision boundaries resolved from QUEUE + A-45/A-48 patterns.

## Decisions (crystallized)
| Decision | Choice | Why |
|----------|--------|-----|
| Client auth mode | **Require** when `--tls-client-ca` set | Canonical mTLS; QUEUE wording |
| `/health` under mTLS | Also requires client cert | Handshake-level; no second listener in this micro |
| Enable condition | Only with HTTPS (`--tls-*`) | No mTLS over plain HTTP |
| Bearer interaction | Independent; both may apply | A-48 stays HTTP-layer |
| Default bind | Unchanged loopback | Out of scope |

## Non-goals
- Optional/anonymous client auth
- Separate plaintext health port
- Public bind default change
- Mapping client cert CN → TrustStore / multi-tenant authz
- Manifesto/Meditation edits

## Acceptance
1. `--tls-client-ca <pem>` + HTTPS → rustls requires client cert signed by that CA
2. Missing CA / without HTTPS → clear CLI error
3. Without `--tls-client-ca` → server-only TLS unchanged (A-45)
4. Unit/integration: build config with CA; reject config without client auth path documented
5. Docs + QUEUE #15 DONE

## Decision boundaries (agent may decide)
- Exact rustls/axum-server API wiring
- Test shape (config load + verifier vs live curl smoke)
- Flag name `--tls-client-ca` (matches ecosystem)
