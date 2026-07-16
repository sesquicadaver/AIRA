# Code Review — Analyze-28 Dual-Key Grace Window

**Date:** 2026-07-16  
**Scope:** `grace_until`; `to_keyring_at`; CLI `--until`; sync grace keep

## Recommendation

**APPROVE**

## Architectural status

**CLEAR**

## Findings

None blocking.

## Checklist

- [x] Opt-in grace via explicit until
- [x] CRL still blocks upsert during grace
- [x] Immediate cutover when until omitted
- [x] tests cover during/after/invalid
- [x] workspace + clippy
- [x] originals untouched
- [x] anti-stub

## Notes

Uses `time` for RFC3339 parse/format — appropriate for operator `--until`.
