# AIRA-RFC-0228 — Model select API Auto/required (RFC-D)

## 1. Summary

QUEUE `#345` (Phase X / Pack 2 M2): typed [`select_model`] resolves **Auto** or **Required** against per-model lifecycle (#344). Unready or removed required models fail closed with an explained error — never a silent substitute to another available model.

## 2. Problem Statement

After `#344`, slots make A/B independently available, but callers still lacked a select API: “Auto” was only “omit `model_ref` and hope `activated.latest`”, and required unready/removed collapsed into opaque activate-gate strings without a durable resolve contract.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) M2; PR-M1 local select/Auto; parent RFC-0226 reserved until `#358`.

## 4. Scope

- `ModelSelection::{ Auto, Required(String) }` + `select_model(aira_root, selection)`
- Auto: prefer available `activated.latest` tip; else first available by `model_ref`; empty → `NoAvailableModels`
- Required available → `SelectedModel`; verified-only → `ModelUnready`; absent → `ModelRemoved`
- No silent substitute when Required names a missing model while another is available

## 5. Non-Goals

```text
Request profile → snapshot (#346)
CLI supported-contract parity (#347)
Settings Models catalog GUI (#348)
Work executor / Compare
Weakening activate evidence/hash admit
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

Admission still accepts optional `model_ref`. Callers may place `SelectedModel.model_ref` into constraints. Gate (#344) unchanged. Excludes / auto-within-set remain `#346` / later.

## 7. Acceptance

```text
Auto + no available → NoAvailableModels (explained)
Auto + tip available among A/B → tip model_ref
Required available A → A
Required verified-only → ModelUnready (explained)
Required unknown while B available → ModelRemoved (not B)
Tip → first OPEN #346
```

## 8. References

- QUEUE `#345` · Analyze-382
- Parent: RFC-0226; prior RFC-0227 lifecycle; RFC-0109 activate gate; RFC-0159 model triple
