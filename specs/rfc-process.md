# AIRA RFC Template & Change Process v0.1

```text
Type: Normative / Governance
Scope: AIRA Change Management
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - Book IV — Reference Implementation
  - Book V — Research Track
  - AIRA Specification Control Document
  - AIRA Conformance Test Specification
  - AIRA Schema Pack
Exports:
  - RFC Classes
  - RFC Metadata Model
  - Change Admission Process
  - Architecture Change Criteria
  - Core / ABI Change Criteria
  - Protocol Change Criteria
  - CSU Contract Change Criteria
  - Schema Change Criteria
  - Research Promotion Process
  - Compatibility Impact Model
  - Rollback Requirements
  - Conformance Test Requirements
```

AIRA RFC Template & Change Process v0.1 визначає, як змінювати AIRA після переходу до стандартного корпусу, не руйнуючи концепцію, не роздуваючи Core і не повертаючись до застарілих `GPU / Node / Driver / Scheduler / Blockchain-first` моделей. Поточний стандартний корпус уже зафіксований як Book 0–V, де Book 0–III є нормативними, Book IV — референсним, а Book V — дослідницьким.      

---

# 1. Purpose

RFC-процес існує для того, щоб будь-яка зміна AIRA:

```text
мала чітку проблему;
мала вимірювану користь;
не змінювала Core без необхідності;
не дублювала існуючі сутності;
не ламала сумісність без major-version підстав;
мала rollback path;
мала conformance tests;
була класифікована як Reference, Optional, Optimization або Research.
```

RFC не є місцем для довільної генерації ідей.

RFC — це контрольований механізм еволюції стандарту.

---

# 2. Non-Goals

RFC-процес **не призначений** для:

```text
хаотичного додавання нових шарів;
перейменування канонічних понять без технічної потреби;
перенесення Research напряму в Operational Plane;
введення нових Core entities без доказу неповноти;
обходу Book 0–III;
опису приватних реалізацій як стандарту;
примусового включення ML, GPU, blockchain або federation у Core.
```

Будь-яка така пропозиція автоматично повертається в Research Track або Rejected.

---

# 3. Normative Basis

RFC повинен бути сумісний із такими фундаментальними положеннями:

```text
AIRA starts from Problem Statement.
AIRA produces Verified Result Artifact.
Core remains minimal.
Capabilities are provided by CSU.
Protocols define interoperability, not algorithms.
Research cannot affect Operational Plane directly.
Schemas make compatibility machine-checkable.
Conformance tests define observable compatibility.
```

Book 0 фіксує місію та онтологію, Book I — Core/ABI, Book II — protocol contracts, Book III — CSU contracts, Conformance Specification — перевірку сумісності, Schema Pack — машинно перевірні data contracts.      

---

# 4. RFC Classes

Кожна RFC **MUST** мати рівно один primary class.

```text
RFC-A  Architecture RFC
RFC-B  Core / ABI RFC
RFC-C  Protocol RFC
RFC-D  CSU Contract RFC
RFC-S  Schema RFC
RFC-T  Conformance Test RFC
RFC-E  Reference Implementation RFC
RFC-R  Research RFC
RFC-P  Research Promotion RFC
RFC-X  Deprecation / Supersession RFC
```

---

# 5. RFC-A — Architecture RFC

## 5.1 Scope

RFC-A змінює:

```text
Book 0;
First Principles;
Core Ontology;
Mission;
Canonical Pipeline;
fundamental invariants.
```

## 5.2 Admission Criteria

RFC-A допускається лише якщо доведено хоча б одну умову:

```text
logical contradiction;
incomplete ontology;
unrepresentable task class;
broken invariant;
impossibility of expressing required behavior through CSU / Protocol / Artifact / Algorithm.
```

## 5.3 Default Decision

```text
Default: Reject
```

Архітектура після Freeze стабільна. Нова фундаментальна сутність допускається лише як виняток.

---

# 6. RFC-B — Core / ABI RFC

## 6.1 Scope

RFC-B змінює:

```text
Book I;
Core Object Model;
Stable ABI;
Handle Model;
Event Runtime;
Policy Gate;
Invariant Checker;
Security Boundary;
CSU Runtime contract.
```

## 6.2 Admission Criteria

RFC-B допускається, якщо зміна:

