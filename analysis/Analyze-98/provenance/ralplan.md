# Analyze-98 — ralplan (QUEUE #63)

## Architect
**CLEAR** — extend acquisition CSU with `verify_quarantine`; signed ModelArtifact is expected-hash + signature source; staging under `models/verified`.

## Critic
**APPROVE** — reject leaves quarantine; TESTSIG/missing sig = unsigned; no activate/inventory.

## Plan
1. RFC-0012 verify semantics.
2. `verify_quarantine` + CLI `models verify`.
3. Unit tests: pass / hash mismatch / unsigned.
