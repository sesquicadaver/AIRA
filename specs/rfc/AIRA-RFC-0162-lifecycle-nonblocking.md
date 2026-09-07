# AIRA-RFC-0162 — Lifecycle non-blocking (RFC-D)

## 1. Summary

Phase P `#272`: Desktop Start/Stop/Quit run off the egui `update()` thread via `AsyncDesktopJobs` lifecycle workers. Spawning a lifecycle job bumps `refresh_generation` so a stale status refresh cannot overwrite post-Start/Stop UI. Quit queues Stop then closes. RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

Submit/refresh were already async (`#257`), but Start/Stop/Quit still called synchronous lifecycle (health waits up to tens of seconds) during UI build — blocking F1/navigation. Refresh generation was not invalidated on control ops.

## 3. Motivation

`phase-p-plan` / post-O audit §5: Start/Stop must not block F1; stale refresh must not win.

## 4. Scope

- `LifecycleJobKind` / `LifecycleJobResult`; `try_spawn_lifecycle` / `poll_lifecycle` / `invalidate_refresh`
- UI `request_lifecycle` / `request_quit`; optimistic Starting/Stopping
- Tests; QUEUE → `#273`

## 5. Non-Goals

```text
Help F1 routing (#273)
Consolidating RFC-0156 (#274)
New job framework
```

## 6. Compatibility / Security

No Core changes. Same `start`/`stop` runtime APIs, off-thread only.

## 7. Rollout

QUEUE `#272` → Analyze-307 → PR; next `#273` Help F1 routing.
