# Book III — CSU & ABI Contracts v0.1

```text
Type: Normative
Scope: AIRA CSU Layer
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
Exports:
  - CSU Contract Model
  - CSU Manifest Schema
  - CSU Lifecycle Contract
  - CSU Capability Declaration Contract
  - CSU Event Interaction Contract
  - CSU Artifact Interaction Contract
  - Standard CSU Role Contracts
```

Book III описує **Cognitive Service Units (CSU)** — ізольовані, замінні функціональні одиниці AIRA, які працюють поверх Core ABI, взаємодіють через Event/Artifact/Policy contracts і реалізують конкретні функції без зміни ядра. Термін `Driver` з попередніх документів вважається застарілим і заміненим на `CSU`; Book I уже формалізував Core як Object Model + Stable ABI + Event Runtime + Policy Gate + Invariant Checker + Security Boundary + CSU Runtime.  

---

# 1. Scope

Book III визначає:

```text
CSU identity;
CSU manifest;
CSU lifecycle;
CSU capability declaration;
CSU event subscriptions;
CSU artifact contracts;
CSU failure semantics;
standard CSU roles;
CSU conformance levels.
```

Book III не визначає:

```text
алгоритми маршрутизації;
моделі ML;
PHM prediction logic;
knowledge aggregation logic;
storage backend;
transport implementation;
settlement ledger implementation;
конкретну reference implementation.
```

Ці речі належать до Book IV або Book V. Book II уже зафіксував, що протокольний шар описує лише контракти, а не конкретні алгоритми. 

---

# 2. CSU Definition

## 2.1 Канонічне визначення

**CSU (Cognitive Service Unit)** — це ізольований функціональний модуль AIRA, який:

```text
має Identity;
публікує Capability;
працює через Stable ABI;
споживає та генерує Events;
публікує або використовує Artifacts;
підкоряється Policy Gate;
не змінює Core;
може бути замінений сумісним CSU.
```

CSU є єдиною допустимою формою функціонального розширення AIRA поза Core.

---

## 2.2 CSU не є

CSU не є:

```text
Kernel component;
global scheduler;
shared mutable service;
trusted monolith;
direct peer dependency;
physical node;
GPU binding;
LLM binding;
blockchain smart contract.
```

Фізичний вузол, GPU, модель, backend, blockchain, wallet або scheduler можуть бути представлені через CSU, Capability, Artifact або Protocol, але не є фундаментальними сутностями Core Ontology. 

---

# 3. Normative Keywords

```text
MUST      — обов'язкова вимога;
MUST NOT  — заборона;
SHOULD    — рекомендована вимога;
MAY       — дозволена опція.
```

---

# 4. CSU Design Principles

Кожен CSU **MUST** дотримуватись таких принципів:

```text
single responsibility;
no direct CSU-to-CSU mutation;
no shared mutable state;
event-mediated coordination;
artifact-mediated data exchange;
policy-before-action;
least privilege;
replaceability;
explicit capability declaration;
failure transparency;
provenance preservation.
```

Це продовжує Lightweight Architecture Doctrine: ядро не повинно містити алгоритми, ML, scheduler, blockchain, GPU або LLM як обов’язкові залежності; складність локалізується в замінних модулях. 

---

# 5. CSU Manifest Contract

Кожен CSU **MUST** мати manifest.

```yaml
csu_id: string
csu_name: string
csu_type: string
csu_version: string
abi_version: string
manifest_version: string

identity_ref: identity_ref
publisher_identity: identity_ref

capabilities:
  - capability_descriptor

permissions:
  - permission_descriptor

event_subscriptions:
  - event_filter

event_outputs:
  - event_type_descriptor

artifact_inputs:
  - artifact_type_descriptor

artifact_outputs:
  - artifact_type_descriptor

policy_refs:
  - policy_ref

resource_requirements:
  cpu: requirement | null
  memory: requirement | null
  storage: requirement | null
  accelerator: requirement | null
  network: requirement | null

sandbox:
  filesystem: enum
  network: enum
  process: enum
  device_access: enum
  secret_access: enum

lifecycle_hooks:
  init: boolean
  activate: boolean
  suspend: boolean
  resume: boolean
  shutdown: boolean

provenance_refs:
  - artifact_ref

signature: signature
created_at: timestamp
```

