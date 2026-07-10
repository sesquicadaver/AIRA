# AIRA Conformance Test Specification v0.1

```text
Type: Normative / Conformance
Scope: AIRA Compatibility Validation
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - Specification Control Document
Exports:
  - Conformance Profiles
  - Test Artifact Model
  - Core Test Suites
  - Protocol Test Suites
  - CSU Test Suites
  - Failure / Evidence Tests
  - Operational / Evolution Separation Tests
  - Compatibility Matrix
```

Цей документ визначає, як перевіряти, що реалізація є сумісною з AIRA. Нормативною основою є Book 0–III: Book 0 задає фундаментальні принципи та онтологію, Book I — Core/ABI, Book II — протоколи, Book III — CSU-контракти.    

---

# 1. Purpose

Conformance Test Specification має перевірити, що реалізація AIRA:

```text
дотримується Book 0 ontology;
реалізує Book I Core / ABI;
реалізує заявлений набір Book II protocols;
реалізує заявлені Book III CSU contracts;
створює валідний Verified Result Artifact;
зберігає Artifact immutability;
зберігає Event causality;
застосовує Policy Gate;
перетворює failures на Evidence;
не змішує Operational Plane та Evolution Plane.
```

Тестування перевіряє **сумісність**, а не якість алгоритмів.

---

# 2. Non-Goals

Conformance Test Specification **не перевіряє**:

```text
оптимальність routing;
якість ML;
ефективність PHM;
продуктивність hardware;
якість евристик;
економічну модель;
досконалість UI;
ефективність конкретного storage backend.
```

Ці аспекти належать до Book IV / Book V або до окремих benchmark suite, але не до базової сумісності. Book IV є референсною, а не нормативною реалізацією; Book V є research/non-normative.  

---

# 3. Normative Keywords

```text
MUST      — обов'язкова вимога;
MUST NOT  — заборона;
SHOULD    — рекомендована вимога;
MAY       — дозволена опція.
```

---

# 4. Conformance Profiles

## AIRA-C0 — Local Core

Перевіряє мінімальне ядро:

```text
Book 0 ontology
Book I Object Model
Handle Model
Event Runtime
Policy Gate
Invariant Checker
Artifact Runtime
Security Boundary
```

## AIRA-C1 — Local Operational Node

Додає мінімальний operational pipeline:

```text
Context CSU
Reduction CSU
Execution CSU
Verification CSU
Evidence CSU
Verified Result Artifact
```

## AIRA-C2 — Protocol-Compatible Node

Додає локальну або мережеву сумісність:

```text
Event Protocol
Artifact Protocol
Identity Protocol
Discovery Protocol
```

## AIRA-C3 — Federated Node

Додає federation-рівень:

```text
Federation Protocol
Capability Advertisement
Capability Routing Protocol
policy-scoped import/export
```

## AIRA-C4 — Settlement-Compatible Node

Додає:

```text
Settlement / Audit Protocol
receipts
contribution records
audit events
```

## AIRA-C5 — Research-Capable Node

Додає:

```text
Evolution CSU
Research CSU
Artifact Promotion Pipeline
Research isolation
```

C5 не є обов’язковим для базової AIRA-сумісності.

---

# 5. Test Artifact Model

Кожен тестовий запуск **MUST** створити Conformance Artifact.

```yaml
conformance_artifact_id: string
implementation_id: string
implementation_version: string
aira_standard_version: string
profile: enum
test_suite_version: string
started_at: timestamp
completed_at: timestamp
result: enum  # passed | failed | partial | invalid
test_results:
  - test_id: string
    status: enum
    evidence_refs: [artifact_or_event_ref]
    failure_refs: [artifact_or_event_ref]
signature: signature
```

Conformance Artifact є immutable.

Повторний запуск створює нову версію.

---

# 6. Test Result Semantics

Кожен тест повертає один статус:

```text
PASS
FAIL
SKIP
UNSUPPORTED
INVALID
```

