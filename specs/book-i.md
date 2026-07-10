# Book I — Core Architecture & ABI v0.1

```text
Type: Normative
Scope: AIRA Core
Status: Draft v0.1
Depends on: Book 0 — Mathematical & Conceptual Foundations
Exports: Core Object Model, Stable ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary, CSU Runtime Contract
```

Book I описує **ядро AIRA**. Він не описує алгоритми, евристику, ML, discovery, CRP, PHM, KnowledgeOps або конкретну реалізацію. Book I визначає лише те, без чого незалежні реалізації AIRA не зможуть бути сумісними. Це узгоджується з Book 0, де зафіксовано місію `Problem Statement → Progressive Resolution → Verified Result Artifact` і межу ядра як механізму, а не інтелектуального шару. 

---

# 1. Core Mission

AIRA Core існує для забезпечення стабільного середовища, у якому незалежні CSU можуть перетворювати `Problem Statement` на `Verified Result Artifact` через канонічні об’єкти, події, політики та контракти.

Core **не вирішує задачі**.

Core гарантує:

```text
об'єктну модель;
стабільний ABI;
поширення подій;
перевірку політик;
перевірку інваріантів;
межу безпеки;
ізольоване підключення CSU.
```

---

# 2. Non-Goals

AIRA Core **MUST NOT** містити:

```text
LLM logic
GPU logic
scheduler logic
routing logic
ML models
prediction
PHM
blockchain consensus
knowledge algorithms
optimization heuristics
domain-specific logic
```

Це прямо випливає з мікроядерної моделі: ядро містить лише механізми, тоді як Discovery, CRP, LLM Backend, Verification, PHM, Knowledge, Federation та інші компоненти працюють як зовнішні модулі/CSU. 

---

# 3. Core Boundary

AIRA Core складається лише з таких підсистем:

```text
Object Model
Stable ABI
Event Runtime
Policy Gate
Invariant Checker
Security Boundary
CSU Runtime
```

Будь-який компонент, що приймає предметне рішення, оптимізує маршрут, аналізує знання, прогнозує стан або виконує обчислення, **MUST** бути винесений за межі Core.

---

# 4. Core Object Model

Core оперує лише канонічними об’єктами Book 0:

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

Ці сутності утворюють мінімальний набір, достатній для опису всієї AIRA. GPU, LLM, Node, Driver, Scheduler, Blockchain, Wallet, HIE, Model Registry та Knowledge Graph не є фундаментальними об’єктами ядра. Вони можуть бути реалізовані як CSU, Artifact, Protocol або Research-компоненти. 

---

# 5. Object Requirements

Кожен Core Object **MUST** мати мінімальний дескриптор:

```yaml
object_id: string
object_type: string
schema_version: string
created_at: timestamp
producer_identity: identity_ref
policy_refs: [policy_ref]
provenance_refs: [artifact_or_event_ref]
content_hash: hash
signature: signature
```

Об’єкти ядра **MUST** бути immutable.

Будь-яка зміна стану **MUST** створювати новий об’єкт або подію, а не змінювати попередній об’єкт in-place. Це відповідає Stable ABI, де Kernel Objects є immutable і змінюються лише через події. 

---

# 6. Handle Model

Core не передає CSU прямі вказівники на внутрішні структури.

Усі взаємодії виконуються через handles.

```text
ProblemHandle
ContextHandle
EvidenceHandle
CapsuleHandle
CapabilityHandle
ArtifactHandle
EventHandle
PolicyHandle
CSUHandle
ResultHandle
```

Handle є непрозорим ідентифікатором.

CSU **MUST NOT** робити припущення про внутрішню структуру handle.

---

# 7. Stable ABI

Stable ABI — головний контракт між Core та CSU.

ABI **MUST** бути стабільнішим за будь-який CSU, Protocol або Algorithm.

Мінімальний ABI включає:

```text
object_open(handle)
object_read(handle, selector)
object_emit(event_descriptor)
object_subscribe(event_filter)
policy_check(policy_query)
capability_register(capability_descriptor)
artifact_publish(artifact_descriptor)
artifact_resolve(artifact_ref)
csu_register(csu_manifest)
csu_suspend(csu_handle)
csu_resume(csu_handle)
csu_unload(csu_handle)
```

ABI не повинен містити понять:

```text
GPU
model
prompt
wallet
route
scheduler
blockchain
```

Якщо нова функція потребує додавання таких понять до ABI, вона не належить до Core.

---

# 8. Event Runtime

Event Runtime є єдиним механізмом координації між Core і CSU.

CSU **MUST NOT** викликати один одного напряму.

Взаємодія відбувається лише так:

```text
CSU
  ↓ emits
Event
  ↓ validated by Core
Event Runtime
  ↓ observed by
CSU
```

