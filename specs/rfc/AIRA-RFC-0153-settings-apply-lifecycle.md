# AIRA-RFC-0153 — Desktop Settings apply lifecycle (Saved ≠ Applied ≠ Restart needed)

## 1. Summary

Phase O `#262`: track disk-saved settings separately from runtime-applied network/listen values; surface **Applied** vs **Restart needed**; group Settings (General / Models / Connection / Advanced); state that closing the window does not stop AIRA. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

Settings auto-persisted without showing whether the running node still used previous network/listen values.

## 3. Motivation

`desktop-ux` §5 requires visible Saved vs Applied and Restart needed when Stop→Start is required.

## 4. Scope

- `settings_apply` module: `AppliedRuntimeSettings`, `SettingsApplyPhase`
- Sync applied on successful Start; profile/listen persist leaves Restart needed
- Immediate apply: language, open_ui_on_start, autostart (no restart badge)
- Settings UI groups + close≠stop copy
- Tests; RFC-D; QUEUE → `#263`

## 5. Non-Goals

```text
Offline F1 shell (#263)
Seed help articles (#264)
Full model inventory editor (see RFC-0159 amd. 2026-09-10: local Settings select/activate is in scope there; GPU marketplace still out)
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

No Core/ledger changes. Autostart sync remains fail-closed via existing ErrorCode.

## 7. Rollout

QUEUE `#262` → Analyze-297 → PR; next `#263` Offline F1 shell.