## PASS

Вимога виконана.

## FAIL

Вимога порушена.

## SKIP

Тест не застосовується до заявленого profile.

## UNSUPPORTED

Реалізація не заявила підтримку відповідної можливості.

## INVALID

Тест не може бути виконаний через некоректну конфігурацію або пошкоджені fixtures.

---

# 7. Book 0 Foundation Tests

## B0-001 — Core Ontology Recognition

Реалізація **MUST** розпізнавати канонічні сутності:

```text
Problem Statement
Context
Evidence
Epistemic Status
Execution Intent
Execution Capsule
Capability
Artifact
Event
Policy
CSU
Verified Result Artifact
```

## B0-002 — Forbidden Core Entities

Реалізація **MUST NOT** вимагати як Core entity:

```text
GPU
LLM
Node
Driver
Scheduler
Blockchain
Wallet
HIE
Goal
Knowledge Graph
Execution Plan
```

## B0-003 — Operational Pipeline Shape

Реалізація **MUST** підтримувати pipeline:

```text
Problem Statement
↓
Interpret
↓
Reduce
↓
Organize / Materialize if needed
↓
Verify
↓
Verified Result Artifact
```

`Differentiate` може бути stubbed у C0–C1, але для normative ambiguity tests повинен бути підтриманий на рівні Artifact contract.

## B0-004 — Human Final Collapse

Якщо тестовий Problem Statement має дві рівноправні нормативні альтернативи, реалізація **MUST NOT** приховано обрати одну.

Вона повинна створити:

```text
Differentiated Solution Field Artifact
```

або еквівалентний artifact з альтернативами.

## B0-005 — Evidence Primacy

Будь-який Claim Artifact **MUST** мати Evidence references або бути явно позначений як Assumption / Hypothesis.

---

# 8. Book I Core / ABI Tests

## B1-001 — Object Immutability

Спроба змінити існуючий Core Object in-place **MUST** бути відхилена.

Очікуваний результат:

```text
InvariantViolation Event
```

## B1-002 — Artifact Immutability

Спроба змінити Artifact payload після publication **MUST** бути відхилена.

Очікуваний результат:

```text
ArtifactInvalid або InvariantViolation Event
```

## B1-003 — Handle Opacity

CSU **MUST NOT** мати можливість отримати внутрішню структуру handle.

Тест:

```text
CSU receives handle
CSU attempts to infer storage path/internal id
Core rejects or exposes no implementation detail
```

## B1-004 — Event Causality

Подія з `causal_refs` **MUST** зберігати причинний зв’язок.

Тест перевіряє:

```text
ProblemSubmitted
↓
ContextResolved
↓
CapsuleCreated
↓
CapsuleCompleted
↓
ResultPublished
```

## B1-005 — No Global Total Ordering Requirement

Реалізація **MUST NOT** вимагати глобального total order для всіх Events.

Локальний causal order достатній.

## B1-006 — Policy Gate Enforcement

Будь-яка контрольована дія без Policy Check **MUST** бути відхилена.

## B1-007 — Policy Result Semantics

Policy Gate **MUST** повертати тільки:

```text
ALLOW
DENY
REQUIRE
```

## B1-008 — Invariant Checker Authority

Invariant Checker **MUST** мати можливість заблокувати операцію.

## B1-009 — Security Boundary

CSU **MUST NOT** мати прямий доступ до:

```text
Core memory
foreign CSU state
private Artifact payloads
secret keys
unscoped user data
```

## B1-010 — Verified Result Artifact Completeness

Result Artifact **MUST** містити:

```text
problem_statement_ref
context_ref
evidence_refs
verification_status
confidence
scope
provenance_refs
artifact_hash
signature
```

---

# 9. Book II Protocol Tests

## B2-001 — Common Envelope Validation

Кожне protocol message **MUST** мати:

```text
protocol_id
protocol_version
message_type
message_id
issuer_identity
payload_hash
created_at
signature
```

