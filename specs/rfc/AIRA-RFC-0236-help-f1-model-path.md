# AIRA-RFC-0236 — Help F1 model path (RFC-D)

## 1. Summary

QUEUE `#353` (Phase X / Pack 2 X2): Offline UK/EN Help documents the **GUI-first** local model path (Settings → Models Scan/Add/Select/Prepare → Work Auto/specific) without ordering unavailable actions (Compare, marketplace, System catalog edit, CLI-as-required).

## 2. Problem Statement

Older Help still spoke in “activate / register in the model layer” terms and listed CLI as if required, while Desktop now has Settings Models catalog actions. F1 must match **available** buttons and keep Settings ≠ System honest.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2 after human copy (#352). Parent RFC-0226 reserved until `#358`.

## 4. Scope

- Rewrite `docs/help/{en,uk}/model.select.md` and `model.unavailable.md`
- GUI path: Scan / Enable local add / Add / Select / Prepare / Work Auto|Specific / Run
- Honesty: selected ≠ ready ≠ used; choice ≠ VERIFIED; mock banner → Settings
- Settings edit ≠ System observe; offline Help note
- Explicit non-orders: Compare (#354), marketplace default, System catalog edit, CLI-only path

## 5. Non-Goals

```text
Work Compare mode (#354)
Model data paths UI (#355)
Changing HelpId catalog or embedding machinery
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

Help stays embedded offline (`include_str!`). CLI `aira models …` may be mentioned as optional tooling only.

## 7. Acceptance

```text
EN/UK model.select + model.unavailable pass link checker (≥500 chars; Related parity)
GUI-first path; no Compare / marketplace / System-edit orders
Tip → first OPEN #354
```

## 8. References

- QUEUE `#353` · Analyze-390
- Parent: RFC-0226; RFC-0231 Settings Models; RFC-0232 Work readiness; RFC-0233 mock; RFC-0235 Settings≠System
