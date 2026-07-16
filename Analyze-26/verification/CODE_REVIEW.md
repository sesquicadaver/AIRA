# Code Review — Analyze-26 Admin Trust Unrevoke

**Date:** 2026-07-16  
**Scope:** `unrevoke` API + CLI; no silent re-trust

## Recommendation

**APPROVE**

## Architectural status

**CLEAR**

## Findings

None blocking.

## Checklist

- [x] Unrevoke ≠ auto-trust
- [x] NotRevoked on missing CRL entry
- [x] Test covers revoke → block → unrevoke → add → verify
- [x] workspace + clippy
- [x] originals untouched
- [x] anti-stub

## Notes

Correct CRL escape hatch; rotation remains deferred.
