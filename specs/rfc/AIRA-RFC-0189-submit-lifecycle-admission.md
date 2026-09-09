# AIRA-RFC-0189 — Submit∥Start/Stop admission (RFC-D)

## 1. Summary

Phase S `#302`: Desktop Work submit and Start/Stop are mutually exclusive. No parallel `start()` from submit `ensure_started` while a lifecycle Start is in flight. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

`try_spawn_submit(ensure_started=true)` could call `start()` while `try_spawn_lifecycle(Start)` also ran, risking double start. Lifecycle could also begin during an in-flight submit.

## 3. Motivation

`phase-s-plan` `#302` / post-R audit §7: mutual exclusion or defined queue; no double `start()`.

## 4. Scope

- `admit_submit_lifecycle` + gates in `async_jobs` / `lexicon`
- `try_spawn_submit` rejects while lifecycle inflight
- `try_spawn_lifecycle` rejects while submit inflight
- UI surfaces `LifecycleBusy` / `WorkBusyLifecycle`
- QUEUE tip → `#303`

## 5. Non-Goals

```text
Observe miss off UI-thread (#303)
Reachability durability (#304)
Queued FIFO of deferred submit after Start (reject-now is sufficient)
Changing CTA Stop→Start matrix (#301)
```

## 6. Compatibility / Security

Admission only; refresh already rejected during lifecycle (`#282`).

## 7. Rollout

QUEUE `#302` → Analyze-338 → PR; next `#303` Observe miss off UI-thread.