## 5.1 Manifest rules

CSU manifest:

```text
MUST be signed;
MUST declare ABI version;
MUST declare all required permissions;
MUST declare all advertised capabilities;
MUST declare artifact input/output types;
MUST declare event subscriptions;
MUST be immutable after publication;
MUST be superseded by a new manifest version if changed.
```

---

# 6. CSU Lifecycle

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

## 6.1 Discovered

CSU manifest знайдений через локальне сховище, Artifact Registry, Federation Protocol або інший дозволений механізм.

## 6.2 Registered

CSU подав manifest у Core через ABI.

Core **MUST** перевірити:

```text
manifest schema;
signature;
publisher identity;
declared ABI version;
policy compatibility;
permission scope.
```

## 6.3 Verified

CSU пройшов перевірку сумісності.

Verification не означає, що CSU "правильний" або "оптимальний".

Verification означає лише:

```text
CSU може безпечно працювати через ABI
і не порушує Core Invariants.
```

## 6.4 Active

CSU може:

```text
споживати Events;
публікувати Events;
публікувати Artifacts;
реєструвати Capabilities;
обробляти дозволені Object Handles.
```

## 6.5 Suspended

CSU тимчасово не виконує роботу.

Його раніше створені Artifacts та Events залишаються валідними.

## 6.6 Revoked

CSU більше не має права створювати нові Events або Artifacts.

Revocation **MUST** створити Event.

## 6.7 Archived

CSU збережений як історичний Artifact для provenance.

---

# 7. CSU ABI Usage Contract

Book I визначає Stable ABI. Book III визначає, як CSU **MUST** його використовувати.

## 7.1 Мінімальна ABI-взаємодія

CSU **MAY** використовувати такі ABI operations:

```text
csu_register(manifest)
capability_register(capability_descriptor)
object_open(handle)
object_read(handle, selector)
object_emit(event_descriptor)
object_subscribe(event_filter)
artifact_publish(artifact_descriptor)
artifact_resolve(artifact_ref)
policy_check(policy_query)
```

CSU **MUST NOT**:

```text
читати Core memory;
змінювати Core Object in-place;
змінювати чужий Artifact;
викликати інший CSU напряму;
обходити Policy Gate;
створювати Events без підпису;
публікувати Artifact без provenance.
```

---

# 8. Capability Declaration Contract

Capability описує **можливість**, а не залізо.

Мінімальний descriptor:

```yaml
capability_id: string
capability_type: string
schema_version: string
provider_csu: csu_ref

input_artifact_types:
  - artifact_type

output_artifact_types:
  - artifact_type

required_context:
  - context_requirement

constraints:
  latency_max: duration | null
  cost_max: cost_value | null
  privacy_class: string | null
  trust_min: number | null
  reliability_min: number | null

cost_model_ref: artifact_ref | null
evidence_refs:
  - evidence_ref

confidence: number
scope: scope_descriptor

policy_refs:
  - policy_ref

signature: signature
```

## 8.1 Capability rules

Capability:

```text
MUST describe ability, not implementation;
MUST include provider CSU;
MUST include scope;
MUST include policy constraints;
SHOULD include cost model;
SHOULD include evidence references;
MAY include PHM/risk metadata.
```

---

# 9. Event Interaction Contract

CSU взаємодіє з системою через Event.

## 9.1 Event subscription

```yaml
subscription_id: string
subscriber_csu: csu_ref
event_filter:
  event_types: [string]
  object_types: [string]
  artifact_types: [string]
  scope: scope_descriptor
policy_refs:
  - policy_ref
signature: signature
```

## 9.2 Event output

CSU output event **MUST** include:

```yaml
event_id: string
event_type: string
producer_csu: csu_ref
causal_refs:
  - event_ref
object_refs:
  - object_ref
artifact_refs:
  - artifact_ref
policy_refs:
  - policy_ref
payload_hash: hash
signature: signature
created_at: timestamp
```