## B2-002 — Signature Validation

Повідомлення з недійсним підписом **MUST** бути відхилене.

Очікуваний результат:

```text
INVALID_SIGNATURE
```

## B2-003 — Unsupported Version

Несумісна protocol version **MUST** повертати:

```text
UNSUPPORTED_VERSION
```

без side effects.

## B2-004 — Discovery by Capability

Discovery Protocol **MUST** шукати Capability, а не Node.

Запит типу:

```text
Find(Node)
```

не є canonical Discovery request.

## B2-005 — Capability Advertisement

Capability Advertisement **MUST** містити:

```text
provider_csu
capability_type
scope
policy_refs
signature
```

## B2-006 — CRP Route Candidate

CRP response **MUST** повертати capability chain або route failure.

Він **MUST NOT** вимагати global routing table.

## B2-007 — Artifact Resolve

Artifact Protocol **MUST** перевіряти hash і signature.

## B2-008 — Event Protocol Idempotency

Повторна доставка тієї самої події **MUST NOT** створювати дубльований semantic effect.

## B2-009 — Identity Key Rotation

Identity Protocol **SHOULD** підтримувати key rotation.

Для C2+ це рекомендований тест.

## B2-010 — Federation Autonomy

Federation Protocol **MUST NOT** вимагати глобальної policy uniformity.

## B2-011 — Settlement Privacy

Settlement / Audit Protocol **MUST NOT** розкривати:

```text
raw prompt
private result payload
secret data
```

---

# 10. Book III CSU Contract Tests

## B3-001 — CSU Manifest Validity

CSU manifest **MUST** містити:

```text
csu_id
csu_type
csu_version
abi_version
identity_ref
capabilities
permissions
event_subscriptions
artifact_inputs
artifact_outputs
signature
```

## B3-002 — Manifest Signature

Unsigned CSU manifest **MUST** бути відхилений.

## B3-003 — Capability Declaration

CSU **MUST** декларувати Capability перед використанням.

## B3-004 — No Direct CSU Call

CSU **MUST NOT** напряму викликати інший CSU поза ABI/Event/Artifact mechanism.

## B3-005 — Event Subscription Contract

CSU **MUST** підписуватись на Events через Event Runtime.

## B3-006 — Artifact Input Validation

CSU **MUST** перевіряти input Artifact descriptor, hash, policy і schema version.

## B3-007 — Failure Event

Будь-яка CSU failure **MUST** створити Failure Event.

## B3-008 — Failure Evidence

Failure **SHOULD** створити Failure Evidence Artifact.

Для C1+ це SHOULD; для C5 це MUST.

## B3-009 — Policy Before Action

CSU **MUST** виконати Policy Check перед контрольованою дією.

## B3-010 — Role Contract

CSU, що заявляє роль, **MUST** пройти role-specific tests.

---

# 11. Standard CSU Role Tests

## CTX-001 — Context CSU

Context CSU:

```text
MUST produce Context Artifact;
MUST NOT produce final Result;
MUST preserve unresolved ambiguity.
```

## RED-001 — Reduction CSU

Reduction CSU:

```text
MUST prefer Ready Solution / Knowledge lookup before Execution;
MUST produce Negative Lookup Artifact if lookup fails;
MUST preserve multiple equivalent candidates.
```

## EVD-001 — Evidence CSU

Evidence CSU:

```text
MUST collect observations;
MUST preserve provenance;
MUST NOT assign final truth unless also Epistemic CSU.
```

## EPI-001 — Epistemic CSU

Epistemic CSU:

```text
MUST separate Evidence, Confidence, Scope, Epistemic Status;
MUST support Counter Evidence;
MUST preserve Revision History.
```

## EXE-001 — Execution CSU

Execution CSU:

```text
MUST execute only authorized Execution Capsules;
MUST produce Output Artifact;
MUST emit CapsuleCompleted or CapsuleFailed.
```

