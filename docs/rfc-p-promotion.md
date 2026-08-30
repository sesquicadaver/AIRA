# RFC-P — Research Promotion (Reference working copy)

**QUEUE:** `#177` (Analyze-212) process; `#178`–`#180` **DONE**. **Статус:** status rollup — `#181`.  
**Канон виконання:** [`QUEUE.md`](../QUEUE.md). **План:** [`phase-h-plan.md`](phase-h-plan.md) H5.  
**Нормативні джерела:** Book V §3 / §7 / §8 / §28 / §30; [`specs/rfc-process.md`](../specs/rfc-process.md) §12–§13 / §31 (RFC-R / RFC-P / Promotion Gate).  
**Контрактний тест:** `crates/aira-desktop-runtime/tests/phase_h_doc.rs`.

## 1. Invariant

```text
Book V → operational лише через promotion.
Research = Evidence + Hypothesis + Experiment + Validation + Artifact Promotion
         - Direct Operational Authority
```

Research Track (Book V) is **non-normative**. Compatibility is Book 0–III. Book IV is reference. A Research Artifact, Open Research Artifact, or Artifact Promotion Candidate **MUST NOT** be an Operational Plane input, an Operational Artifact, or a Verified Result Artifact until an accepted RFC-P has promoted it.

Operational Plane **does not wait** for Research Plane. Research may improve future Resolution; it must not block current Resolution.

## 2. Purpose

This file is the **reference working copy** of RFC-P for this tree. It does not promote any research item. It records how promotion is admitted so later atoms (`#178`–`#181`) can encode the gate without inventing policy.

RFC-P class (`rfc-process.md` §13) moves a validated Research Artifact into one of:

```text
Reference | Optional | Optimization | Protocol extension
CSU contract | Schema extension | Conformance test | Rejected
```

A numbered instance (`AIRA-RFC-P-NNNN`) is a *use* of this process, not a substitute for it. Phase H does not mint RFC-P-0001 in this atom.

## 3. Non-Goals (`#177`)

```text
promotion-candidate schema fixtures          (#178)
runtime non-operational gate + test          (#179 DONE)
run_c5 scaffold                              (#180 DONE)
implementation-status C5 / promotion rollup  (#181)
Reference v0.3 / RFC-0077                    (#182–#183)
PHM / HIE / Goal Compiler / DSM as runtime
GPU / LLM / blockchain / scheduler (anti-mission)
silent mix of Research into Core / OperationalPlane
changing Book 0–III without a later RFC class
```

## 4. Boundary (Book V §3)

Research **MUST NOT**:

```text
change Core
bypass Stable ABI
bypass Policy Gate
mutate Artifact in-place
create Operational Artifact directly
perform hidden normative collapse
require Book 0–III change without RFC
affect Operational Plane without Artifact Promotion
```

Research **MAY** produce:

```text
Evidence Artifact
Hypothesis Artifact
Research Artifact
Best Current Hypothesis Artifact
Negative Result Artifact
Open Research Artifact
Artifact Promotion Candidate
```

Those types stay **non-operational until promote**.

## 5. Lifecycle → Promotion Pipeline

Book V §7:

```text
Proposal → Hypothesis → Experiment Design → Execution
  → Evidence Collection → Validation → Replication
  → Promotion Candidate → Reference RFC or Archive
```

Book V §8 (only path Research → Operational):

```text
Research Artifact
  → Evidence Review
  → Epistemic Validation
  → Compatibility Check
  → Safety Check
  → Cost/Benefit Evidence
  → Canary Deployment
  → Reference RFC (RFC-P)
  → Optional / Reference / Rejected
```

Fail-closed for this reference: a research or promotion-candidate artifact presented as operational input is **rejected** (`OperationalPlane` inject/drain, RFC-0082 / `#179`). CAS may still store research artifacts. `run_c5` local scaffold is `#180` (RFC-0083).

Canary Deployment is a Book V §8 pipeline step. Phase H `#180` is a **local** `run_c5` scaffold, not a canary harness and not a distributed deploy.

## 6. RFC-P required evidence

RFC-P **MUST** contain the union of `rfc-process.md` §13.2 and Book V §28 (every field present; empty/N/A only with explicit rationale — do not drop the field):

