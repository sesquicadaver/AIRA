# ULTRAQA — Analyze-90

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Local inventory payload schema validates; missing `signature` fails; prior model fixtures still pass.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | schema | load `$id` | present | `aira-schema` list_ids | PASS |
| U2 | valid | fixture | pass | `validate_fixtures` | PASS |
| U3 | invalid | missing signature | fail | invalid manifest entry | PASS |
| U4 | enum | ArtifactType | unchanged | no aira-artifact diff | PASS |
| U5 | cli | schema validate --fixtures | failed=0 (32 passed) | aira-cli | PASS |
| U6 | regression | artifact+profile fixtures | still pass | U2 | PASS |
| U7 | scope | scan/list CLI | absent | no aira-cli inventory cmds | PASS |
