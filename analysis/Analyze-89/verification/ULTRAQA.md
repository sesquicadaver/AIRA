# ULTRAQA — Analyze-89

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Model profile payload schema validates; missing `model_ref` fails; existing fixtures including ModelArtifact still pass.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | schema | load `$id` | present | `aira-schema` list_ids | PASS |
| U2 | valid | fixture | pass | `validate_fixtures` | PASS |
| U3 | invalid | missing model_ref | fail | invalid manifest entry | PASS |
| U4 | enum | ArtifactType | unchanged | no aira-artifact diff | PASS |
| U5 | cli | schema validate --fixtures | failed=0 (30 passed) | aira-cli | PASS |
| U6 | regression | artifact schema fixtures | still pass | U2 + artifact test | PASS |