## VER-001 — Verification CSU

Verification CSU:

```text
MUST distinguish Output Artifact from Verified Result Artifact;
MUST produce verification_status;
MUST reject unverifiable output.
```

## ART-001 — Artifact CSU

Artifact CSU:

```text
MUST preserve immutability;
MUST support supersession;
MUST preserve provenance.
```

## DIS-001 — Discovery CSU

Discovery CSU:

```text
MUST search by Capability;
MUST NOT require global registry;
MUST respect federation/policy scope.
```

## SET-001 — Settlement CSU

Settlement CSU:

```text
MUST produce verifiable receipt;
MUST NOT expose private payload;
MAY use blockchain but MUST NOT require it.
```

## PHM-001 — PHM CSU

PHM CSU:

```text
MUST produce Health/Risk Evidence;
MUST NOT make routing decisions directly;
MUST distinguish degradation from malicious behavior.
```

## HUM-001 — Human Interaction CSU

Human Interaction CSU:

```text
MUST present alternatives;
MUST NOT silently collapse normative alternatives;
MUST preserve user decision provenance.
```

---

# 12. Operational Pipeline Test Suite

## OP-001 — Minimal Successful Flow

Input:

```text
Problem Statement: "Calculate 2 + 2"
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
```

## OP-002 — Ready Solution Reuse

Якщо Ready Solution Artifact існує, реалізація **MUST** reuse it before Execution.

## OP-003 — Execution Required

Якщо Ready Solution відсутній, система **MAY** створити Execution Capsule.

## OP-004 — Verification Failure

Якщо Output не проходить Verification, система **MUST NOT** створити Verified Result Artifact.

Expected:

```text
VerificationFailed Event
Rejected Output Artifact
Failure Evidence Artifact
```

## OP-005 — Negative Result

Якщо всі доступні методи не досягли Success Criterion, система **MUST** створити:

```text
Negative Result Artifact
```

або

```text
Open Research Artifact
```

залежно від профілю.

---

# 13. Operational / Evolution Separation Tests

## SEP-001 — No Direct Evolution Promotion

Evolution CSU **MUST NOT** створювати Operational Artifact напряму.

Expected path:

```text
Research Artifact
↓
Evidence Review
↓
Epistemic Validation
↓
Promotion Candidate
↓
Operational Use
```

## SEP-002 — Background Failure Isolation

Failure в Research CSU **MUST NOT** зупиняти Operational Plane.

## SEP-003 — Research Quarantine

Research Artifact **MUST** бути позначений як non-operational до promotion.

## SEP-004 — Promotion Evidence

Artifact Promotion **MUST** містити:

```text
evidence_refs
validation_refs
compatibility_check
policy_refs
rollback_path
```

---

# 14. Security Test Suite

## SEC-001 — Unsigned Event Rejection

Unsigned Event **MUST** бути відхилений.

## SEC-002 — Unauthorized Artifact Access

CSU без Policy permission **MUST NOT** отримати private Artifact payload.

## SEC-003 — Policy Bypass Attempt

Спроба обійти Policy Gate **MUST** створити InvariantViolation Event.

## SEC-004 — Secret Leakage

CSU **MUST NOT** emit secret keys or private payloads into Events.

## SEC-005 — Revoked CSU

Revoked CSU **MUST NOT** створювати нові Events або Artifacts.

---

# 15. Privacy Test Suite

## PRIV-001 — No Raw Prompt in Settlement

Settlement receipt **MUST NOT** містити raw prompt.

## PRIV-002 — No Raw Private Payload in Event

Event payload **MUST NOT** містити raw private Artifact content.

## PRIV-003 — Policy-Scoped Export

Cross-federation export **MUST** пройти policy check.

---

# 16. Failure / Evidence Test Suite

## FAIL-001 — Failure Creates Event

Кожна failure condition **MUST** створити Event.

## FAIL-002 — Failure Creates Evidence