## 9.3 Rules

CSU:

```text
MUST preserve causal_refs;
MUST emit failure events;
MUST NOT depend on total global ordering;
MUST handle duplicate events idempotently;
MUST tolerate missing optional events;
MUST NOT assume direct delivery to another CSU.
```

---

# 10. Artifact Interaction Contract

CSU **MUST** exchange durable data through Artifacts.

## 10.1 Artifact input declaration

```yaml
artifact_type: string
schema_version: string
required: boolean
policy_requirements:
  - policy_ref
validation_requirements:
  hash_required: boolean
  signature_required: boolean
  provenance_required: boolean
```

## 10.2 Artifact output declaration

```yaml
artifact_type: string
schema_version: string
content_hash: hash
producer_csu: csu_ref
provenance_refs:
  - artifact_or_event_ref
dependency_refs:
  - artifact_ref
policy_refs:
  - policy_ref
signature: signature
```

## 10.3 Rules

CSU:

```text
MUST NOT mutate Artifacts;
MUST publish new Artifact versions for changes;
MUST preserve provenance;
MUST validate input Artifact policies;
MUST produce output Artifact descriptors;
MUST fail safely if Artifact validation fails.
```

Artifact Lifecycle Architecture already established the general rule that Knowledge Package, Model Profile, Context Profile, PHM Model, Routing Heuristic and Policy Package are all Artifact specializations. 

---

# 11. Policy Compliance Contract

Before performing any action affecting Core Objects, Artifacts, Events or Capability binding, CSU **MUST** perform Policy check.

## 11.1 Policy query

```yaml
subject: identity_ref
csu_ref: csu_ref
action: string
object_refs:
  - object_ref
artifact_refs:
  - artifact_ref
context_refs:
  - context_ref
evidence_refs:
  - evidence_ref
requested_at: timestamp
```

## 11.2 Policy result

```yaml
decision: enum  # ALLOW | DENY | REQUIRE
requirements:
  - requirement_descriptor
reason_refs:
  - artifact_or_event_ref
signature: signature
```

CSU **MUST NOT** interpret `DENY` as failure of the system.

`DENY` is a valid policy outcome and **MUST** be represented as Event.

---

# 12. Standard CSU Role Classes

Book III defines standard CSU role classes.

These are **contracts**, not implementations.

A compatible implementation **MAY** implement multiple roles in one binary, but it **MUST** expose them as separate logical CSU contracts.

```text
Context CSU
Reduction CSU
Evidence CSU
Epistemic CSU
Execution CSU
Verification CSU
Artifact CSU
Discovery CSU
Federation CSU
Settlement CSU
Optimization CSU
Evolution CSU
Research CSU
```

---

# 13. Context CSU Contract

## 13.1 Purpose

Context CSU transforms `Problem Statement` into usable Context and may produce `Execution Intent`.

## 13.2 Inputs

```text
Problem Statement
User Context Artifact
Session Context Artifact
Domain Context Artifact
Policy Artifact
```

## 13.3 Outputs

```text
Context Artifact
Execution Intent Artifact
ContextResolutionEvent
```

## 13.4 Requirements

Context CSU:

```text
MUST interpret, not execute;
MUST NOT select physical resources;
MUST NOT produce final Result;
MUST preserve ambiguity when ambiguity remains;
MUST mark unresolved context explicitly;
SHOULD produce context confidence and scope.
```

---

# 14. Reduction CSU Contract

## 14.1 Purpose

Reduction CSU reduces Solution Space before expensive Materialization.

It may perform:

```text
ready solution lookup;
verified knowledge lookup;
external source lookup;
solution candidate filtering;
budget-aware escalation decision.
```

## 14.2 Inputs

```text
Problem Statement
Context Artifact
Evidence Artifact
Knowledge Artifact
Policy Artifact
```

## 14.3 Outputs

```text
Solution Candidate Artifact
Ready Solution Artifact
Negative Lookup Artifact
Escalation Recommendation Artifact
```

