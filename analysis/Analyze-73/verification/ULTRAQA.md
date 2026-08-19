# ULTRAQA — Analyze-73

**Verdict:** PASS (local contract commands; CI runner is the remaining host)  
**Date:** 2026-08-19

## Goal
Workflow fails closed on schema/C0/C1 failure; current tree passes those commands.

## Scenario matrix
| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | operator | `schema validate --fixtures fixtures` | failed=0, exit 0 | passed=26 failed=0 | PASS |
| U2 | operator | `conformance run --profile C0` | failed=0 | 5 passed | PASS |
| U3 | operator | `conformance run --profile C1` | failed=0 | 4 passed | PASS |
| U4 | contract | C2 not added to CI | no C2 step | `ci.yml` has C0/C1 only | PASS |
| U5 | fail-closed | invalid fixtures → CLI FAILURE | `report.failed > 0` → exit 1 | code path in `aira-cli` SchemaCommands::Validate | PASS (code) |
| U6 | fail-closed | C0/C1 `results.failed > 0` | ExitCode::FAILURE | ConformanceCommands::Run | PASS (code) |

C2 profile remains callable locally; it is not a merge gate in this row.
