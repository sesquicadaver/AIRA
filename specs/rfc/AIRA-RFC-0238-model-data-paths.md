# AIRA-RFC-0238 — Model data paths (light) (RFC-D)

## 1. Summary

QUEUE `#355` (Phase X / Pack 2 X2): Settings → Models shows the local **storage catalog** (`models/` root + quarantine/verified/cache), **used bytes**, and **volume free space** when observable — otherwise **unknown** (fail-closed). Not download/delete management, marketplace, or System path editing.

## 2. Problem Statement

Pack 2 asked for model data paths (catalog + occupied/free space) in Settings. Desktop had lifecycle catalog actions but no honest disk/path surface.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2 after Compare (#354). Parent RFC-0226 reserved until `#358`.

## 4. Scope

- `ModelStorageSnapshot` + `load_model_storage` / `format_bytes`
- Settings → Models storage block (root, used, free/unknown, observe-only subdirs)
- Tip → first OPEN `#356`

## 5. Non-Goals

```text
Apply-diff honesty (#356)
Two-model M6 (#357)
Download / delete management UI
Marketplace / remote fetch
System path editor (Settings ≠ System)
Pack 3 ratings
```

## 6. Compatibility

Paths remain under Desktop `data_root/models` (inventory scoped root). Symlink escapes are not followed for used-byte accounting.

## 7. Acceptance

```text
Settings shows models_root + used bytes
Available free space is Option — UI unknown when None
RFC-0238 + tip first OPEN #356
```

## 8. References

- QUEUE `#355` · Analyze-392
- Parent: RFC-0226; RFC-0231 Settings Models catalog
