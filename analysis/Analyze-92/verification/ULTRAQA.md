# ULTRAQA — Analyze-92

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Acquisition policy payload validates with `auto_download=false`; missing `auto_download` fails; prior model fixtures still pass.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | schema | load `$id` | present | `aira-schema` list_ids | PASS |
| U2 | valid | auto_download=false | pass | fixture + assert | PASS |
| U3 | invalid | missing auto_download | fail | invalid manifest entry | PASS |
| U4 | enum | ArtifactType | unchanged | no aira-artifact diff | PASS |
| U5 | cli | schema validate --fixtures | failed=0 (36 passed) | aira-cli | PASS |
| U6 | out | downloader / allowlist runtime | absent | not in diff | PASS |
