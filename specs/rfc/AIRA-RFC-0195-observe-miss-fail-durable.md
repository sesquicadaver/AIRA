# AIRA-RFC-0195 — Observe miss fail durable (RFC-D)

## 1. Summary

Phase T `#309`: light observe records a **versioned fail** (`models/activated.observe-fail.json`) bound to pointer fingerprint + cache len/mtime. UI `observe()` returns that failure without re-scheduling a streaming hash until the version changes. Pending / Ready / Failed are distinct. Admission (`check_activated`) still stream-hashes every generate-local. RFC-0192 stays file-free until `#312`.

## 2. Problem Statement

After `#303`, a cache miss schedules background hash. If the warm path fails (weights mismatch / evidence), the next UI refresh still saw a miss → Pending → another full stream-hash. Damaged weights caused a rehash storm on every status tick.

## 3. Motivation

`phase-t-plan` `#309` / post-S audit §4: Pending/Ready/Failed per version; fail without rehash storm; admit hash policy must not weaken.

## 4. Scope

- `ObserveFailCache` + `activated.observe-fail.json`
- UI observe: fail hit → stored detail (not `OBSERVE_HASH_PENDING`)
- Light warm writes fail on mismatch/evidence error; success clears fail and writes ready
- Admit ignores fail cache (always hashes)
- QUEUE tip → `#310`

## 5. Non-Goals

```text
Quit∥Submit completion (#310)
AddressBook candidate honesty (#311)
Weakening admission hash
Queued FIFO of deferred status after hash
```

## 6. Compatibility / Security

Ready-cache hit semantics from `#277`/`#303` unchanged. Fail is fail-closed for ready. Generate-local still full-hashes weights.

## 7. Rollout

QUEUE `#309` → Analyze-345 → PR; next `#310` Quit∥Submit completion.
