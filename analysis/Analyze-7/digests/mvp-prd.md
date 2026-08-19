# MVP PRD (аналіз)

**Джерело:** `Manifesto etc/AIRA MVP PRD v0.1.md` (769 рядків)  
**Type:** Product Requirements  
**Confidence:** High

## Evidence

- **§1:** Local proof PS→Context→Reuse→Exec→Verify→VRA→Evidence; C0/C1 conformance. Core не вирішує задачі.
- **§3 Users:** core/CSU/spec maintainers; evaluators/integrators.
- **§4 Scope:** local node + stores + Policy + Invariants + CSU registry + basic CSU (Context/Reduction/Execution/Verification/Evidence/Artifact) + CLI + schema + C0/C1 + 3 demos. Out: federation/CRP/GPU/LLM/chain/PHM/Research/UI/cloud/K8s.
- **§5 FR:** submit/context/reduction/safe exec (math.eval.safe|echo|uppercase|json.identity)/verify/evidence/immutability/events/policy/conformance.
- **§6 Flows:** 2+2; Ready Solution reuse; Failure→Evidence; Normative split stub (no silent collapse).
- **§7–11:** FR-001…017; NFR determinism/safety/minimality/testability; release criteria 1–15; NREQ-001…010; success metrics.
- **§12 Risks:** scope creep, core pollution, fake verification, hidden failure, schema drift.
- **§13 Boundary:** MVP ≠ AI platform / distributed / GPU market / LLM / chain / research engine.

## Inference

Продуктова межа = Book IV R0 spirit + Conformance C0/C1. Готовий acceptance checklist для Issue #80.
