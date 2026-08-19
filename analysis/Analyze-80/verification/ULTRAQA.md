# ULTRAQA — Analyze-80

**Verdict:** PASS (local; filled after script+CI wiring)  
**Date:** 2026-08-19

## Goal
CI-detectable dependency firewall without changing crate graph.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | graph | core→node | error | `--self-test` | PASS |
| U2 | graph | core→flow→CSU | error | `--self-test` | PASS |
| U3 | graph | CSU→CSU | error | `--self-test` | PASS |
| U4 | graph | A↔B cycle | error | `--self-test` | PASS |
| U5 | live | current workspace | clean | `python3 scripts/dep_firewall.py` | PASS |
| U6 | CI | workflow step present | red on fail | `ci.yml` dependency firewall | PASS |
