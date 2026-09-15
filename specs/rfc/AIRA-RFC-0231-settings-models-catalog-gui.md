# AIRA-RFC-0231 — Settings Models catalog GUI (RFC-D)

## 1. Summary

QUEUE `#348` (Phase X / Pack 2 X2): Settings → Models is a **local catalog** with list / scan / add / select / prepare and per-row ready reasons — without requiring CLI for the operator path.

## 2. Problem Statement

RFC-0159 amd. and Help promised select/activate under Settings → Models, but the UI was observe-only triple text. Lifecycle/select/activate APIs (#344–#346) existed in CSU while Desktop still told operators to use CLI.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) M4→X2 Settings catalog; parent RFC-0226 reserved until `#358`.

## 4. Scope

- Runtime facade `model_catalog` over `list_model_lifecycle`, `scan_and_publish`, `fetch_to_quarantine`, `select_model`, `activate_verified_model`
- Desktop Settings Models: catalog rows + Scan / Add / Select / Prepare / Enable local add
- Per-row `ready_reason`; tip ≠ sole authority
- Choice of model still ≠ VERIFIED

## 5. Non-Goals

```text
Work executor Auto/specific / pre-submit readiness (#349)
Compare mode (#354)
Remote download / marketplace
Weakening activate evidence/hash admit
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

CLI `aira models` remains valid. Triple selected ≠ ready ≠ used unchanged. Add still respects acquisition policy (default DENY until Enable local add). Quarantine without verify evidence cannot Prepare to available — explained in message.

## 7. Acceptance

```text
empty root → empty catalog + ready empty hint
fixture / verified slots → rows with ready_reason
Scan → inventory publish message
Enable local add → policy auto_download
Select Auto empty → explained fail-closed
Prepare missing → explained
Tip → first OPEN #349
```

## 8. References

- QUEUE `#348` · Analyze-385
- Parent: RFC-0226; prior RFC-0230 CLI parity; RFC-0227…0229; RFC-0159 model triple
