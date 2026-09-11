# Repair Package 1 — Admission integrity & binding

**Status:** **IN PROGRESS** — Phase V [`phase-v-plan.md`](phase-v-plan.md); `#323`–`#326` **DONE**; first OPEN `#327`.  
**Tip at start of work:** `b8c36fd` (Pack 0 / RFC-0207 on `main`).  
**Plan source (out of git):** `p-only/aira-repair.md` Pack 1.  
**Consolidating RFC:** **AIRA-RFC-0208** reserved file-free until `#330`.  
**Trace:** [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M3 / PR-H1 binding rows).  
**Analyze:** [`Analyze-360`](../analysis/Analyze-360/) (wiring), [`Analyze-361`](../analysis/Analyze-361/) (`#324` / RFC-0209), [`Analyze-362`](../analysis/Analyze-362/) (`#325` / RFC-0210), [`Analyze-363`](../analysis/Analyze-363/) (`#326` / RFC-0211)

## Intent

Keep microkernel + VERIFIED. Make user constraints an **immutable admission snapshot** so choice cannot be lost between frontend, reuse, and executor. Fix verify→activate hash continuity and Problem→Capsule→Output→Result binding prerequisites for Pack 2 GUI.

## Delivered so far

| Item | Change |
|------|--------|
| Phase V plan | [`phase-v-plan.md`](phase-v-plan.md) IN PROGRESS |
| QUEUE | `#323`–`#326` DONE; `#327`–`#330` OPEN |
| Living smoke | `phase_v_doc.rs` |
| RFC-0208 | reserved file-free |
| `#324` / RFC-0209 | `AdmissionSnapshot` + publish on admit + Context + ProblemRecord |
| `#325` / RFC-0210 | HTTP/CLI/Desktop carry `AdmissionConstraints`; mid-run Settings ≠ mutate admit |
| `#326` / RFC-0211 | Reuse key ⊇ admission snapshot; RequireNewExecution; legacy text-only miss |

## Explicitly not delivered here (later V atoms / Pack 2+)

`activate_verified` hash check (`#327`); capsule/result binding; multi-model GUI.

## Acceptance (Pack 1 / after `#330`)

```text
Constraints cannot be bypassed via another frontend, unknown field, fallback, reuse, or mutable pointer.
Settings change mid-run does not mutate the admitted task.
Verify tamper before activate is rejected.
QUEUE V closed; no OPEN V atoms.
```
