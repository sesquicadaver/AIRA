# Repair Package 0 — Product contract & documentation rules

**Status:** **DONE** (2026-09-10) — contract documentation closed; UI/runtime delivery remains Pack 1+.  
**Tip at start of work:** `e7ed7d6` (RFC-0159 amd. already on `main`).  
**Plan source (out of git):** `p-only/aira-repair.md` Pack 0; audit `p-only/aira-current.md`.  
**RFC-D:** [AIRA-RFC-0207](../specs/rfc/AIRA-RFC-0207-repair-package-0-product-contract.md)  
**Trace:** [`product-requirements-trace.md`](product-requirements-trace.md)  
**Analyze:** [`analysis/Analyze-359/`](../analysis/Analyze-359/)

## Intent

Keep microkernel + VERIFIED. Restore the **product obligation** to expose user control (models, resource permissions, ratings view, profiles) **outside Core**. Do not treat MVP / Developer Preview limits as permanent product bans.

## Delivered in this package

| Item | Change |
|------|--------|
| Book 0 working copy | §1.3 product control plane; §3.2 clarified «out of Core ≠ out of product» |
| Canonical terminology | Explicit Kernel-exit vs project-exit; product capabilities allowed |
| Specification Control | §17 product-requirements tracing (non-substitutable by schema/CLI DONE) |
| desktop-ux | End-user = installation owner/steward; operator-out-of-scope **superseded** |
| implementation-status | Product honesty banner + link to trace matrix |
| Historical supersession | Meditation `48` «не вибирає LLM» → replaced by «not obligated to choose» (history preserved) |
| Living smoke | `phase_repair_pkg0_doc.rs` |

## Explicitly not delivered here

Pack 1 admission snapshot — Phase V **DONE** ([`phase-v-plan.md`](phase-v-plan.md) / [`repair-package-1.md`](repair-package-1.md); `#323`–`#330` DONE @ RFC-0208; QUEUE V closed; residual Phase W `#331`–`#342` IN PROGRESS). Pack 2 multi-model GUI — Phase X [`phase-x-plan.md`](phase-x-plan.md) `#343`–`#358` **QUEUED** (RFC-0226 reserved; після W). Pack 3–7 still need their own phase plans.

## Acceptance (Pack 0)

```text
No required product capability is marked product-DONE only because a schema, CLI helper, or local mock exists.
Core anti-mission unchanged (no GPU marketplace; no LLM-in-Core).
User may automate selection but retains the right to constrain it.
```
