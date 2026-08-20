# AIRA-RFC-0007 — `aira models scan|list` (RFC-E)

## 1. Summary

Add CLI commands `aira models scan` and `aira models list` that invoke the Local Model Inventory CSU API. Local-only; no network; no download.

## 2. Problem Statement

Operators need a read-only way to materialize and inspect a local model inventory artifact.

## 3. Motivation

Phase D QUEUE `#58` Done when requires `aira models scan|list`.

## 4. Scope

```text
aira models scan [--dir <path under root/models>]
aira models list
```

## 5. Non-Goals

```text
aira models compatible (#59)
aira models policy / download
federation model market
```

## 6. Behavior

- `scan`: ensure init; call `scan_and_publish`; print artifact_id / content_hash / installed count.
- `list`: load `models/inventory.latest.json` + CAS payload; print installed model refs.
- Default scan directory: `<root>/models` (created if missing).
- `--dir` outside scoped root → error.

## 7. Compatibility Impact

Additive clap surface only.

## 8. Rollback Plan

Remove `Models` command and `commands/models.rs`.

## 9. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) RFC-E; EVO-3 §4 CLI list.
