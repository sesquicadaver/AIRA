# AIRA MVP PRD v0.1

```text
Type: Product Requirements Document
Scope: AIRA MVP
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - Book IV — Reference Implementation
  - AIRA Schema Pack
  - AIRA Conformance Test Specification
  - AIRA MVP Implementation Roadmap
  - AIRA Repository Bootstrap Plan
```

AIRA MVP PRD фіксує **що саме має бути побудовано в першій версії продукту**, без розширення архітектури. MVP не є GPU marketplace, LLM runtime, blockchain network або distributed execution platform. Це локальний доказ головної формули AIRA: `Problem Statement → Progressive Resolution → Verified Result Artifact → Evidence → Lower-cost future Resolution`. Ця мета вже зафіксована в Book 0 і MVP Roadmap.  

---

# 1. Product Goal

## 1.1 Primary Goal

Побудувати мінімальну локальну реалізацію AIRA, яка приймає `Problem Statement`, проходить через контрольований pipeline, створює `Verified Result Artifact`, фіксує Evidence і проходить C0/C1 conformance.

```text
Problem Statement
↓
Context
↓
Reuse / Reduction
↓
Execution only if needed
↓
Verification
↓
Verified Result Artifact
↓
Evidence
```

## 1.2 Product Principle

MVP має довести:

```text
найкраще доступне вирішення задачі
мінімально необхідними ресурсами
через перевірні артефакти
і замінні CSU
без роздування Core
```

Core не вирішує задачі сам; Core забезпечує стабільне середовище: Object Model, ABI, Events, Policy Gate, Invariant Checker, Security Boundary і CSU Runtime. 

---

# 2. Problem Statement

Зараз AIRA має формалізовану архітектуру, але ще не має мінімальної runnable реалізації, яка доводить, що специфікація працює як продукт.

Потрібно перевірити три речі:

```text
1. Чи можна реалізувати AIRA локально без distributed complexity?
2. Чи достатньо Book 0–III для сумісної реалізації?
3. Чи може система створити Verified Result Artifact з Evidence?
```

MVP не повинен доводити масштабованість, federation, routing, PHM, settlement або Research Plane. Це не задача першої версії.

---

# 3. Target Users

## 3.1 Primary Users

```text
AIRA core developer
```

Пише Core Runtime, Object Runtime, Event Runtime, Artifact Runtime, Policy Gate, Invariant Checker.

```text
CSU developer
```

Пише ізольовані модулі, які працюють через ABI, Events, Artifacts і Policy.

```text
Specification maintainer
```

Перевіряє, що реалізація відповідає Book 0–III, Schema Pack і Conformance Specification.

## 3.2 Secondary Users

```text
Technical evaluator
```

Запускає MVP, перевіряє demo flows, читає Events, Artifacts, Conformance Report.

```text
Early integrator
```

Оцінює, чи можна створити власний CSU або адаптер без зміни Core.

---

# 4. MVP Scope

## 4.1 Included

MVP включає лише:

```text
local AIRA node
local Core Runtime
local Object Store
local Artifact Store
local Event Log
Policy Gate
Invariant Checker
CSU Registry
basic CSU set
CLI
schema validation
C0/C1 conformance runner
three demo flows
```

## 4.2 Required CSU

```text
Context CSU
Reduction CSU
Execution CSU
Verification CSU
Evidence CSU
Artifact CSU
```

CSU є єдиною допустимою формою функціонального розширення за межами Core. Вони ізольовані, замінні й працюють через ABI, Events, Artifacts і Policy. 

## 4.3 Out of Scope

MVP не включає:

```text
real federation
multi-node execution
real CRP routing
GPU execution
LLM inference
blockchain settlement
PHM
Research Plane
self-optimization
web UI
cloud deployment
Kubernetes
production auth
```

Book IV прямо визначає Reference Implementation як мінімальну реалізацію, що не потребує ML, GPU, blockchain, federation або distributed execution. 

---

# 5. Product Requirements

# 5.1 Problem Submission

## Requirement

Користувач повинен мати змогу подати текстовий `Problem Statement`.

```bash
aira problem submit --text "Calculate 2 + 2"
```

## Acceptance Criteria

```text
Problem Statement object created.
ProblemSubmitted event emitted.
Object descriptor schema-valid.
Problem ref returned to user.
```

---

# 5.2 Context Creation

## Requirement

Після `ProblemSubmitted` система повинна створити `Context Artifact`.

## Acceptance Criteria

```text
Context CSU observes ProblemSubmitted.
Context Artifact created.
ContextResolved event emitted.
Unresolved ambiguity is preserved if present.
Context CSU does not execute task.
Context CSU does not produce final result.
```

---

# 5.3 Reduction / Reuse

## Requirement

Перед виконанням система повинна перевірити наявність готового рішення або локального знання.

## Acceptance Criteria

```text
Reduction CSU checks Ready Solution Artifact first.
If ready solution exists, Execution CSU is not invoked.
If no ready solution exists, Negative Lookup Artifact is created.
Execution Capsule is created only if needed.
ReductionCompleted event emitted.
```

Це реалізує принцип `Reuse before compute`, зафіксований у Book 0 та MVP Roadmap.  

