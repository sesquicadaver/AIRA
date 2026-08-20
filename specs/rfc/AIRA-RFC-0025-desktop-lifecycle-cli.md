# AIRA-RFC-0025 — Desktop lifecycle CLI (`aira desktop`)

## 1. Summary

Introduce crate `aira-desktop-runtime` and CLI `aira desktop start|stop|status` for P0 Desktop lifecycle: OS/dev layout, settings, init+identity, per-install Bearer token, supervised `aira-node --http` with PID/lock, attach, and fail-closed port conflicts.

## 2. Problem Statement

Phase E `#76` requires a shared lifecycle contract so tray (`#78`) and CLI do not duplicate process management, and mutating HTTP is not left unauthenticated on loopback.

## 3. Motivation

[`docs/phase-e-plan.md`](../../docs/phase-e-plan.md) §2.1–§2.4; settings schema `#75` / RFC-0024.

## 4. Scope

- `crates/aira-desktop-runtime`
- `aira desktop start|stop|status` (`--data-root`, `--node-bin`)
- Lifecycle tests (idempotent attach, stale PID, port conflict)

## 5. Non-Goals

```text
tray/GUI (#78)
.desktop launcher (#77)
AppImage (#79)
OS autostart hooks (#78)
peer / P1+
desktop_ipc auth mode runtime
in-process HTTP (always spawn aira-node)
```

## 6. Current Behavior

No Desktop supervisor; `aira-node --http` is manual.

## 7. Proposed Change

- Shared library owns layout, settings load/create, bootstrap, token file (0600), spawn/stop, health attach.
- Auth contract for `#76`: **bearer_token** only (`desktop_ipc` reserved → error).
- Port policy: free→start; same instance+root+listen+live pid+health→attach; foreign occupant→fail-closed; no auto-increment.

## 8–18.

Additive. Tests: `cargo test -p aira-desktop-runtime`. Rollback: remove crate + CLI wiring + this RFC.

## 19. Open Questions

Exact XDG paths on macOS/Windows — deferred to E2/E3 (Linux XDG first).
