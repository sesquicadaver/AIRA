# AIRA-RFC-0171 — Lifecycle revision races (RFC-D)

## 1. Summary

Phase Q `#282`: each Desktop Start/Stop bumps one `lifecycle_revision` and invalidates refresh; refresh cannot spawn or apply while lifecycle is in flight; Quit during Start queues Stop after Start succeeds (or closes if Start fails); Applied uses `StartOutcome.used_settings` from the worker, not live UI settings. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`#272` moved Start/Stop off-thread but Quit during Start only set a flag (no Stop chain), mid-lifecycle refresh could overwrite post-transition UI, and Applied read live `self.settings` instead of the snapshot `start()` actually loaded.

## 3. Motivation

`phase-q-plan` `#282` / acceptance: one revision per Start/Stop; refresh must not clobber post-transition; Quit-during-Start → Stop; Applied from worker settings snapshot.

## 4. Scope

- `lifecycle_revision` + refresh gate in `AsyncDesktopJobs`
- `quit_followup_after_lifecycle` / Quit→Stop chain in `pump_async_jobs`
- `StartOutcome.used_settings`; Applied from worker snapshot
- QUEUE → `#283`

## 5. Non-Goals

```text
Backend≠model used (#283)
New job framework / Core changes
Consolidating RFC-0164
```

## 6. Compatibility / Security

No Core changes. Same start/stop semantics; Applied honesty improved.

## 7. Rollout

QUEUE `#282` → Analyze-317 → PR; next `#283` Backend≠model used.