---

# 5.4 Safe Execution

## Requirement

MVP повинен підтримувати тільки deterministic local execution.

Allowed execution kinds:

```text
math.eval.safe
text.echo
text.uppercase
json.identity
```

## Acceptance Criteria

```text
Execution CSU executes only authorized Execution Capsule.
No arbitrary shell execution.
No network access by default.
No GPU / LLM dependency.
Output Artifact created.
CapsuleCompleted or CapsuleFailed event emitted.
```

---

# 5.5 Verification

## Requirement

Output Artifact не є результатом. Система повинна явно перевірити Output і лише після цього створити `Verified Result Artifact`.

## Acceptance Criteria

```text
Verification CSU observes completed execution.
Output Artifact is validated.
Verified Result Artifact is created only after verification.
VerificationCompleted event emitted.
If invalid, VerificationFailed event emitted.
No fake Verified Result Artifact is created.
```

`Verified Result Artifact` є штатним завершенням Operational Pipeline.  

---

# 5.6 Evidence Creation

## Requirement

Кожен успіх або failure повинен створювати Evidence.

## Acceptance Criteria

```text
ResultPublished produces Evidence Artifact.
CapsuleFailed produces Failure Evidence Artifact.
VerificationFailed produces Failure Evidence Artifact.
Evidence is queryable.
Evidence preserves provenance.
Failure is not hidden.
```

---

# 5.7 Artifact Immutability

## Requirement

Artifacts повинні бути immutable і content-addressed.

## Acceptance Criteria

```text
Artifact has content_hash.
Artifact has provenance_refs.
Artifact has signature.
Artifact payload mutation is rejected.
Supersession creates new artifact instead of modifying old one.
```

Schema Pack формалізує Artifact, Event, Object, CSU Manifest, Verified Result Artifact та Conformance Report як машинно перевірні data contracts. 

---

# 5.8 Event Traceability

## Requirement

Кожна суттєва дія повинна створювати Event.

## Acceptance Criteria

```text
Events are append-only.
Events preserve causal_refs.
Events are queryable by object_ref and artifact_ref.
Duplicate event delivery is idempotent.
No global total ordering is required.
```

Book II вимагає event causality, але не вимагає глобального total ordering. 

---

# 5.9 Policy Gate

## Requirement

Контрольовані дії повинні проходити через Policy Gate.

## Acceptance Criteria

```text
Policy decision is ALLOW, DENY or REQUIRE.
Unknown controlled action defaults to DENY.
PolicyEvaluated event emitted.
Policy bypass attempt creates InvariantViolation.
```

---

# 5.10 Conformance

## Requirement

MVP повинен проходити C0 і C1 conformance.

## Acceptance Criteria

```text
aira conformance run --profile C0 passes.
aira conformance run --profile C1 passes.
Conformance Report Artifact created.
All MUST tests pass.
Failures include evidence_refs.
```

Conformance Specification визначає C0/C1 як перевірку Book 0 ontology, Book I Core/ABI, CSU contracts, Verified Result Artifact, failure-to-evidence behavior і Operational/Evolution separation. 

---

# 6. MVP User Flows

## 6.1 Flow 1 — Calculate 2 + 2

Input:

```text
Calculate 2 + 2
```

Expected flow:

```text
ProblemSubmitted
ContextResolved
ReductionCompleted
CapsuleCreated
CapsuleCompleted
VerificationCompleted
ResultPublished
EvidenceCreated
```

Expected result:

```text
Verified Result Artifact
value = 4
verification_status = VERIFIED
confidence = 1.0
scope = local
```

---

## 6.2 Flow 2 — Ready Solution Reuse

Setup:

```text
Ready Solution Artifact already exists for "Calculate 2 + 2".
```

Expected behavior:

```text
Reduction CSU finds Ready Solution.
Execution CSU is not invoked.
Verification CSU verifies reused artifact.
Verified Result Artifact is published.
Evidence Artifact is created.
```

Purpose:

```text
prove reuse before compute
```

---

## 6.3 Flow 3 — Failure to Evidence

Setup:

```text
Execution Capsule references missing input artifact.
```

Expected behavior:

```text
Artifact resolution fails.
CapsuleFailed event emitted.
Failure Evidence Artifact created.
No Verified Result Artifact created.
```

Purpose:

```text
prove failure is evidence
```

---

## 6.4 Flow 4 — Normative Split Stub

Input:

```text
Problem Statement with two equivalent normative alternatives.
```

Expected behavior:

```text
System does not silently choose.
Differentiated Solution Field Artifact created.
requires_human_collapse = true.
```

This preserves Human Final Collapse from Book 0 without implementing full ethical reasoning in MVP. 

---

# 7. Functional Requirements Summary

```text
FR-001  Submit Problem Statement.
FR-002  Create Context Artifact.
FR-003  Perform Ready Solution lookup.
FR-004  Create Execution Capsule only if needed.
FR-005  Execute deterministic local action.
FR-006  Publish Output Artifact.
FR-007  Verify Output Artifact.
FR-008  Publish Verified Result Artifact.
FR-009  Create Evidence Artifact.
FR-010  Convert failures to Failure Evidence Artifact.
FR-011  Persist append-only Events.
FR-012  Persist immutable Artifacts.
FR-013  Enforce Policy Gate.
FR-014  Register basic CSU.
FR-015  Validate schemas.
FR-016  Run C0/C1 conformance.
FR-017  Provide CLI access to problem/result/artifact/event.
```

