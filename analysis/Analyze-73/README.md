# Analyze-73 — CI schema/C0/C1 gate (QUEUE #38)

## Status
CLOSED (QUEUE #38 DONE @ a55f61a / PR #1).
GitHub Actions must run the AIRA contract already declared in README: schema fixture validation plus conformance C0 and C1. Fail the job on non-zero CLI exit. C2, schema semantics, signatures, and file splits stay out.

## Done when
CI is red on invalid fixtures or C0/C1 failure; green on the current tree (local `schema validate` + C0 + C1 already pass).

## Out
C2 in CI; changing schema pack meaning; Event/Artifact signature work (#39+); modularize (#46+).