```text
не може бути реалізована як CSU;
не може бути реалізована як Protocol extension;
не може бути реалізована як Artifact schema;
не може бути реалізована як Book IV implementation choice;
не порушує Book 0;
має conformance impact analysis.
```

## 6.3 Versioning

Breaking RFC-B **MUST** збільшувати major ABI version.

---

# 7. RFC-C — Protocol RFC

## 7.1 Scope

RFC-C змінює Book II protocol layer:

```text
Event Protocol
Artifact Protocol
Identity Protocol
Discovery Protocol
Capability Advertisement Protocol
CRP
Federation Protocol
Settlement / Audit Protocol
```

## 7.2 Admission Criteria

RFC-C допускається, якщо зміна:

```text
покращує interoperability;
не вводить algorithm mandate;
не вимагає Core changes;
має protocol versioning plan;
має failure semantics;
має privacy analysis;
має conformance tests.
```

## 7.3 Forbidden Content

RFC-C **MUST NOT** стандартизувати:

```text
routing heuristic;
scheduler;
optimizer;
PHM prediction algorithm;
ML model;
storage backend;
ledger implementation.
```

---

# 8. RFC-D — CSU Contract RFC

## 8.1 Scope

RFC-D змінює або додає CSU role contract.

Приклади:

```text
new CSU role;
new required event output;
new artifact input contract;
new failure semantics;
new capability declaration field;
role-specific conformance tests.
```

## 8.2 Admission Criteria

RFC-D допускається, якщо:

```text
роль має одну відповідальність;
роль не потребує Core changes;
роль взаємодіє лише через ABI / Events / Artifacts / Policy;
роль має manifest requirements;
роль має failure model;
роль має conformance tests;
роль є replaceable.
```

Book III уже формалізує CSU як ізольовані, замінні одиниці, які працюють поверх Core ABI і не змінюють ядро. 

---

# 9. RFC-S — Schema RFC

## 9.1 Scope

RFC-S змінює Schema Pack:

```text
Core Object Descriptor
Artifact Descriptor
Event Descriptor
Protocol Envelope
Policy Query / Decision
Capability Descriptor
CSU Manifest
Execution Capsule
Evidence Artifact
Epistemic Assessment
Verified Result Artifact
Conformance Report
```

## 9.2 Admission Criteria

RFC-S допускається, якщо:

```text
schema change is versioned;
backward compatibility is documented;
hashing/signing impact is defined;
validation rules are updated;
conformance fixtures are updated;
extension namespace is specified;
no forbidden core coupling is introduced.
```

## 9.3 Forbidden Schema Couplings

RFC-S **MUST NOT** вводити обов’язкові поля:

```text
gpu_id
node_id as core identity
driver_id as canonical role
scheduler_id
wallet_address as mandatory identity
blockchain_height as mandatory settlement field
llm_model_id as core dependency
global_state_version
```

Це прямо підтримує ціль Schema Pack — не допустити повернення до застарілих моделей. 

---

# 10. RFC-T — Conformance Test RFC

## 10.1 Scope

RFC-T змінює:

```text
test profiles;
test fixtures;
required PASS criteria;
MUST / SHOULD tests;
failure evidence tests;
security tests;
privacy tests;
schema validation tests;
protocol compatibility tests.
```

## 10.2 Admission Criteria

RFC-T допускається, якщо:

```text
test is deterministic;
test is automatable;
test maps to normative requirement;
test does not enforce implementation-specific algorithm;
test produces Conformance Artifact;
test has fixture set;
test has clear PASS / FAIL semantics.
```

Conformance Specification already defines profile-based validation from C0 to C5 and requires checking ontology, Core/ABI, protocols, CSU contracts, failure-to-evidence behavior and Operational/Evolution separation. 

---

# 11. RFC-E — Reference Implementation RFC

## 11.1 Scope

RFC-E змінює Book IV або reference implementation.

Може включати:

```text
repository layout;
reference storage backend;
reference CLI;
reference CSU implementation;
reference test harness;
reference local protocol adapter;
developer tooling.
```

## 11.2 Admission Criteria

RFC-E допускається, якщо:

```text
не змінює Book 0–III;
позначає implementation-specific choices;
не створює hidden normative requirement;
має migration path;
має tests;
може бути замінена іншою implementation.
```

Book IV є reference/non-normative і показує мінімальний runnable AIRA node без обов’язкових ML, GPU, blockchain, federation або distributed execution. 

---

# 12. RFC-R — Research RFC

