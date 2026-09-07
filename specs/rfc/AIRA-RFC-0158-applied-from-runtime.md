# AIRA-RFC-0158 — Applied settings from confirmed runtime (RFC-D)

## 1. Summary

Phase P `#268`: Settings **Applied** values come only from confirmed runtime (successful Start/attach outcome or Running status with pid record). When unconfirmed → **Undefined** (not fake Applied from disk settings). Refresh and submit-start sync applied via status. RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

`AppliedRuntimeSettings::from_settings` at GUI open painted Saved as Applied without a running node. Submit-worker Start did not sync applied; refresh ignored applied.

## 3. Motivation

`phase-p-plan` P1 and post-O audit: Applied reflects confirmed runtime or Explicitly Undefined.

## 4. Scope

- `SettingsApplyPhase::Undefined`
- `applied_runtime: Option<…>`; `from_start_outcome` / `from_status`
- `PidRecordView` peer profile/TTL for confirmation
- Settings UI + i18n for undefined
- Tests; QUEUE → `#269`

## 5. Non-Goals

```text
Model triple (#269)
Reachability endpoint bind (#270)
Lifecycle Start/Stop off update() (#272)
Consolidating RFC-0156 (#274)
```

## 6. Compatibility / Security

No Core/ledger changes. Closing the window still ≠ Stop.

## 7. Rollout

QUEUE `#268` → Analyze-303 → PR; next `#269` Model triple UX.
