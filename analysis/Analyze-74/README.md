# Analyze-74 — Canonical descriptor hash helper (QUEUE #39)

## Status
CLOSED (QUEUE #39 DONE @ a73d81d / PR #2).
Shared primitive: Canonical JSON → SHA-256 → Ed25519 over `hash.as_str()` bytes. No production call-site switch.

## Done when
Helper tests green; Event/Artifact/Object/CSU verify paths unchanged.

## Out
#40–#43 wiring; leftover LOCAL_TEST fallback sweep (#44).