## 12.1 Scope

RFC-R додає або уточнює Research Track.

Приклади:

```text
Exploratory Resolution;
Cognitive Field;
Capability Algebra;
Distributed Scientific Method;
Self-Organizing Constellations;
Advanced PHM;
Goal Compiler;
Human Final Collapse UX research.
```

## 12.2 Admission Criteria

RFC-R допускається, якщо:

```text
не впливає на Operational Plane напряму;
працює через Research CSU / Artifacts;
має research budget model;
має safety boundary;
зберігає negative results;
має promotion criteria;
не змінює Core без RFC-A.
```

Book V уже фіксує Research Track як non-normative контур, що не є умовою сумісної реалізації і не має права руйнувати Core, protocols, CSU contracts або Operational Plane. 

---

# 13. RFC-P — Research Promotion RFC

## 13.1 Scope

RFC-P переводить validated Research Artifact у:

```text
Reference;
Optional;
Optimization;
Protocol extension;
CSU contract;
Schema extension;
Conformance test;
або відхиляє promotion.
```

## 13.2 Required Evidence

RFC-P **MUST** містити:

```text
research artifact refs;
evidence refs;
counter-evidence refs;
replication evidence;
measured benefit;
risk analysis;
privacy analysis;
compatibility analysis;
rollback path;
conformance tests;
promotion scope.
```

## 13.3 Promotion Gate

Research може перейти до Reference лише якщо:

```text
дає вимірюване зменшення total cost;
не змінює Core;
сумісний з Book 0–III;
має validation evidence;
має rollback;
має conformance coverage;
не створює hidden dependency;
не виконує hidden normative collapse.
```

---

# 14. RFC-X — Deprecation / Supersession RFC

## 14.1 Scope

RFC-X позначає документ, концепцію, schema, protocol або CSU contract як:

```text
Deprecated
Superseded
Historical
Rejected
Removed from Active Corpus
```

## 14.2 Admission Criteria

RFC-X допускається, якщо:

```text
newer document fully covers old content;
remaining unique fragments are migrated;
compatibility impact is documented;
historical provenance is preserved;
active corpus becomes smaller;
no current conformance test depends on removed item.
```

Specification Control already defines source pruning and identifies superseded early documents; RFC-X is the controlled mechanism for applying this policy. 

---

# 15. RFC Metadata Template

Кожна RFC **MUST** починатися з metadata block.

```yaml
rfc_id: AIRA-RFC-0000
title: string
class: enum  # A | B | C | D | S | T | E | R | P | X
status: enum # draft | review | accepted | rejected | superseded | withdrawn
author: identity_ref
created_at: timestamp
updated_at: timestamp
target_books:
  - Book0 | BookI | BookII | BookIII | BookIV | BookV | SchemaPack | Conformance | SCD
affected_versions:
  standard: string
  abi: string | null
  protocol: string | null
  schema: string | null
  csu_contract: string | null
dependencies:
  - rfc_ref
supersedes:
  - rfc_ref
superseded_by:
  - rfc_ref
discussion_refs:
  - artifact_or_thread_ref
evidence_refs:
  - artifact_ref
signature: signature
```

---

# 16. RFC Document Template

```text
# AIRA-RFC-0000 — Title

## 1. Summary

One-paragraph description.

## 2. Problem Statement

What problem exists in the current standard?

## 3. Motivation

Why is solving this problem valuable?

## 4. Scope

What is included?

## 5. Non-Goals

What is explicitly excluded?

## 6. Current Behavior

How does AIRA behave before this RFC?

## 7. Proposed Change

Precise change.

## 8. Affected Books / Schemas / Tests

List exact affected artifacts.

## 9. Compatibility Impact

Backward-compatible / breaking / additive / research-only.

## 10. Security Impact

Security analysis.

## 11. Privacy Impact

Privacy analysis.

## 12. Policy Impact

Policy Gate / federation / user rights / human collapse impact.

## 13. Failure Semantics

New or changed failure behavior.

## 14. Rollback Plan

How to revert safely.

## 15. Conformance Tests

New or changed tests.

## 16. Migration Plan

How existing implementations migrate.

## 17. Alternatives Considered

Rejected approaches.

## 18. Evidence

Evidence refs, experiments, benchmarks, proofs.

## 19. Open Questions

Unresolved issues.

## 20. Decision

Accepted / rejected / deferred.
```

---

