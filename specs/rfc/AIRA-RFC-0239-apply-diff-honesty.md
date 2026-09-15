# AIRA-RFC-0239 — Apply-diff honesty (RFC-D)

## 1. Summary

QUEUE `#356` (Phase X / Pack 2 X2): Settings shows **Changed** (unsaved Connection draft) vs **Saved** vs **Applied** vs **Restart needed**, plus an inline **Apply error** on validate/persist failure. Cancel restores disk. System Connection stays observe+CTA (Settings ≠ System).

## 2. Problem Statement

Saved ≠ Applied ≠ Restart existed (#262/#268), but unsaved draft edits were silent and apply failures only appeared in chrome problems — not as an honest Settings apply-diff.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2 after model data paths (#355). Parent RFC-0226 reserved until `#358`.

## 4. Scope

- `SettingsApplyPhase::Changed` + `settings_connection_draft_dirty`
- Disk-only phase for restart CTA (`settings_apply_phase_disk`)
- Settings badge Save/Cancel draft + inline apply error
- Help `settings.apply` EN/UK
- Tip → first OPEN `#357`

## 5. Non-Goals

```text
Two-model M6 acceptance (#357)
RFC-0226 consolidating close (#358)
System as second edit form
Marketplace / model download-delete
Pack 3 ratings
```

## 6. Compatibility

Existing Applied / RestartNeeded / Undefined semantics unchanged when draft is clean. Connection CTA still keys off RestartNeeded (disk vs runtime), not Changed.

## 7. Acceptance

```text
Draft ≠ disk → Changed; Save/Cancel work
Apply/persist fail → inline Apply error in Settings
Restart CTA ignores unsaved draft
Tip → first OPEN #357; RFC-0239
```

## 8. References

- QUEUE `#356` · Analyze-393
- Parent: RFC-0226; `#262`/`#268` apply phases; RFC-0235 Settings≠System
