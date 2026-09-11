# Repair Package 1 — Admission integrity & binding

**Status:** **DONE** @ [AIRA-RFC-0208](../specs/rfc/AIRA-RFC-0208-phase-v-admission-integrity.md) — Phase V [`phase-v-plan.md`](phase-v-plan.md); `#323`–`#330` **DONE**; **QUEUE V closed**; no OPEN V atoms.  
**Tip at start of work:** `b8c36fd` (Pack 0 / RFC-0207 on `main`).  
**Plan source (out of git):** `p-only/aira-repair.md` Pack 1.  
**Consolidating RFC:** **AIRA-RFC-0208** **DONE** @ `#330`.  
**Trace:** [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M3 / PR-H1 binding rows).  
**Analyze:** [`Analyze-360`](../analysis/Analyze-360/) … [`Analyze-366`](../analysis/Analyze-366/); close [`Analyze-367`](../analysis/Analyze-367/) (`#330` / RFC-0208)

## Intent

Keep microkernel + VERIFIED. Make user constraints an **immutable admission snapshot** so choice cannot be lost between frontend, reuse, and executor. Fix verify→activate hash continuity and Problem→Capsule→Output→Result binding prerequisites for Pack 2 GUI.

## Delivered

| Item | Change |
|------|--------|
| Phase V plan | [`phase-v-plan.md`](phase-v-plan.md) **DONE** @ RFC-0208 |
| QUEUE | `#323`–`#330` DONE; **QUEUE V closed** |
| Living smoke | `phase_v_doc.rs` |
| RFC-0208 | consolidating **DONE** |
| `#324` / RFC-0209 | `AdmissionSnapshot` + publish on admit + Context + ProblemRecord |
| `#325` / RFC-0210 | HTTP/CLI/Desktop carry `AdmissionConstraints`; mid-run Settings ≠ mutate admit |
| `#326` / RFC-0211 | Reuse key ⊇ admission snapshot; RequireNewExecution; legacy text-only miss |
| `#327` / RFC-0212 | activate_verified source/post-copy == VerifiedPointer.content_hash |
| `#328` / RFC-0213 | generate-local executor facts + CapsuleCompleted binding |
| `#329` / RFC-0214 | Pack 1 §6 e2e fail-closed (`phase_v_pack1_e2e`) |
| `#330` / RFC-0208 | consolidating close |

## Explicitly not delivered (Pack 2+)

Multi-model GUI / Settings catalog / Work «Порівняти»; Pack 3–7.

## Acceptance (Pack 1 / after `#330`)

```text
Constraints cannot be bypassed via another frontend, unknown field, fallback, reuse, or mutable pointer.
Settings change mid-run does not mutate the admitted task.
Verify tamper before activate is rejected.
QUEUE V closed; no OPEN V atoms.
```
