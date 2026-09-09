# AIRA-RFC-0191 — Reachability durability honesty (RFC-D)

## 1. Summary

Phase S `#304`: document the real durability boundary for DIRECT apply. `apply_successful_probe` persists challenge-id replay **before** the caller saves `peers/reachability.json`. The pair is **not** a joint atomic commit. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

Operators / docs could read «durable replay» as implying crash-safe atomic apply of replay+state. Reality is two independent `fs::write`s with a burn-window for `challenge_id` if state save never lands.

## 3. Motivation

`phase-s-plan` `#304` / audit §9: crash-safer boundary **або** honest docs that do not claim «атомарно» wider than fact. Peer JSON paths have no shared journal; temp+rename on each file would not close the joint window.

## 4. Scope

- Docstrings on `apply_successful_probe` / `save_reachability_replay`
- `docs/peer-link.md` Phase S `#304` section
- Test: replay on disk before state file; re-apply burns without state save
- QUEUE tip → `#305`

## 5. Non-Goals

```text
Multi-file journal / 2PC for replay+state
Changing admission crypto (#281 / #296)
RFC-0182 consolidating close (#305)
Weakening anti-replay
```

## 6. Compatibility / Security

Anti-replay remains first-write-wins for challenge ids. Honesty does not change on-disk format.

## 7. Rollout

QUEUE `#304` → Analyze-340 → PR; next `#305` RFC-0182 + QUEUE S close.