# 17. Change Lifecycle

Кожна RFC проходить однаковий життєвий цикл.

```text
Proposal
↓
Draft
↓
Triage
↓
Technical Review
↓
Compatibility Review
↓
Security / Privacy Review
↓
Conformance Review
↓
Decision
↓
Implementation
↓
Validation
↓
Publication
↓
Supersession / Archive
```

---

# 18. RFC Status Model

```text
proposed
draft
triaged
under_review
accepted
accepted_with_changes
rejected
deferred
withdrawn
implemented
validated
superseded
archived
```

## 18.1 proposed

Ідея подана, але ще не перевірена.

## 18.2 draft

Автор оформив мінімальний RFC template.

## 18.3 triaged

Визначено RFC class і affected scope.

## 18.4 under_review

Йде технічний, суміснісний, безпековий і privacy review.

## 18.5 accepted

Зміна прийнята.

## 18.6 implemented

Є implementation або reference patch.

## 18.7 validated

Conformance tests проходять.

## 18.8 superseded

Замінено новішою RFC.

---

# 19. Triage Rules

Triage відповідає лише на п’ять питань:

```text
1. Чи це справжня проблема?
2. Чи проблема вже вирішена існуючим Book / RFC?
3. Чи потрібна зміна стандарту?
4. Чи можна реалізувати це як CSU / Algorithm / Artifact?
5. Який мінімальний RFC class?
```

Якщо проблему можна вирішити без зміни Book 0–III, RFC-A/B/C/D не допускається.

---

# 20. Compatibility Impact Model

RFC **MUST** класифікувати compatibility impact:

```text
No Impact
Additive Compatible
Behavior-Compatible
Schema-Compatible
Protocol-Compatible
Conditionally Compatible
Breaking
Research-Only
Reference-Only
```

## 20.1 No Impact

Не впливає на реалізації.

## 20.2 Additive Compatible

Додає optional behavior без ламання старих реалізацій.

## 20.3 Schema-Compatible

Додає backward-compatible schema fields.

## 20.4 Protocol-Compatible

Додає protocol extension із version negotiation.

## 20.5 Breaking

Вимагає major version або compatibility bridge.

---

# 21. Versioning Rules

## 21.1 Major Version

Потрібна, якщо RFC:

```text
змінює Core Ontology;
ламає ABI;
ламає protocol envelope;
ламає artifact immutability;
ламає Verified Result Artifact contract;
змінює conformance profile semantics.
```

## 21.2 Minor Version

Потрібна, якщо RFC:

```text
додає optional protocol;
додає CSU role;
додає schema extension;
додає conformance tests;
додає reference capability.
```

## 21.3 Patch Version

Потрібна, якщо RFC:

```text
уточнює формулювання;
виправляє неоднозначність;
не змінює behavior;
не змінює schemas;
не змінює conformance.
```

---

# 22. Rollback Requirements

Кожна non-trivial RFC **MUST** мати rollback plan.

Rollback plan містить:

```text
rollback trigger;
rollback steps;
affected artifacts;
affected protocols;
data migration reversal;
compatibility bridge removal;
conformance re-run;
risk after rollback.
```

RFC без rollback plan може бути прийнята лише якщо вона:

```text
documentation-only;
research-only;
або явно irreversible with human approval.
```

---

# 23. Conformance Requirements

Кожна RFC, що змінює normative behavior, **MUST** додати або змінити conformance tests.

Мінімальні поля:

```yaml
test_id: string
profile: C0 | C1 | C2 | C3 | C4 | C5
requirement_ref: string
fixture_refs:
  - artifact_ref
expected_result: PASS | FAIL
failure_event_required: boolean
evidence_required: boolean
```

RFC без conformance tests не може змінити Book 0–III.

---

# 24. Evidence Requirements

RFC **SHOULD** містити Evidence Artifact.

Для різних RFC classes потрібні різні Evidence.

```text
RFC-A: proof of contradiction / incompleteness.
RFC-B: ABI necessity proof.
RFC-C: interoperability need.
RFC-D: CSU role necessity.
RFC-S: schema validation need.
RFC-T: missing coverage proof.
RFC-E: implementation value.
RFC-R: research hypothesis.
RFC-P: validation + measured benefit.
RFC-X: supersession coverage matrix.
```

---

# 25. Supersession Coverage Matrix

Для RFC-X обов’язкова матриця покриття.