Подія має мінімальний дескриптор:

```yaml
event_id: string
event_type: string
schema_version: string
producer_identity: identity_ref
causal_refs: [event_ref]
object_refs: [object_ref]
policy_refs: [policy_ref]
payload_hash: hash
signature: signature
created_at: timestamp
```

Event Runtime **MUST** підтримувати причинні зв’язки між подіями.

Event Runtime **MUST NOT** гарантувати глобальний синхронний стан системи.

---

# 9. Event Classes

Book I фіксує лише базові класи подій:

```text
ObjectCreated
ObjectReferenced
ArtifactPublished
CapabilityRegistered
PolicyEvaluated
CapsuleCreated
CapsuleBound
CapsuleCompleted
VerificationCompleted
ResultPublished
CSURegistered
CSUSuspended
CSUFailed
InvariantViolation
```

Конкретні доменні події визначаються Book II або CSU-specific contracts.

---

# 10. Policy Gate

Policy Gate — єдиний механізм перевірки обмежень.

Policy Gate не виконує дій.

Policy Gate повертає лише:

```text
ALLOW
DENY
REQUIRE
```

Policy Query має форму:

```yaml
subject: identity_ref
action: string
object: object_ref
context_refs: [context_ref]
evidence_refs: [evidence_ref]
requested_at: timestamp
```

Жодна операція Core не може бути виконана без Policy Gate, якщо для неї існує застосовна Policy.

---

# 11. Invariant Checker

Invariant Checker перевіряє незмінні правила Book 0.

Мінімальний набір перевірок:

```text
object immutability;
artifact provenance;
event signature;
policy enforcement;
CSU isolation;
no direct CSU-to-CSU mutation;
no Core semantic leakage;
Verified Result Artifact completeness;
Operational/Evolution separation.
```

Invariant Checker **MUST** мати право зупинити операцію, якщо вона порушує Book 0.

Invariant Checker **MUST NOT** приймати оптимізаційні рішення.

---

# 12. Security Boundary

Кожен CSU працює в ізоляції.

CSU **MUST NOT** мати прямий доступ до:

```text
Core memory
foreign CSU state
private Artifact payloads
user data
policy internals
raw execution data без дозволу
```

Доступ надається лише через ABI та Policy Gate.

Security Boundary **SHOULD** підтримувати:

```text
sandboxing
capability-based permissions
signed CSU manifests
artifact integrity checks
event signature verification
revocation
least privilege
```

---

# 13. CSU Runtime

CSU — єдина допустима форма розширення функціональності AIRA поза ядром.

CSU може бути:

```text
Context CSU
Execution CSU
Evidence CSU
Verification CSU
Artifact CSU
Discovery CSU
Settlement CSU
PHM CSU
Optimization CSU
Research CSU
```

Core не знає типу внутрішньої реалізації CSU.

CSU декларує лише:

```yaml
csu_id: string
csu_type: string
abi_version: string
capabilities: [capability_descriptor]
permissions: [permission]
event_subscriptions: [event_filter]
artifact_types: [artifact_type]
signature: signature
```

Термін `Driver` у старих документах замінюється на `CSU`. Driver може залишатися лише технічним підтипом CSU для низькорівневих адаптерів.

---

# 14. CSU Lifecycle

Канонічний життєвий цикл CSU:

```text
Discovered
↓
Registered
↓
Verified
↓
Active
↓
Suspended
↓
Revoked
↓
Archived
```

## 14.1 Registered

CSU подав manifest.

## 14.2 Verified

Core перевірив:

```text
ABI version;
signature;
permissions;
capability descriptors;
policy compatibility.
```

## 14.3 Active

CSU може споживати та генерувати події.

## 14.4 Suspended

CSU тимчасово відключений, але його Artifact та Events залишаються валідними.

## 14.5 Revoked

CSU більше не має права створювати нові події.

## 14.6 Archived

Історія CSU зберігається як Artifact provenance.

---

# 15. Capability Registration

Capability описує не фізичний ресурс, а можливість зменшити невизначеність або матеріалізувати частину рішення.

Мінімальний дескриптор Capability:

```yaml
capability_id: string
capability_type: string
provider_csu: csu_ref
constraints: object
cost_model_ref: artifact_ref
policy_refs: [policy_ref]
evidence_refs: [evidence_ref]
confidence: number
scope: object
```

Core не інтерпретує зміст Capability.

Core лише перевіряє формат, підпис, політики та інваріанти.

---

# 16. Artifact Runtime

Artifact Runtime відповідає за публікацію, посилання, перевірку цілісності та provenance Artifact.

Artifact **MUST** бути content-addressed або мати еквівалентний механізм цілісності.

