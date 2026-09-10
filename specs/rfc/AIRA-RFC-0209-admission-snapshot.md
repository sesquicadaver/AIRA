# AIRA-RFC-0209 — Admission snapshot type + persist-on-admit (RFC-D)

## 1. Summary

QUEUE `#324` (Phase V / Repair Pack 1): introduce immutable `AdmissionSnapshot` captured at ProblemSubmitted admit time, published as an OperationalArtifact, mirrored into Context `resolved_factors`, and persisted on `ProblemRecord`. Text-only submit uses `default_for_text` without changing HTTP/CLI request bodies (`#325`).

## 2. Problem Statement

User constraints (model set, content identity, generation params, placement, privacy/budget, fallback, reuse policy) were not frozen at admit. Settings mid-run and text-only reuse could not be constrained by a durable snapshot.

## 3. Motivation

`aira-repair.md` Pack 1; [`docs/phase-v-plan.md`](../../docs/phase-v-plan.md); consolidating RFC-0208 reserved until `#330`.

## 4. Scope

- `aira-flow::AdmissionSnapshot` (+ helpers)
- Publish on `OperationalPlane::submit_problem` / normative-split admit
- `ProblemSubmitted.artifact_refs` includes admission id; `payload_ref` remains problem text
- Context-basic copies snapshot into `resolved_factors.admission_snapshot`
- `ProblemRecord.admission_snapshot` / `admission_artifact_id` (including failed persist)
- Unit + flow + context tests

## 5. Non-Goals

```text
Submit API beyond text (#325)
Reuse key rewrite (#326)
activate_verified hash continuity (#327)
Capsule↔Output↔Result binding (#328)
Pack 2 multi-model GUI
GPU marketplace / LLM-in-Core
```

## 6. Compatibility

Existing `{ "text" }` HTTP/CLI/Desktop submit remains valid. Old problem index rows without admission fields deserialize via `#[serde(default)]`.

## 7. Security / Integrity

Snapshot is immutable after publish (CAS artifact). Default fallback flags are false (no silent model/placement substitute). Reuse policy semantics enforced in `#326`.

## 8. Acceptance

```text
Submit "Calculate 2 + 2" → admission OperationalArtifact on ProblemSubmitted.
ContextResolved.resolved_factors.admission_snapshot.kind == admission_snapshot.
LocalSession problem row after reopen carries admission_snapshot + artifact id.
C1 2+2 still Completed / VERIFIED.
```

## 9. References

- QUEUE `#324` · Analyze-361
- Phase V plan · Pack 1
- Parent consolidating: RFC-0208 (file-free until `#330`)