---

# 8. Non-Functional Requirements

## 8.1 Determinism

```text
MVP demo flows must be repeatable.
No hidden network dependency.
No hidden ML dependency.
No time-dependent pass/fail without mocked clock.
```

## 8.2 Safety by Default

```text
network disabled for CSU;
shell execution disabled;
unsigned CSU rejected;
unsigned Artifact rejected;
unknown ABI rejected;
private Artifact denied by default;
secret material never emitted into Events.
```

## 8.3 Minimality

```text
No new Core entities.
No Scheduler as required component.
No Node as required ontology.
No Driver as canonical role.
No GPU / LLM / blockchain requirement.
```

This protects the MVP from regression into the early P2P LLM/GPU architecture, which was superseded by the current AIRA model.  

## 8.4 Testability

```text
Every requirement must have automated or fixture-based validation.
Every failure path must be observable.
Every durable output must be an Artifact.
Every conformance run must produce Conformance Report Artifact.
```

---

# 9. Release Criteria

MVP v0.1 is releasable when:

```text
1. Clean checkout builds.

2. `aira init` works.

3. `aira identity create` works.

4. Basic CSU set registers.

5. `aira problem submit --text "Calculate 2 + 2"` produces Verified Result Artifact.

6. Ready Solution reuse demo passes.

7. Failure-to-evidence demo passes.

8. Normative split stub does not auto-collapse.

9. All required descriptors are schema-valid.

10. Events are persisted and queryable.

11. Artifacts are immutable and content-addressed.

12. Policy Gate is enforced.

13. C0 conformance passes.

14. C1 conformance passes.

15. No ML/GPU/blockchain/network dependency is required.
```

---

# 10. Explicit Non-Requirements

The following are not product requirements for MVP:

```text
NREQ-001  No real federation.
NREQ-002  No multi-node routing.
NREQ-003  No production settlement.
NREQ-004  No PHM.
NREQ-005  No Research Plane.
NREQ-006  No LLM inference.
NREQ-007  No GPU scheduling.
NREQ-008  No web UI.
NREQ-009  No cloud deployment.
NREQ-010  No autonomous long-running experiments.
```

These may exist later, but they are not required to prove MVP.

---

# 11. Success Metrics

## 11.1 Product Success

```text
MVP proves Problem Statement → Verified Result Artifact locally.
MVP proves Reuse before Compute.
MVP proves Failure → Evidence.
MVP proves Core can stay minimal.
MVP proves CSU can solve tasks without Core changes.
```

## 11.2 Engineering Success

```text
C0 passes.
C1 passes.
All fixtures validate.
All demo flows pass.
No forbidden ontology regression appears.
No direct CSU-to-CSU mutation.
No artifact mutation.
No hidden scheduler.
```

## 11.3 Architectural Success

```text
The implementation does not add new architecture.
The implementation validates existing architecture.
The implementation reduces uncertainty about feasibility.
```

---

# 12. Product Risks

## R1 — Scope Creep

Risk:

```text
MVP absorbs federation, GPU, LLM, blockchain or optimizer.
```

Mitigation:

```text
Hard scope = local C0/C1 + partial local C2 only.
```

## R2 — Core Pollution

Risk:

```text
Task-solving logic leaks into Core.
```

Mitigation:

```text
Core only coordinates.
All solving behavior lives in CSU.
```

## R3 — Fake Verification

Risk:

```text
Output is treated as Result.
```

Mitigation:

```text
Verification CSU must explicitly publish Verified Result Artifact.
```

## R4 — Hidden Failure

Risk:

```text
Failures are logged but not modeled.
```

Mitigation:

```text
Failure Event + Failure Evidence Artifact required.
```

## R5 — Schema Drift

Risk:

```text
Implementation diverges from Schema Pack.
```

Mitigation:

```text
Schema validation in CI.
Fixtures before feature completion.
```

---

# 13. MVP Product Boundary

```text
MVP =
    local proof of AIRA semantics
  + minimal Core Runtime
  + immutable Artifacts
  + causal Events
  + Policy Gate
  + basic CSU
  + schema validation
  + C0/C1 conformance
```

```text
MVP ≠ AI platform
MVP ≠ distributed network
MVP ≠ GPU marketplace
MVP ≠ LLM runtime
MVP ≠ blockchain system
MVP ≠ research cognition engine
```

---

# 14. Status

AIRA MVP PRD v0.1 фіксує продуктову межу першої реалізації.

MVP має довести лише те, що критично для проєкту зараз:

```text
AIRA може локально,
мінімально,
детерміновано,
через Core + CSU + Artifacts + Events,
перетворити Problem Statement
на Verified Result Artifact
з Evidence
і пройти Conformance.
```

Це достатній продукт першого кроку. Усе понад це — не MVP.
