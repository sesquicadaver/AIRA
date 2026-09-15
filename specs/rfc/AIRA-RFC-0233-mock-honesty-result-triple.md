# AIRA-RFC-0233 — Mock honesty + result triple (RFC-D)

## 1. Summary

QUEUE `#350` (Phase X / Pack 2 X2): Work shows a **noticeable demo/mock executor banner** with a Settings CTA, and every result surfaces **requested ≠ applied ≠ executed** without masking mock as a verified LLM.

## 2. Problem Statement

Staff default is often reference `MockBackend`. Small System/strip hints (#319) were easy to miss. Work results showed provenance Origin but not the model request path, so operators could confuse tip/selection with what actually ran.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2: Work executor (#349) → mock honesty → network UX. Parent RFC-0226 reserved until `#358`.

## 4. Scope

- Work-screen banner when `executor_is_reference_mock()` + CTA to Settings → Models
- `WorkSubmitModelContext` + `ResultModelTriple` on `WorkResultView`
- `executed = mock` for mock backend (never fills `used_model`)
- requested = admission `model_ref`; applied = selected/tip at submit; executed from payload/provenance
- Help `work.result` documents the triple

## 5. Non-Goals

```text
Network/address honesty (#351)
Compare mode (#354)
Changing staff executor env binding
Weakening activate evidence
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

`used_model` rules (#283/#339) unchanged. selected≠ready≠used System triple (#269) remains separate from result requested/applied/executed.

## 7. Acceptance

```text
mock staff → noticeable Work banner + Settings CTA
mock generate result → executed = mock; used_model = None
requested/applied from submit context visible on result
process generate → executed = payload model_ref when present
math VERIFIED → executed none (no fake model)
Tip → first OPEN #351
```

## 8. References

- QUEUE `#350` · Analyze-387
- Parent: RFC-0226; prior RFC-0232 Work readiness; RFC-0204 executor honesty; RFC-0223 mock ≠ used
