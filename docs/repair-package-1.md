# Repair Package 1 — Admission integrity & binding

**Status:** Structural **DONE** @ [AIRA-RFC-0208](../specs/rfc/AIRA-RFC-0208-phase-v-admission-integrity.md) (Phase V `#323`–`#330`; **QUEUE V closed**). Product honesty vs audit `d1115f2` residual D1–D6: **DONE** @ [AIRA-RFC-0215](../specs/rfc/AIRA-RFC-0215-phase-w-pack1-residual-honesty.md) (Phase W `#331`–`#342`; **QUEUE W closed**).  
**Tip at start of work:** `b8c36fd` (Pack 0 / RFC-0207 on `main`).  
**Plan source (out of git):** `p-only/aira-repair.md` Pack 1; residual audit `p-only/AIRA-audit-d1115f2-2026-09-11.md`.  
**Consolidating RFC (structural):** **AIRA-RFC-0208** **DONE** @ `#330`.  
**Residual consolidating RFC:** **AIRA-RFC-0215** **DONE** @ `#342`.  
**Trace:** [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M3 / PR-H1 binding rows).  
**Analyze:** V [`Analyze-360`](../analysis/Analyze-360/) … [`Analyze-367`](../analysis/Analyze-367/); W wiring [`Analyze-368`](../analysis/Analyze-368/) (`#331`); math [`Analyze-369`](../analysis/Analyze-369/) (`#332` / RFC-0216); strict submit [`Analyze-370`](../analysis/Analyze-370/) (`#333` / RFC-0217); constraints [`Analyze-371`](../analysis/Analyze-371/) (`#334` / RFC-0218); boundary [`Analyze-372`](../analysis/Analyze-372/) (`#335` / RFC-0219); evidence [`Analyze-373`](../analysis/Analyze-373/) (`#336` / RFC-0220); trust [`Analyze-374`](../analysis/Analyze-374/) (`#337` / RFC-0221); materialize [`Analyze-375`](../analysis/Analyze-375/) (`#338` / RFC-0222); binding [`Analyze-376`](../analysis/Analyze-376/) (`#339` / RFC-0223); reuse [`Analyze-377`](../analysis/Analyze-377/) (`#340` / RFC-0224); result/verify [`Analyze-378`](../analysis/Analyze-378/) (`#341` / RFC-0225); close [`Analyze-379`](../analysis/Analyze-379/) (`#342` / RFC-0215)

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

## Residual honesty (Phase W / audit D1–D6) — DONE

| Item | Change |
|------|--------|
| Phase W plan | [`phase-w-plan.md`](phase-w-plan.md) **DONE** @ RFC-0215 |
| QUEUE W | `#331`–`#342` DONE; **QUEUE W closed** |
| Living smoke | `phase_w_doc.rs` |
| RFC-0215 | consolidating **DONE** |
| A0 | Pack 1 residual honesty without rewriting V as never-DONE |
| `#332` / RFC-0216 | math capsule fidelity; no default `2+2` |
| `#333` / RFC-0217 | strict submit decoding; unknown fields 4xx |
| `#334` / RFC-0218 | constraints enforce-or-reject matrix |
| `#335` / RFC-0219 | admission boundary text/hash/kind/schema |
| `#336` / RFC-0220 | activate evidence authority (pointer locator-only) |
| `#337` / RFC-0221 | production activation trust (no implicit local-test) |
| `#338` / RFC-0222 | safe weights materialization (no-follow + post-copy + bounded buffer) |
| `#339` / RFC-0223 | backend verified binding (mock ≠ used-model; mismatch reject) |
| `#340` / RFC-0224 | reuse candidate independent check (foreign VRA ≠ Completed) |
| `#341` / RFC-0225 | result/verify task binding (swap reject; generate honesty) |
| `#342` / RFC-0215 | consolidating close |

## Explicitly not delivered (Pack 2+)

Multi-model GUI / Settings catalog / Work «Порівняти» (audit M1–M6) → Phase X [`phase-x-plan.md`](phase-x-plan.md) `#343`–`#358` **IN PROGRESS** (`#343`–`#351` DONE; first OPEN `#352`). Pack 3–7 still separate.

## Acceptance

**Structural (after `#330`):** snapshot/key/hash guards + QUEUE V closed.  
**Honesty (after `#342`):** audit A1–A10 / D1–D6 closed; unknown fields and unsupported constraints fail closed; math capsule fidelity; verified backend binding; reuse/result task binding; **QUEUE W closed** @ RFC-0215.
