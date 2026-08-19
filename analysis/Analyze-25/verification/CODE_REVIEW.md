# Code Review — Analyze-25 Minimal Trust CRL

**Date:** 2026-07-16  
**Scope:** Durable trust CRL; revoke CLI; sync + upsert guards

## Recommendation

**APPROVE**

## Architectural status

**CLEAR**

## Findings

None blocking.

## Checklist

- [x] remove vs revoke semantics clear
- [x] local-test protected
- [x] revoked cannot re-enter via upsert
- [x] sync respects CRL (including signing-derived)
- [x] tests cover happy + deny paths
- [x] workspace + clippy
- [x] originals untouched
- [x] anti-stub

## Notes

Rotation ceremony correctly deferred; CRL is the right minimal durable control.
