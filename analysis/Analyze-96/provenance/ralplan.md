# Analyze-96 — ralplan (QUEUE #61)

## Architect
**CLEAR** — extend existing acquisition gate CSU only; add `GateDecision::Allow`; publish ALLOW decision + Event; sandbox `network=none`; no transfer.

## Critic
**APPROVE** — Done when matches D4.1; DENY `#60` regression tests retained; Out (quarantine/verify/activate/HTTP) respected; CLI exit 0 vs 2 split is explicit.

## Plan
1. RFC-0010 ALLOW gate semantics (amends RFC-0009 row for `auto_download=true`).
2. `request_download` branches ALLOW/DENY; shared publish helper.
3. Unit tests: ALLOW + no weights; DENY regressions.
4. CLI help + exit codes; analysis matrix + UltraQA.
