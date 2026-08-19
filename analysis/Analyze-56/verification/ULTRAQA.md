# ULTRAQA — Analyze-56

**Goal:** `--health-listen` plain `GET /health` only when mTLS; fail closed otherwise.  
**Verdict:** **PASS**

## Baseline

| Check | Result |
|-------|--------|
| `cargo test -p aira-node` | 32 passed |
| `cargo clippy -p aira-node -- -D warnings` | ok |

## Scenario matrix

| ID | Intent | Attacker/user | Setup | Expected | Actual | Cleanup |
|----|--------|---------------|-------|----------|--------|---------|
| S1 | Fail closed without mTLS | Misconfig | `--health-listen` no `--tls-client-ca` | non-zero + message | PASS | temp root |
| S2 | Plain probe works | Operator probe | mTLS + health listen | `{"status":"ok"}` HTTP | PASS | kill node |
| S3 | No API surface on health | Path probe | GET `/v1/capabilities` on health | 404 | PASS | |
| S4 | API still mTLS | Unauth HTTPS | GET API `/health` no client cert | curl fail | PASS (ec=56) | |
| S5 | Malformed addr | Bad flag | `--health-listen not-an-addr` | invalid + fail | PASS | |
| S6 | Flag needs `--http` | Flag alone | `--health-listen` no `--http` | require --http | PASS | |
| S7 | Non-loopback fail closed | Accidental public bind | unit `0.0.0.0:8788` | error contains loopback | PASS (unit) | |

## Hostile classes covered

- Malformed input: S5
- Misconfig / fail-closed: S1, S6
- Surface expansion: S3, S4
- Dirty worktree: not mutated outside Analyze-56 files

## Stop

PASS after cycle 1; no product defects.
