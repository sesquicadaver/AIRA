# Repair Package 1 — Admission integrity & binding

**Status:** Structural **DONE** @ [AIRA-RFC-0208](../specs/rfc/AIRA-RFC-0208-phase-v-admission-integrity.md) (Phase V `#323`–`#330`; **QUEUE V closed**). Product honesty vs audit `d1115f2`: **PARTIAL** — residual gaps tracked in Phase W [`phase-w-plan.md`](phase-w-plan.md) `#331`–`#342` (**IN PROGRESS**; first OPEN `#335`; RFC-0215 reserved).  
**Tip at start of work:** `b8c36fd` (Pack 0 / RFC-0207 on `main`).  
**Plan source (out of git):** `p-only/aira-repair.md` Pack 1; residual audit `p-only/AIRA-audit-d1115f2-2026-09-11.md`.  
**Consolidating RFC (structural):** **AIRA-RFC-0208** **DONE** @ `#330`.  
**Residual consolidating RFC:** **AIRA-RFC-0215** reserved file-free until `#342`.  
**Trace:** [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M3 / PR-H1 binding rows).  
**Analyze:** V [`Analyze-360`](../analysis/Analyze-360/) … [`Analyze-367`](../analysis/Analyze-367/); W wiring [`Analyze-368`](../analysis/Analyze-368/) (`#331`); math [`Analyze-369`](../analysis/Analyze-369/) (`#332` / RFC-0216); strict submit [`Analyze-370`](../analysis/Analyze-370/) (`#333` / RFC-0217); constraints [`Analyze-371`](../analysis/Analyze-371/) (`#334` / RFC-0218)

## Intent

Keep microkernel + VERIFIED. Make user constraints an **immutable admission snapshot** so choice cannot be lost between frontend, reuse, and executor. Fix verify→activate hash continuity and Problem→Capsule→Output→Result binding prerequisites for Pack 2 GUI.

## Delivered (Phase V / structural)

| Item | Change |
|------|--------|
| Phase V plan | [`phase-v-plan.md`](phase-v-plan.md) **DONE** @ RFC-0208 |
| QUEUE V | `#323`–`#330` DONE; **QUEUE V closed** |
| Living smoke | `phase_v_doc.rs` |
| RFC-0208 | consolidating **DONE** |
| `#324`–`#329` | snapshot / submit API / reuse / activate hash / executor stamps / §6 e2e |

## Residual honesty (Phase W / audit D1–D6)

| Item | Change |
|------|--------|
| Phase W plan | [`phase-w-plan.md`](phase-w-plan.md) **IN PROGRESS** |
| QUEUE W | `#331`–`#334` DONE; `#335`–`#342` OPEN |
| Living smoke | `phase_w_doc.rs` |
| RFC-0215 | reserved file-free |
| A0 | Pack 1 PARTIAL honesty without rewriting V as never-DONE |
| `#332` / RFC-0216 | math capsule fidelity; no default `2+2` |
| `#333` / RFC-0217 | strict submit decoding; unknown fields 4xx |
| `#334` / RFC-0218 | constraints enforce-or-reject matrix |

## Explicitly not delivered (Pack 2+)

Multi-model GUI / Settings catalog / Work «Порівняти» (audit M1–M6) → Phase X [`phase-x-plan.md`](phase-x-plan.md) `#343`–`#358` **QUEUED** (після W). Pack 3–7 still separate.

## Acceptance

**Structural (after `#330`):** snapshot/key/hash guards + QUEUE V closed.  
**Honesty (after `#342`):** audit A1–A10 / D1–D6 closed; unknown fields and unsupported constraints fail closed; math capsule fidelity; verified backend binding; reuse/result task binding.
