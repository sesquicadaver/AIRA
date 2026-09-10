# Product requirements trace (Repair Package 0)

**Status:** Package 0 contract — **IN FORCE** (2026-09-10).  
**Sources:** `p-only/aira-repair.md` §2–§3 Pack 0; `p-only/aira-current.md`; RFC-0159 amd. 2026-09-10.  
**Rule:** A capability is **not** product-DONE merely because a schema, CLI helper, local mock, or QUEUE atom exists. Status below is honest vs the **user-facing scenario**.

## Product principle

> AIRA automates selection **within delegated authority**. The user may leave selection automatic, set preferences, or impose hard constraints. The user is **not required** to pick a model/resource for every task, but **must be able to**.

Choice of executor **never** grants `VERIFIED`. GPU marketplace / Core-as-LLM-runtime remain anti-mission.

## Trace matrix

| ID | Requirement (user capability) | Normative basis | Runtime / API | GUI / F1 | End-to-end test | Product status |
|----|------------------------------|-----------------|---------------|----------|-----------------|----------------|
| PR-M1 | Select / activate a **local** model (or Auto) | Book 0 §1.3 / §3.2; RFC-0159 amd.; this matrix | Phase D activate + inventory | Settings→Models; Work executor control | TBD Pack 2 | **CONTRACT** — UI lag (observe-only superseded) |
| PR-M2 | Exclude models from auto-selection; set default | Book 0 §1.3; EVO-3 | inventory + policy | Settings catalog | TBD Pack 2 | **ABSENT** (product) |
| PR-M3 | See actual executor on the result (selected≠used) | RFC-0159; RFC-0172 | WorkResultView / provenance | Work + System | `phase_p` / desktop tests | **PARTIAL** (observe yes; binding Pack 1–2) |
| PR-M4 | Compare models on a task without silent substitute | aira-repair Pack 2–3 | multi binding | Work «Порівняти» | TBD | **ABSENT** |
| PR-R1 | Multidimensional ratings (user / quality / ops) | EVO-3; D6–D7; aira-repair Pack 3 | model-rating CSU | Settings / System / result | CLI only today | **PARTIAL** (CLI local; not measured quality UI) |
| PR-R2 | Independence of evidence (copies ≠ independent votes) | aira-repair §4; DSM research | TBD Pack 3/6 | honesty copy | TBD | **ABSENT** (operational) |
| PR-S1 | Share **inference** capability without sharing weights | D5; Pack 5 | peer + capability | Settings participation | TBD Pack 4–5 | **PARTIAL** (local offer; remote cycle incomplete) |
| PR-S2 | Share **weights** without offering compute | Pack 5 | artifact transfer path | Settings | TBD | **ABSENT** (publish tied to activated) |
| PR-C1 | Offer own resources with quotas (who/when/limits) | Book 0 composition of capabilities; Pack 4 | capability admit + OS limits | Settings (owner = end-user) | TBD Pack 4 | **ABSENT** (product) |
| PR-C2 | Use trusted peer capability under privacy/budget | Pack 4 | remote execution cycle | Work + Connection | TBD | **ABSENT** (product) |
| PR-P1 | Installation owner = resource steward (not «operator out of scope») | desktop-ux §1; this matrix | same as Desktop | Settings network/resources | docs smoke | **CONTRACT** (persona fixed Pack 0) |
| PR-V1 | VERIFIED independent of popularity / chosen model | Book 0 A0/A5; VRA | verification CSU | result chrome | C1 + flow tests | **DONE** (criterion); must not regress |
| PR-H1 | Failures when Running ≠ «AIRA not started» if cause is model/policy | RFC-0159 §4.2; lexicon | submit errors | Work problem detail | TBD with Pack 1–2 | **ABSENT** (copy gap) |

Status vocabulary matches [`implementation-status.md`](implementation-status.md), plus **CONTRACT** = normative product requirement in force with incomplete delivery.

## Done-when (Package 0)

```text
[x] Core ban ≠ product ban documented (Book 0 + terminology)
[x] End-user owns installation resources (desktop-ux persona)
[x] Model select/ratings/share/resource control listed as product MUST (this matrix)
[x] MVP scope ≠ permanent product ban
[x] Historical «user does not choose LLM» superseded explicitly
[x] No PR-* marked product DONE solely for schema/CLI/mock
```

## Next packages

Pack 1 — admission snapshot + reuse/binding integrity — **IN PROGRESS** as Phase V [`phase-v-plan.md`](phase-v-plan.md) / [`repair-package-1.md`](repair-package-1.md) (`#323`–`#325` DONE @ RFC-0210; first OPEN `#326`; RFC-0208 reserved).  
Pack 2 — local multi-model GUI (first user-complete result).  
See [`repair-package-0.md`](repair-package-0.md).