```yaml
superseded_document: string
replacement_documents:
  - artifact_ref
coverage:
  ontology: percent
  invariants: percent
  protocols: percent
  csu_contracts: percent
  schemas: percent
  research: percent
remaining_unique_items:
  - item: string
    action: migrated | rejected | archived
decision: supersede | keep | merge_first
```

Документ може бути вилучений з активного корпусу лише якщо:

```text
coverage == 100%;
remaining_unique_items == empty або migrated/rejected;
no conformance dependency remains.
```

---

# 26. Architecture Change Gate

RFC-A проходить додатковий gate.

```text
A0 Reality Primacy Check
A1 Problem Primacy Check
A2 Context Primacy Check
A3 Reuse Before Compute Check
A4 Minimal Necessary Action Check
A5 Evidence Primacy Check
A6 Human Final Collapse Check
A7 Evolution Separation Check
```

Якщо RFC-A порушує хоча б один First Principle, вона відхиляється або переводиться в Research.

---

# 27. Core Change Gate

RFC-B проходить Core Change Gate.

```text
Can it be a CSU?
Can it be a Protocol?
Can it be an Artifact?
Can it be a Schema Extension?
Can it be Book IV implementation?
Can it be Book V research?
```

Якщо відповідь "так" хоча б на одне питання — Core не змінюється.

---

# 28. Protocol Change Gate

RFC-C проходить Protocol Change Gate.

```text
Does it define interoperability?
Does it avoid algorithm mandates?
Does it preserve policy visibility?
Does it preserve privacy?
Does it preserve event causality?
Does it support version negotiation?
Does it produce failure events?
```

---

# 29. CSU Change Gate

RFC-D проходить CSU Change Gate.

```text
Single responsibility?
Replaceable?
Event-mediated?
Artifact-mediated?
Policy-gated?
Failure-transparent?
No Core mutation?
No hidden direct dependency?
```

---

# 30. Schema Change Gate

RFC-S проходить Schema Change Gate.

```text
Machine-validatable?
Backward-compatible?
Canonical serialization safe?
Hash/signature impact defined?
Extension namespace valid?
No forbidden core coupling?
Conformance fixtures updated?
```

---

# 31. Research Promotion Gate

RFC-P проходить Promotion Gate.

```text
Validated?
Replicated if needed?
Measured benefit?
No direct Operational influence?
No hidden normative collapse?
Compatible with Book 0–III?
Rollback exists?
Conformance tests exist?
```

---

# 32. Decision Model

RFC може бути прийнята лише після проходження всіх applicable gates.

Рішення:

```text
ACCEPT
ACCEPT_WITH_CHANGES
REJECT
DEFER
SEND_TO_RESEARCH
REQUIRE_MORE_EVIDENCE
SUPERSEDE
WITHDRAW
```

## 32.1 ACCEPT

Зміна приймається.

## 32.2 ACCEPT_WITH_CHANGES

Приймається після конкретних правок.

## 32.3 REJECT

Не відповідає стандарту або не дає користі.

## 32.4 DEFER

Проблема реальна, але рішення передчасне.

## 32.5 SEND_TO_RESEARCH

Ідея перспективна, але недостатньо доведена.

## 32.6 REQUIRE_MORE_EVIDENCE

Бракує Evidence.

---

# 33. Decision Record

Кожне рішення **MUST** створити Decision Artifact.

```yaml
decision_id: string
rfc_ref: string
decision: enum
decided_at: timestamp
deciders:
  - identity_ref
reason: string
evidence_refs:
  - artifact_ref
required_changes:
  - string
conformance_impact:
  - string
signature: signature
```

---

# 34. RFC Artifact Model

Кожна RFC **MUST** бути Artifact.

```yaml
artifact_id: string
artifact_type: RFCArtifact
rfc_id: string
rfc_class: string
status: string
content_hash: hash
provenance_refs:
  - artifact_or_event_ref
evidence_refs:
  - artifact_ref
decision_refs:
  - artifact_ref
policy_refs:
  - policy_ref
created_at: timestamp
signature: signature
```

RFC не є неформальним текстом. Це частина Artifact Lifecycle.

---

# 35. Minimal RFC Acceptance Checklist

Перед прийняттям RFC повинна мати:

