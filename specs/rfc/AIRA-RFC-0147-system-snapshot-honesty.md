# AIRA-RFC-0147 — SystemSnapshot honesty (RFC-D)

## 1. Summary

Phase O `#256`: Desktop runtime exposes a typed [`SystemSnapshot`](../../crates/aira-desktop-runtime/src/network_mesh.rs) projection. `MeshTopLevel` no longer maps `UNKNOWN` / `LOCAL_ONLY` to `OFFLINE`. AddressBook size and live session count are separate fields; unknown live sessions are `None` (not `0`). RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

RFC-0136 mapped `LOCAL_ONLY` (and effectively unknown) to banner `OFFLINE`, and Network UI showed AddressBook length as peer connectivity. Phase O UX canon forbids presenting unknown/stale facts as confirmed offline or connected.

## 3. Motivation

End-user honesty: configured book ≠ authenticated sessions; unprobed reachability ≠ offline failure.

## 4. Scope

- `MeshTopLevel::{LocalOnly, Unknown}` distinct from `Offline`
- `NetworkMeshSnapshot::{address_book_count, live_session_count: Option<usize>}`
- `SystemSnapshot` + `DataQuality` + `load_system_snapshot`
- GUI labels for book vs live sessions; `None` → em dash / «немає даних»
- Unit tests; RFC-D this file; QUEUE → `#257`

## 5. Non-Goals

```text
Non-blocking submit/refresh (#257)
Live peer-process session feed into Desktop
Shell IA rewrite (#259)
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

Supersedes RFC-0136 banner mapping for UNKNOWN/LOCAL_ONLY only. No ledger in `aira-core`. No `aira-node → aira-desktop` dependency.

## 7. Rollout

QUEUE `#256` → Analyze-291 → PR; next `#257` non-blocking work/refresh.