Мінімальний Artifact Descriptor:

```yaml
artifact_id: string
artifact_type: string
schema_version: string
content_hash: hash
producer_identity: identity_ref
provenance_refs: [artifact_or_event_ref]
dependency_refs: [artifact_ref]
policy_refs: [policy_ref]
signature: signature
created_at: timestamp
```

Artifact Runtime не знає семантики Artifact.

Knowledge Artifact, Result Artifact, Policy Artifact, Execution Artifact, Research Artifact — усі проходять через єдиний механізм.

---

# 17. Verified Result Artifact

Core не створює Verified Result Artifact самостійно.

Core гарантує, що опублікований `Verified Result Artifact` відповідає контракту.

Мінімальний Result Descriptor:

```yaml
result_id: string
problem_statement_ref: object_ref
context_ref: object_ref
solution_refs: [artifact_ref]
evidence_refs: [evidence_ref]
verification_status: enum
confidence: number
scope: object
provenance_refs: [artifact_or_event_ref]
artifact_hash: hash
signature: signature
created_at: timestamp
```

Verified Result Artifact є єдиним штатним завершенням Operational Pipeline.

---

# 18. Operational / Evolution Separation

Core **MUST** гарантувати, що Evolution Plane не змінює Operational Plane напряму.

Допустимий шлях:

```text
Evidence
↓
Validation
↓
Artifact Promotion
↓
Operational Use
```

Background/Evolution CSU можуть створювати Evidence, Hypothesis, Research або Knowledge Artifact.

Вони не можуть створювати Operational Artifact без Promotion.

---

# 19. Failure Model

Core не приховує failure.

Будь-який failure **MUST** бути представлений як Event або Artifact.

Мінімальні failure-події:

```text
CSUFailed
PolicyDenied
InvariantViolation
ArtifactInvalid
VerificationFailed
CapsuleFailed
CapabilityUnavailable
```

Failure не є "порожнім результатом".

Failure створює Evidence для Evolution Plane.

---

# 20. Compatibility

Дві реалізації AIRA Core вважаються сумісними, якщо вони:

```text
підтримують однакову Core Object Model;
підтримують сумісний Stable ABI;
однаково перевіряють Core Invariants;
приймають сумісні Event Descriptor;
приймають сумісні Artifact Descriptor;
можуть створювати й перевіряти сумісний Verified Result Artifact.
```

Внутрішня реалізація може відрізнятися повністю.

---

# 21. Versioning

Book I вводить незалежне версіонування:

```text
Core ABI Version
Object Schema Version
Event Schema Version
Artifact Schema Version
Policy Schema Version
CSU Manifest Version
```

Зміна алгоритму не змінює Core ABI.

Зміна CSU не змінює Core ABI.

Зміна протоколу не повинна змінювати Core ABI, якщо її можна виразити через існуючі Artifact, Event або CSU contracts.

---

# 22. Conformance Levels

## Level 0 — Minimal Core

Підтримує:

```text
Object Model
Handle Model
Event Runtime
Policy Gate
Invariant Checker
Security Boundary
```

## Level 1 — Local Core

Додає:

```text
local CSU Runtime
local Artifact Runtime
local Verified Result validation
```

## Level 2 — Federated Core

Додає сумісність з Book II protocols.

## Level 3 — Reference Compatible

Повністю сумісний із Book I–III.

---

# 23. Заборонені реалізаційні практики

Implementation **MUST NOT**:

```text
додавати предметну логіку в Core;
дозволяти прямі виклики CSU-to-CSU поза ABI;
змінювати Artifact in-place;
обходити Policy Gate;
приховувати failure;
змінювати Core Object Model без зміни версії;
використовувати ML у Kernel;
залежати від конкретного GPU, LLM або blockchain.
```

---

# 24. Нормативна формула Book I

```text
AIRA Core =
    Object Model
  + Stable ABI
  + Event Runtime
  + Policy Gate
  + Invariant Checker
  + Security Boundary
  + CSU Runtime
```

Усе інше є:

```text
Protocol
CSU
Algorithm
Artifact
Research
Implementation
```

---

# 25. Статус Book I

Book I v0.1 формалізує ядро AIRA.

Він закріплює мікроядерну архітектуру, Stable ABI, immutable object model, event-native координацію, policy enforcement, invariant checking, ізоляцію CSU та механізм Verified Result Artifact.

Наступний нормативний документ:

```text
Book II — Protocol Specifications v0.1
```

Його scope:

```text
Discovery Protocol
Artifact Protocol
Event Protocol
Federation Protocol
Identity Protocol
Settlement Protocol
CRP / Capability Routing Protocol
```

Book II не має права описувати алгоритми. Він описує лише мережеві та між-CSU контракти.