```text
[ ] Problem Statement
[ ] Scope
[ ] Non-Goals
[ ] Affected Books
[ ] Compatibility Impact
[ ] Security Impact
[ ] Privacy Impact
[ ] Policy Impact
[ ] Failure Semantics
[ ] Rollback Plan
[ ] Conformance Tests
[ ] Evidence
[ ] Versioning Plan
[ ] Decision Artifact
```

---

# 36. Fast-Track RFC

Fast-track допускається лише для:

```text
typo fixes;
broken references;
non-semantic clarifications;
test fixture fixes;
schema examples that do not change validation;
Book IV implementation improvements without standard impact.
```

Fast-track **MUST NOT** змінювати:

```text
Book 0 ontology;
Book I ABI;
Book II protocols;
Book III CSU contracts;
Schema validation;
Conformance profile semantics.
```

---

# 37. Emergency RFC

Emergency RFC допускається для:

```text
security vulnerability;
privacy leak;
critical conformance break;
artifact integrity issue;
signature validation bug;
policy bypass.
```

Emergency RFC може бути прийнята швидше, але post-review **MUST** бути виконаний після стабілізації.

---

# 38. Rejection Criteria

RFC відхиляється, якщо вона:

```text
дублює існуючу сутність;
додає Core complexity без необхідності;
не має вимірюваної користі;
ламає Book 0;
вимагає ML/GPU/blockchain у Core;
обходить Policy Gate;
не має failure semantics;
не має rollback;
не має conformance tests;
змішує Research і Operational Plane;
приховано виконує normative collapse.
```

---

# 39. Active Corpus Protection

RFC-процес повинен захищати активний корпус від розростання.

Новий документ додається до активного корпусу лише якщо він:

```text
є Book update;
є accepted RFC;
є Conformance update;
є Schema update;
є Reference update;
є Research artifact з активним статусом.
```

Усі інші документи:

```text
Superseded
Historical
Rejected
Research Archive
```

---

# 40. RFC Numbering

Формат:

```text
AIRA-RFC-0001
```

Класифікований формат:

```text
AIRA-RFC-A-0001
AIRA-RFC-B-0001
AIRA-RFC-C-0001
AIRA-RFC-D-0001
AIRA-RFC-S-0001
AIRA-RFC-T-0001
AIRA-RFC-E-0001
AIRA-RFC-R-0001
AIRA-RFC-P-0001
AIRA-RFC-X-0001
```

Номер ніколи не перевикористовується.

---

# 41. RFC Repository Layout

```text
rfcs/
 ├── accepted/
 │   ├── architecture/
 │   ├── core/
 │   ├── protocols/
 │   ├── csu/
 │   ├── schemas/
 │   ├── conformance/
 │   ├── reference/
 │   ├── research/
 │   ├── promotion/
 │   └── supersession/
 │
 ├── draft/
 ├── rejected/
 ├── withdrawn/
 ├── superseded/
 └── templates/
```

---

# 42. RFC Template File

```text
rfcs/templates/aira-rfc-template.md
```

Required machine-readable metadata file:

```text
rfcs/templates/aira-rfc-template.yaml
```

---

# 43. Governance Minimalism

AIRA governance must follow the same principle as AIRA architecture:

```text
мінімально достатня дія;
мінімально достатній документ;
мінімально достатня зміна;
максимальне повторне використання існуючих контрактів.
```

RFC-процес не повинен стати бюрократичним аналогом моноліту.

Його задача — не ускладнювати розвиток, а захищати архітектуру від неконтрольованого розщеплення.

---

# 44. Status

AIRA RFC Template & Change Process v0.1 формалізує контрольовану еволюцію AIRA.

Він закріплює:

```text
RFC classes;
RFC metadata;
RFC lifecycle;
architecture change gate;
core change gate;
protocol change gate;
CSU change gate;
schema change gate;
research promotion gate;
compatibility model;
rollback requirements;
conformance requirements;
supersession process;
active corpus protection.
```

Цей документ напряму підтримує головну мету AIRA:

```text
Problem Statement
↓
Progressive Resolution
↓
Verified Result Artifact
↓
Evidence
↓
Evolution
↓
Lower-cost future Resolution
```

Подальший рекомендований документ:

```text
AIRA MVP Implementation Roadmap v0.1
```

Scope:

```text
minimal runnable node;
implementation milestones;
repository structure;
C0 → C1 → C2 profile path;
reference CSU priorities;
test harness implementation;
schema registry implementation;
first demo flow;
risks;
definition of done.
```
