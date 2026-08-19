# ULTRAQA — Analyze-88

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Model payload schema validates; missing hash fails; existing C0/C1 fixtures still pass.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | schema | load `$id` | present | `aira-schema` list_ids | PASS |
| U2 | valid | fixture | pass | `validate_fixtures` | PASS |
| U3 | invalid | missing content_hash | fail | invalid manifest entry | PASS |
| U4 | enum | ArtifactType | unchanged | git diff aira-artifact | PASS |
| U5 | cli | schema validate --fixtures | failed=0 | aira-cli | PASS |
