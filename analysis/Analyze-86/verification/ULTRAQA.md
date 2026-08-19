# ULTRAQA — Analyze-86

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Operator/docs/module comments lock C1 reference-local status without changing plane drain semantics.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | drain | `drain_from` body | identical to pre-#51 | git diff: comment only | PASS |
| U2 | flow | existing plane tests | green | `cargo test -p aira-flow --lib` | PASS |
| U3 | C1 | conformance still uses plane | green | `cargo test -p aira-conformance --lib` | PASS |
| U4 | clippy | flow+conformance `-D warnings` | green | cargo clippy -p aira-flow -p aira-conformance | PASS |
| U5 | docs | EVO-2 four-line lock | present | `docs/operational-plane.md` | PASS |