## 14.4 Requirements

Reduction CSU:

```text
MUST prefer reuse before compute;
MUST NOT materialize execution unless its contract allows it;
MUST produce Evidence for failed lookup;
MUST expose escalation reason;
MUST preserve multiple equivalent candidates.
```

---

# 15. Evidence CSU Contract

## 15.1 Purpose

Evidence CSU collects and normalizes Evidence.

## 15.2 Inputs

```text
Events
Execution outputs
Telemetry Artifacts
Failure Events
External observations
```

## 15.3 Outputs

```text
Evidence Artifact
Observation Artifact
Failure Evidence Artifact
```

## 15.4 Requirements

Evidence CSU:

```text
MUST collect observations;
MUST NOT assign final truth;
MUST NOT assign Epistemic Status unless also implementing Epistemic CSU contract;
MUST preserve source provenance;
MUST preserve counter-evidence;
MUST distinguish observation from conclusion.
```

---

# 16. Epistemic CSU Contract

## 16.1 Purpose

Epistemic CSU maps Evidence and Claims to Epistemic Status.

## 16.2 Inputs

```text
Claim Artifact
Evidence Artifact
Counter Evidence Artifact
Context Artifact
Policy Artifact
```

## 16.3 Outputs

```text
Epistemic Assessment Artifact
Epistemic Status Update Event
Knowledge Candidate Artifact
```

## 16.4 Required fields

```yaml
claim_ref: artifact_ref
evidence_refs: [evidence_ref]
counter_evidence_refs: [evidence_ref]
epistemic_status: enum
confidence: number
scope: scope_descriptor
contextual_fitness: number | null
revision_refs: [artifact_ref]
signature: signature
```

## 16.5 Requirements

Epistemic CSU:

```text
MUST separate Evidence, Confidence, Scope, Status;
MUST NOT delete old knowledge;
MUST produce revision history;
MUST support contradictory evidence;
MUST distinguish Fact, Theory, Hypothesis, Assumption, Myth, Absurd.
```

This follows the Epistemic Layer and Confidence Gradient correction: knowledge does not expire through TTL; trust and scope evolve through evidence.  

---

# 17. Execution CSU Contract

## 17.1 Purpose

Execution CSU materializes Execution Capsule.

Execution CSU does not decide whether execution is needed. It executes only after receiving valid Capsule binding and Policy clearance.

## 17.2 Inputs

```text
Execution Capsule
Context Artifact
Capability Binding
Policy Artifact
Input Artifacts
```

## 17.3 Outputs

```text
Output Artifact
Execution Event
CapsuleCompleted Event
CapsuleFailed Event
Execution Evidence Artifact
```

## 17.4 Requirements

Execution CSU:

```text
MUST execute only authorized Capsules;
MUST emit start/completion/failure events;
MUST preserve input/output provenance;
MUST NOT publish Verified Result Artifact directly unless also implementing Verification CSU;
MUST support cancellation or suspension where possible;
MUST expose resource usage evidence;
MUST fail transparently.
```

---

# 18. Verification CSU Contract

## 18.1 Purpose

Verification CSU determines whether Output can become Verified Result Artifact.

## 18.2 Inputs

```text
Output Artifact
Problem Statement
Context Artifact
Evidence Artifact
Policy Artifact
Verification Rule Artifact
```

## 18.3 Outputs

```text
Verification Artifact
Verified Result Artifact
Rejected Output Artifact
VerificationFailed Event
```

## 18.4 Requirements

Verification CSU:

```text
MUST distinguish Output from Result;
MUST produce verification_status;
MUST include confidence, scope, provenance;
MUST preserve failure evidence;
MUST NOT silently accept unverifiable output;
MUST support multiple verification strategies through artifacts.
```

---

# 19. Artifact CSU Contract

## 19.1 Purpose

Artifact CSU manages Artifact publication, resolution, verification, dependency tracking and supersession.

## 19.2 Inputs

```text
Artifact Descriptor
Artifact Content Reference
Policy Artifact
Dependency Artifacts
```

## 19.3 Outputs

