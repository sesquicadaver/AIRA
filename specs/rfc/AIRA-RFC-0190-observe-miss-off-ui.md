# AIRA-RFC-0190 — Observe miss off UI-thread (RFC-D)

## 1. Summary

Phase S `#303`: on observe-ready cache miss, Desktop UI observe defers a **streaming** weights hash onto a background thread and returns not-ready with a stable pending detail. Callers never `fs::read` full weight files into memory. Admission (`check_activated`) still stream-hashes on every generate-local. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

After `#277`, observe cache **hits** skip weights sha256, but a **miss** still did `fs::read` + hash on the caller path used by model triple / status refresh — enough to stall UI on large weights.

## 3. Motivation

`phase-s-plan` `#303` / post-R audit §8: cache miss → background/streaming hash; UI must not full-buffer weights; admit hash policy must not weaken.

## 4. Scope

- `ContentHash::sha256_path` — streaming SHA-256
- `ActivatedPointerGate::observe` — UI path: miss → schedule warm + `OBSERVE_HASH_PENDING`
- `ActivatedPointerGate::observe_verify_now` — blocking light observe (tests / warm)
- `check_activated` — always stream-hash (never trusts observe-ready alone)
- `ModelTripleSnapshot::load` stays on `observe()`
- QUEUE tip → `#304`

## 5. Non-Goals

```text
Reachability durability honesty (#304)
Weakening admission hash / trusting observe-ready for generate-local
Merging with dial / CTA / submit atoms
Queued FIFO of deferred status after hash (periodic refresh is enough)
```

## 6. Compatibility / Security

Pending observe is fail-closed for ready. Admission unchanged in strength (still full content verify, now streaming). Observe-ready cache semantics from `#277` retained on hit.

## 7. Rollout

QUEUE `#303` → Analyze-339 → PR; next `#304` Reachability durability honesty.
