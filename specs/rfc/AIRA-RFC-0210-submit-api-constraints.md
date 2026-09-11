# AIRA-RFC-0210 — Submit API carries admission constraints (RFC-D)

## 1. Summary

QUEUE `#325` (Phase V / Repair Pack 1): HTTP, CLI, and Desktop submit surfaces may carry [`AdmissionConstraints`] that are frozen into an immutable [`AdmissionSnapshot`] at admit time. Text-only bodies remain valid (`default_for_text`). Live Settings after submit must not rewrite the admitted snapshot.

## 2. Problem Statement

`#324` persisted defaults for text-only submit, but frontends could not express model/placement/generation/reuse constraints at request time. Without a request-bound snapshot, a later Settings change could be mistaken for mutating an in-flight task.

## 3. Motivation

`aira-repair.md` Pack 1; [`docs/phase-v-plan.md`](../../docs/phase-v-plan.md); parent consolidating RFC-0208 reserved until `#330`.

## 4. Scope

- `AdmissionConstraints` + `AdmissionSnapshot::from_text_and_constraints`
- `OperationalPlane` / `LocalSession::submit_problem_with_admission`
- HTTP `POST /v1/problems` optional `admission` object
- CLI optional submit flags (`--model-ref`, `--reuse-policy`, …)
- Desktop `submit_problem_http` includes `admission` JSON at submit time
- Tests: constraints round-trip; post-submit mutation of a local constraints copy does not change `last_admission`

## 5. Non-Goals

```text
Reuse key rewrite (#326)
activate_verified hash continuity (#327)
Capsule↔Output↔Result binding (#328)
Pack 2 multi-model GUI
GPU marketplace / LLM-in-Core
```

## 6. Compatibility

`{ "text": "..." }` alone still admits with `#324` defaults. Old clients unchanged.

## 7. Security / Integrity

`statement_content_hash` is always recomputed from request text. Snapshot is CAS-immutable after publish. Mid-run Settings are not re-read into an already-admitted task.

## 8. Acceptance

```text
HTTP/CLI/Desktop can admit with model_ref + generation knobs.
Admitted last_admission survives post-submit mutation of a constraints copy.
Text-only submit and C1 Calculate 2 + 2 still Completed.
Reuse-index key unchanged (still text SHA) until #326.
```

## 9. References

- QUEUE `#325` · Analyze-362
- Depends on RFC-0209 (`#324`)
- Parent consolidating: RFC-0208 (file-free until `#330`)
- Superseded for reuse keys by RFC-0211 (`#326`)