```text
ArtifactPublished Event
ArtifactResolved Event
ArtifactInvalid Event
ArtifactSuperseded Event
```

## 19.4 Requirements

Artifact CSU:

```text
MUST verify hashes;
MUST verify signatures;
MUST preserve dependencies;
MUST preserve provenance;
MUST support immutable versioning;
MUST support supersession without mutation;
MUST NOT delete historical artifacts as normal operation.
```

---

# 20. Discovery CSU Contract

## 20.1 Purpose

Discovery CSU finds Capability providers, not physical nodes.

## 20.2 Inputs

```text
Capability Query
Context Artifact
Policy Artifact
Federation Descriptor
```

## 20.3 Outputs

```text
Capability Candidate Artifact
DiscoveryResponse Event
CapabilityUnavailable Event
```

## 20.4 Requirements

Discovery CSU:

```text
MUST search by Capability;
MUST NOT require global registry;
MUST NOT expose private topology unless policy allows;
MUST support local/federation/inter-federation scope;
MAY use DHT, registry, gossip, bootstrap list, address book.
```

This normalizes early Discovery and Overlay concepts into capability-based discovery rather than node/resource-centric lookup. 

---

# 21. Federation CSU Contract

## 21.1 Purpose

Federation CSU manages federation membership, import/export scope, protocol compatibility and federation policy.

## 21.2 Inputs

```text
Federation Descriptor
Identity Descriptor
Capability Advertisement
Policy Artifact
Evidence Artifact
```

## 21.3 Outputs

```text
FederationJoin Event
FederationAccepted Event
FederationRejected Event
FederationLimited Event
FederationRevoked Event
```

## 21.4 Requirements

Federation CSU:

```text
MUST preserve federation autonomy;
MUST support local policy;
MUST support limited participation;
MUST support revocation;
MUST NOT enforce global policy uniformity.
```

---

# 22. Settlement CSU Contract

## 22.1 Purpose

Settlement CSU produces receipts, contribution records and audit evidence.

It does not require blockchain.

## 22.2 Inputs

```text
Execution Event
Verification Artifact
Capability Usage Evidence
Policy Artifact
Identity Descriptor
```

## 22.3 Outputs

```text
Settlement Receipt Artifact
Contribution Record Artifact
Audit Event
SettlementFailed Event
```

## 22.4 Requirements

Settlement CSU:

```text
MUST NOT expose prompt content;
MUST NOT expose private result payload;
MUST record verifiable references, hashes and receipts;
MUST support own-resource execution without forced redistribution;
MAY support cold-start contribution policy;
MAY use blockchain, append-only log, federation ledger or local audit store.
```

Early Neurocoin and Blockchain documents are normalized into Settlement/Audit CSU and Contribution Credit rather than Core mechanisms.  

---

# 23. Optimization CSU Contract

## 23.1 Purpose

Optimization CSU reduces cost, latency, risk or uncertainty without changing semantics.

## 23.2 Inputs

```text
Context Artifact
Evidence Artifact
Capability Candidates
Cost Model Artifact
Policy Artifact
Historical Events
```

## 23.3 Outputs

```text
Optimization Recommendation Artifact
Cost Reduction Evidence Artifact
Route Hint Artifact
Placement Hint Artifact
```

## 23.4 Requirements

Optimization CSU:

```text
MUST NOT change semantic meaning of Result;
MUST NOT override Policy;
MUST NOT be required for Reference correctness;
MUST expose expected benefit;
MUST emit evidence of optimization outcome;
MUST be replaceable.
```

Optimization components are not part of system logic; they change only the cost of obtaining a result, not result semantics. 

---

# 24. PHM CSU Contract

## 24.1 Purpose

PHM CSU provides health, degradation and risk evidence.

## 24.2 Inputs

```text
Telemetry Events
Failure Events
Execution History
Hardware Health Evidence
Behavioral Reliability Evidence
```

## 24.3 Outputs

```text
Health Evidence Artifact
Risk Assessment Artifact
Capability Risk Metadata
PHM Event
```

## 24.4 Requirements

