# Code Review — Analyze-29 Per-CSU Publisher Identity

**Date:** 2026-07-16  
**Scope:** signature_for; make_*_as; basic CSU publisher emits; with_publisher

## Recommendation

**APPROVE**

## Architectural status

**CLEAR**

## Findings

None blocking.

## Checklist

- [x] Fail closed without signing key
- [x] Plane primary path unchanged (make_event wrappers)
- [x] Default path compatible
- [x] workspace + clippy
- [x] originals untouched
- [x] anti-stub

## Notes

Correct separation of identity_ref vs publisher_identity; lifecycle/runtime publisher deferred.
