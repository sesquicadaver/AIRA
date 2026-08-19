# ULTRAQA — Analyze-81

**Verdict:** PASS (local workspace)  
**Date:** 2026-08-19

## Goal
CLI behavior unchanged after file split.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | clap | secret-hex XOR file | parse err, no seed leak | clap_secret_hex_file | PASS |
| U2 | tenant | seed parse/load/persist | same as pre-split | tenant_secret tests | PASS |
| U3 | clippy | aira-cli -D warnings | green | cargo clippy -p aira-cli | PASS |
| U4 | help | `aira --help` / `identity --help` | same command names | smoke | PASS |