PHM CSU:

```text
MUST provide evidence, not routing decisions;
MUST distinguish behavior from hardware health;
MUST preserve uncertainty;
MUST expose scope and confidence;
MUST NOT punish node owner without evidence.
```

This preserves the later distinction between unreliable behavior and physical degradation from Resource Health / PHM work.  

---

# 25. Evolution CSU Contract

## 25.1 Purpose

Evolution CSU operates in Background/Evolution Plane.

It improves future resolution but does not affect current Operational Plane directly.

## 25.2 Inputs

```text
Evidence Artifact
Failure Artifact
Historical Events
Research Artifact
Hypothesis Artifact
```

## 25.3 Outputs

```text
Aggregated Evidence Artifact
Hypothesis Artifact
Validated Knowledge Artifact
Artifact Promotion Candidate
Negative Result Artifact
Open Research Artifact
```

## 25.4 Requirements

Evolution CSU:

```text
MUST NOT create Operational Artifact directly;
MUST pass through validation and promotion;
MUST preserve evidence chain;
MUST support negative results;
MUST support long-running exploratory work;
MUST operate under explicit budgets and policies.
```

Knowledge Lifecycle and Artifact Lifecycle are therefore implemented as Evolution CSU contracts rather than Core logic.  

---

# 26. Research CSU Contract

## 26.1 Purpose

Research CSU explores non-reference strategies, hypothesis generation, experimental algorithms and long-running cognitive evolution.

## 26.2 Inputs

```text
Open Research Artifact
Hypothesis Artifact
Evidence Artifact
Research Policy Artifact
Budget Artifact
```

## 26.3 Outputs

```text
Research Artifact
Best Current Hypothesis Artifact
Negative Result Artifact
Validated Candidate Artifact
```

## 26.4 Requirements

Research CSU:

```text
MUST be isolated from Operational Plane;
MUST NOT modify Core;
MUST NOT bypass Epistemic Validation;
MUST NOT promote artifacts directly into operational use;
MUST declare research scope and budget;
MUST preserve failed experiments as Evidence.
```

---

# 27. Human Interaction CSU Contract

## 27.1 Purpose

Human Interaction CSU handles cases where AIRA produces multiple epistemically equivalent solution regions.

## 27.2 Inputs

```text
Differentiated Solution Field Artifact
Alternative Solution Artifacts
Risk/Consequence Artifacts
Policy Artifact
```

## 27.3 Outputs

```text
Human Choice Artifact
Decision Context Artifact
Collapse Event
```

## 27.4 Requirements

Human Interaction CSU:

```text
MUST present alternatives without hidden normative collapse;
MUST expose assumptions, risks, consequences;
MUST preserve user decision provenance;
MUST NOT silently choose among equal normative alternatives;
MUST support deferral if user does not choose.
```

This implements Human Final Collapse from Book 0.

---

# 28. Failure Semantics

Any CSU failure **MUST** create:

```text
Failure Event
```

and SHOULD create:

```text
Failure Evidence Artifact
```

Failure types:

```text
InputArtifactInvalid
PolicyDenied
CapabilityUnavailable
ExecutionFailed
VerificationFailed
Timeout
BudgetExceeded
InvariantViolation
CSUInternalError
DependencyUnavailable
```

Failure is not empty output. Failure is evidence for Evolution Plane.

---

# 29. CSU Compatibility

Two CSU implementations are compatible if they:

```text
support the same CSU role contract;
support compatible ABI version;
accept compatible input artifact schemas;
produce compatible output artifact schemas;
emit required events;
preserve policy and provenance requirements;
handle failures according to this Book.
```

They do not need to use the same algorithm.

---

# 30. CSU Conformance Levels

## Level C0 — Manifest Compatible

CSU provides valid manifest, identity, signature, ABI version and declared permissions.

## Level C1 — Event Compatible

CSU can subscribe to and emit compliant Events.

## Level C2 — Artifact Compatible

CSU can consume and publish compliant Artifacts.

## Level C3 — Policy Compatible

CSU enforces Policy Gate before controlled actions.

