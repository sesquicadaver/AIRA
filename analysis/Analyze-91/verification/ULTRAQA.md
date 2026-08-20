# ULTRAQA — Analyze-91

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Compatibility evidence payload validates; missing `reason` fails; prior model fixtures still pass; no rating fields.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | schema | load `$id` | present | `aira-schema` list_ids | PASS |
| U2 | valid | fixture | pass | `validate_fixtures` | PASS |
| U3 | invalid | missing reason | fail | invalid manifest entry | PASS |
| U4 | enum | ArtifactType | unchanged | no aira-artifact diff | PASS |
| U5 | cli | schema validate --fixtures | failed=0 (34 passed) | aira-cli | PASS |
| U6 | regression | prior model fixtures | still pass | U2 | PASS |
| U7 | out | rating_score field | absent | schema properties | PASS |
