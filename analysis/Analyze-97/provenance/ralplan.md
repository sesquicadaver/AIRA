# Analyze-97 — ralplan (QUEUE #62)

## Architect
**CLEAR** — extend acquisition CSU with `fetch_to_quarantine`; destination scoped under `models/quarantine`; gate via existing `request_download`; no inventory CSU dep.

## Critic
**APPROVE** — ALLOW prerequisite; URL reject; no verify/activate; CLI `--source` optional (gate-only without it).

## Plan
1. RFC-0011 quarantine local fetch.
2. `fetch_to_quarantine` + pointer + Event.
3. Manifest filesystem `scoped`.
4. CLI + unit/smoke tests.
