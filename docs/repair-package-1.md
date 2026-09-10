# Repair Package 1 — Admission integrity & binding

**Status:** **IN PROGRESS** — Phase V [`phase-v-plan.md`](phase-v-plan.md); `#323`–`#324` **DONE**; first OPEN `#325`.  
**Tip at start of work:** `b8c36fd` (Pack 0 / RFC-0207 on `main`).  
**Plan source (out of git):** `p-only/aira-repair.md` Pack 1.  
**Consolidating RFC:** **AIRA-RFC-0208** reserved file-free until `#330`.  
**Trace:** [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M3 / PR-H1 binding rows).  
**Analyze:** [`Analyze-360`](../analysis/Analyze-360/) (wiring), [`Analyze-361`](../analysis/Analyze-361/) (`#324` / RFC-0209)

## Intent

Keep microkernel + VERIFIED. Make user constraints an **immutable admission snapshot** so choice cannot be lost between frontend, reuse, and executor. Fix verify→activate hash continuity and Problem→Capsule→Output→Result binding prerequisites for Pack 2 GUI.

## Delivered so far

| Item | Change |
|------|--------|
| Phase V plan | [`phase-v-plan.md`](phase-v-plan.md) IN PROGRESS |
| QUEUE | `#323`–`#324` DONE; `#325`–`#330` OPEN |
| Living smoke | `phase_v_doc.rs` |
| RFC-0208 | reserved file-free |
| `#324` / RFC-0209 | `AdmissionSnapshot` + publish on admit + Context factor + ProblemRecord persist |

## Explicitly not delivered here (later V atoms / Pack 2+)

Submit API beyond `text` (`#325`); reuse key rewrite; `activate_verified` hash check; capsule/result binding; multi-model GUI.

## Acceptance (Pack 1 / after `#330`)

```text
Constraints cannot be bypassed via another frontend, unknown field, fallback, reuse, or mutable pointer.
Settings change mid-run does not mutate the admitted task.
Verify tamper before activate is rejected.
QUEUE V closed; no OPEN V atoms.
```
