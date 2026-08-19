# ULTRAQA — Analyze-84

**Verdict:** PASS (local)  
**Date:** 2026-08-19

## Goal
HTTP API behavior unchanged after file split.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | health | GET /health | 200 ok | `http_health` | PASS |
| U2 | plane | POST problem 2+2 | completed | `http_post_problem_2_plus_2` | PASS |
| U3 | bearer | missing/wrong/ok / health exempt | 401/401/200/200 | `http_bearer_*` | PASS |
| U4 | tenant | register cross-forbidden; list filter | 403 / 1 vs 2 | tenant_* tests | PASS |
| U5 | clippy | aira-node -D warnings | green | cargo clippy -p aira-node | PASS |