## Level C4 — Role Compatible

CSU fully implements at least one standard role contract.

## Level C5 — Reference Compatible

CSU passes conformance tests for Book I–III.

---

# 31. CSU Test Requirements

A conformant CSU **MUST** pass tests for:

```text
manifest schema validation;
signature verification;
policy enforcement;
event idempotency;
artifact immutability;
provenance preservation;
failure emission;
permission isolation;
ABI compatibility;
role-specific input/output contracts.
```

---

# 32. Versioning

CSU versioning is independent from Core ABI versioning.

```text
CSU implementation version
CSU manifest version
CSU role contract version
Artifact schema version
Event schema version
ABI version
```

A CSU update **MUST NOT** require Core changes unless the relevant ABI major version changes.

---

# 33. Security Requirements

CSU **MUST** operate under least privilege.

CSU **MUST NOT** access:

```text
private user data without policy;
foreign CSU state;
raw prompts without authorization;
raw result payloads without authorization;
secret keys;
Core internals;
unscoped artifacts.
```

CSU **SHOULD** support sandboxing and revocation.

---

# 34. Privacy Requirements

CSU **MUST** minimize disclosure.

CSU **MUST NOT** emit raw private data into Events.

If private payloads must be used, they **MUST** be represented as protected Artifacts with explicit Policy.

---

# 35. Book III Compliance

An implementation is Book III compliant if it:

```text
supports CSU manifest registration;
supports CSU lifecycle states;
supports capability declaration;
supports event-based interaction;
supports artifact-based exchange;
enforces policy checks;
isolates CSU execution;
preserves failure evidence;
implements at least one standard CSU role contract;
passes role-specific conformance tests.
```

---

# 36. Book III Exclusions

Book III explicitly excludes:

```text
exact Context interpretation algorithm;
exact Evidence scoring method;
exact Verification algorithm;
exact PHM model;
exact Routing optimizer;
exact Settlement ledger;
exact Artifact storage backend;
exact Research method;
exact ML model;
exact human UI.
```

---

# 37. Canonical CSU Set

Book III v0.1 defines the following canonical CSU roles:

```text
AIRA-CSU-CTX   Context CSU
AIRA-CSU-RED   Reduction CSU
AIRA-CSU-EVD   Evidence CSU
AIRA-CSU-EPI   Epistemic CSU
AIRA-CSU-EXE   Execution CSU
AIRA-CSU-VER   Verification CSU
AIRA-CSU-ART   Artifact CSU
AIRA-CSU-DIS   Discovery CSU
AIRA-CSU-FED   Federation CSU
AIRA-CSU-SET   Settlement CSU
AIRA-CSU-OPT   Optimization CSU
AIRA-CSU-PHM   PHM CSU
AIRA-CSU-EVO   Evolution CSU
AIRA-CSU-RES   Research CSU
AIRA-CSU-HUM   Human Interaction CSU
```

---

# 38. Normative Formula

```text
CSU =
    Identity
  + Manifest
  + Capability Declaration
  + Event Contract
  + Artifact Contract
  + Policy Compliance
  + Lifecycle
  + Failure Semantics
```

A CSU is valid only if it can participate in AIRA without modifying Core and without direct dependency on another CSU implementation.

---

# 39. Status

Book III v0.1 formalizes the CSU layer of AIRA.

It defines:

```text
CSU manifest;
CSU lifecycle;
CSU capability contracts;
CSU event contracts;
CSU artifact contracts;
standard CSU roles;
failure semantics;
conformance levels;
security and privacy requirements.
```

Book III completes the normative foundation required for independent compatible implementations.

Наступний нормативний документ:

```text
Book IV — Reference Implementation v0.1
```

Scope of Book IV:

```text
minimal runnable AIRA node;
local Core implementation;
reference Event Runtime;
reference Artifact Store;
reference CSU registry;
reference Context CSU;
reference Execution CSU;
reference Verification CSU;
reference CLI/API;
test suite;
conformance harness.
```

Book IV may describe algorithms and implementation choices, but only as **reference**, not as protocol requirements.