```text
research artifact refs
evidence refs
counter-evidence refs
replication evidence
measured benefit
risk analysis
security analysis
privacy analysis
compatibility analysis
rollback path
conformance tests
promotion scope
problem statement
failure model
impact on Book 0–III
```

`rfc-process.md` §31 asks «Replicated if needed?» at the **gate**; §13.2 still requires the `replication evidence` field on every RFC-P (absence of replication is recorded, not omitted).

Missing any of the above → **REQUIRE_MORE_EVIDENCE** or **REJECT**. Promotion is not a docs-only rename of a research file into `schemas/` or `crates/`.

## 7. Promotion Gate (must all hold)

Research may become Reference only if (`rfc-process.md` §13.3 / §31; Book V §8.1):

```text
Validated?
Replicated if needed?
Measured benefit (measurable total-cost reduction)?
No Direct Operational Authority before accept?
No hidden normative collapse?
Compatible with Book 0–III?
No Core change (else RFC-A / RFC-B, not RFC-P alone)?
No hidden dependency?
Rollback exists?
Conformance tests exist?
Failure semantics exist?
```

If the idea still needs a new fundamental entity, it stays Research (or becomes RFC-A after Architecture Change Gate). If it can be a CSU / Protocol / Artifact / Schema / Book IV choice, Core does not change (`rfc-process.md` §27).

## 8. Book IV experimental toggles (Book V §30)

Any Book V feature present in this reference **MUST** be:

```text
disabled by default
marked experimental
isolated from Operational Plane
excluded from Reference compatibility (C0–C4)
covered by safety policy
excluded from core conformance tests
```

C5 (`run_c5`, `#180`, RFC-0083) is the local scaffold for **research separation + promotion gate**, not a merge gate and not a licence to run research as operational.

## 9. Research-only items (not promotion candidates here)

Book V §29 remains research-only unless a future RFC-P (after `#183`) proves otherwise. This includes:

```text
AIRA-RES-DSM   Distributed Scientific Method
AIRA-RES-SOEC  Self-Organizing Execution Constellations
AIRA-RES-CAP   Capability Algebra
AIRA-RES-PHM   Advanced PHM
AIRA-RES-KF    Knowledge / Cognitive Field
AIRA-RES-GBG   Glass Bead Game Mode
AIRA-RES-GC    Goal Compiler
AIRA-RES-UIT   Universal Intent Theory
AIRA-RES-CW    Collective Wisdom
AIRA-RES-HIE   Hypergraph / Higher-Order Intelligence Engines
AIRA-RES-EXP   Long-running Background Experiments
AIRA-RES-HFC   Human Final Collapse UX / Governance Research
```

Anti-mission (README §«What AIRA is not») is **never** a promotion target:

```text
GPU marketplace · LLM runtime · blockchain network · scheduler
Core що «вирішує задачі» замість CSU contracts
```

## 10. Mapping to Phase H H5

| QUEUE | Atom | Role vs this doc |
|-------|------|------------------|
| `#177` | this file | policy: Book V → operational лише через promotion |
| `#178` | promotion-candidate fixtures | schema + invalid-without-required-fields (**DONE**) |
| `#179` | non-operational gate | research/promotion artifact **rejected as operational** (**DONE**, RFC-0082) |
| `#180` | `run_c5` | research separation + promotion gate local cases (**DONE**, RFC-0083) |
| `#181` | status rollup | `implementation-status` C5 / promotion notes |

Per-atom numbered RFC for the **gate** is `#179`/`#180` ([`phase-h-plan.md`](phase-h-plan.md) §6). Consolidating Phase H RFC is RFC-0077 (`#183`).

## 11. Pointers

| Surface | Path |
|---------|------|
| Book V | `Manifesto etc/Book V — Research Track v0.1.md` |
| RFC classes | [`specs/rfc-process.md`](../specs/rfc-process.md) |
| OperationalPlane (C1 demo, not research) | [`operational-plane.md`](operational-plane.md) |
| Conformance C5 scaffold | [`conformance.md`](conformance.md) |
| Status (Research rows stay **RESEARCH** until `#181`) | [`implementation-status.md`](implementation-status.md) |
