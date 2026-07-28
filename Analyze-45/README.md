# Analyze-45 — HTTP TLS + persist discovery

**Scope:** Optional TLS for `aira-node --http` + durable `.aira/discovery/registry.json`. No mTLS/authn, no DHT, no public-bind default change. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## RALPLAN-DR

### Principles
1. TLS is opt-in; plain HTTP remains default for local MVP
2. Discovery persistence is local-only (no federation)
3. Cert+key must be paired; self-signed helper for loopback smoke
4. Reuse existing HTTP router — transport only
5. Close A-19 deferred items that fit one micro

### Decision Drivers
1. QUEUE #11 names HTTP TLS / persist discovery
2. A-19 CODE_REVIEW deferred TLS + persist
3. Avoid aws-lc build friction (`tls-rustls-no-provider` + ring)

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. axum-server rustls + discovery JSON persist** | Matches axum ecosystem; testable | Extra dep |
| B. Terminate TLS only at reverse proxy (docs only) | Zero code | Leaves A-19 open |
| C. TLS + mTLS + auth in one PR | Complete security | Too wide |

**Chosen: A.** B invalid (no ship); C violates one-slice rule.

### Architect
- **Antithesis:** Embedding TLS in-process vs always proxy.
- **Tension:** Local DX vs crypto surface — mitigated by opt-in flags + self-signed under `.aira/http/`.
- **Synthesis:** Transport module in `aira-node`; discovery persist in `aira-protocol`.

### Critic
- Acceptance: persist roundtrip + TLS PEM load/serve smoke
- Anti-stub; clippy `-D warnings`
- **Verdict: APPROVE**

### Acceptance
- [ ] DiscoveryRegistry load/save under `.aira/discovery/registry.json`
- [ ] AppState seeds then persists
- [ ] `--tls-cert` + `--tls-key` HTTPS serve
- [ ] `--tls-self-signed` generates PEM under root
- [ ] docs/local-node.md + QUEUE #11 DONE for this micro
- [ ] Tests + clippy; CODE_REVIEW APPROVE/CLEAR

### Out
mTLS, bearer auth, DHT, changing default bind off loopback.