Для C1+ реалізації failure **SHOULD** створити Evidence Artifact.

Для C5 — **MUST**.

## FAIL-003 — Failure Is Queryable

Failure Evidence **MUST** бути доступна через Artifact/Event query.

## FAIL-004 — Failure Does Not Mutate Prior Artifacts

Failure **MUST NOT** змінювати попередній Artifact.

---

# 17. Versioning Tests

## VER-001 — Schema Version Required

Object, Event, Artifact, CSU Manifest і Protocol Message **MUST** мати schema/protocol version.

## VER-002 — Backward-Compatible Extension

Backward-compatible extension **MUST NOT** ламати старі conformance tests.

## VER-003 — Breaking Change Detection

Breaking change **MUST** вимагати major version або окремий RFC.

---

# 18. Determinism Requirements

Conformance tests **MUST** бути deterministic.

Допускається:

```text
async delivery;
different internal ordering;
different storage backend;
different algorithm.
```

Не допускається:

```text
non-repeatable test result без declared nondeterminism;
hidden network dependency;
hidden ML dependency;
time-dependent pass/fail без mocked clock.
```

---

# 19. Test Fixtures

Мінімальний набір fixtures:

```text
valid_problem_statement.json
valid_context_artifact.json
valid_execution_capsule.json
valid_output_artifact.json
valid_verified_result_artifact.json
valid_event_chain.json
valid_policy_allow.json
valid_policy_deny.json
valid_csu_manifest.json
invalid_signature_event.json
mutable_artifact_attempt.json
normative_split_problem.json
research_artifact_unpromoted.json
```

---

# 20. Conformance Report

Після тестування реалізація **MUST** створити report:

```yaml
implementation:
  name: string
  version: string
  commit: string | null

aira:
  standard_version: string
  profile: string

results:
  total: number
  passed: number
  failed: number
  skipped: number
  unsupported: number
  invalid: number

failures:
  - test_id: string
    reason: string
    evidence_refs: [artifact_or_event_ref]

signature: signature
```

---

# 21. Compatibility Decision

Реалізація вважається сумісною з profile, якщо:

```text
усі MUST-тести profile мають PASS;
жоден MUST NOT-тест не порушений;
усі failure cases створюють Events;
усі required Artifact descriptors валідні;
усі required signatures валідні.
```

SHOULD-тести можуть бути failed, але повинні бути задокументовані.

---

# 22. Minimal Pass Matrix

```text
C0:
  B0, B1, SEC minimal

C1:
  C0 + OP + core CSU role tests

C2:
  C1 + B2 Event/Artifact/Identity/Discovery

C3:
  C2 + Federation + CRP + Capability Advertisement

C4:
  C3 + Settlement/Audit

C5:
  C4 + Evolution/Research Separation + Promotion Pipeline
```

---

# 23. Canonical Test Command

Рекомендований інтерфейс:

```bash
aira-conformance run --profile C1 --target http://localhost:8080
```

Приклад результату:

```json
{
  "profile": "C1",
  "result": "passed",
  "passed": 84,
  "failed": 0,
  "skipped": 12,
  "unsupported": 4
}
```

---

# 24. Status

AIRA Conformance Test Specification v0.1 формалізує перевірку сумісності реалізацій.

Він закріплює:

```text
profile-based validation;
Book 0 ontology tests;
Book I Core/ABI tests;
Book II protocol tests;
Book III CSU tests;
Verified Result Artifact validation;
failure-to-evidence behavior;
Operational/Evolution separation;
security and privacy baseline;
conformance artifact generation.
```

Наступний нормативний документ:

```text
AIRA Schema Pack v0.1
```

Scope:

```text
JSON Schema / CDDL / Protocol Schemas
Core Object Descriptor
Event Descriptor
Artifact Descriptor
Policy Query
CSU Manifest
Capability Descriptor
Verified Result Artifact
Protocol Envelope
Conformance Report
```
