# AIRA-RFC-0148 — Non-blocking Desktop submit/refresh (RFC-D)

## 1. Summary

Phase O `#257`: Work submit and status/mesh refresh run on background threads. egui `update()` only polls channels and schedules repaint. `request_repaint_after` is not a data-refresh API. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

`submit_work()` and Refresh previously blocked the UI thread on HTTP / process status. `request_repaint_after(2s)` only redrew; it did not reload status.

## 3. Motivation

F1/navigation must remain responsive during slow `/v1/problems` or status I/O (Phase O UX canon §8).

## 4. Scope

- `async_jobs::{AsyncDesktopJobs, collect_status_snapshot, run_submit_job}`
- At most one submit and one refresh in flight; refresh generation drops stale results
- Periodic refresh every `STATUS_REFRESH_INTERVAL` (2s) via job spawn, not via repaint alone
- GUI: disabled Submit while in flight; Refresh button queues background job
- Unit tests; RFC-D this file; QUEUE → `#258`

## 5. Non-Goals

```text
Action/error/help IDs (#258)
Shell IA rewrite (#259)
Start/Stop buttons off-thread (may follow later)
Tokio / unbounded job queues
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

No Core/ledger changes. Submit still uses Desktop auth + loopback HTTP. No second policy path.

## 7. Rollout

QUEUE `#257` → Analyze-292 → PR; next `#258` action/error/help IDs.
