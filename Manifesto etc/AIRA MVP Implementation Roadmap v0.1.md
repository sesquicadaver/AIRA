# AIRA MVP Implementation Roadmap v0.1

```text
Type: Reference / Planning
Scope: AIRA MVP Implementation
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - Book IV — Reference Implementation
  - AIRA Schema Pack
  - AIRA Conformance Test Specification
Exports:
  - MVP Scope
  - Implementation Milestones
  - Repository Plan
  - C0 → C1 → C2 Profile Path
  - Minimal CSU Set
  - Test Harness Plan
  - Definition of Done
```

AIRA MVP має довести не “AI marketplace”, не “GPU network” і не “LLM runtime”, а мінімальну працездатність головної формули:

```text
Problem Statement
↓
Progressive Resolution
↓
Verified Result Artifact
↓
Evidence
↓
Lower-cost future Resolution
```

Це прямо випливає з Book 0, де ціль системи зафіксована як перетворення `Problem Statement` на `Verified Result Artifact`, а не максимізація використання обчислень. 

---

# 1. MVP Mission

MVP повинен довести, що AIRA може працювати як:

```text
local
minimal
deterministic
event-native
artifact-based
CSU-driven
schema-validatable
conformance-testable
```

система, яка не потребує:

```text
ML
GPU
blockchain
federation
distributed execution
global registry
PHM
research plane
optimizer
```

Book IV уже фіксує, що Reference Implementation має довести працездатність AIRA без ML, GPU, blockchain, federation і distributed execution. 

---

# 2. MVP Non-Goals

MVP **не повинен** реалізовувати:

```text
distributed network
real federation
real CRP across nodes
real settlement ledger
real PHM
model hosting
LLM inference
background research
self-optimization
knowledge field
multi-node execution
production security hardening
```

Ці речі або належать до пізніших профілів, або до Book V Research Track. Book V прямо визначає Research Track як необов’язковий для сумісної реалізації. 

---

# 3. Target Conformance

MVP повинен пройти шлях:

```text
C0 → C1 → partial C2
```

## 3.1 C0 — Local Core

Обов’язково:

```text
Object Runtime
Handle Model
Event Runtime
Policy Gate
Invariant Checker
Artifact Runtime
Security Boundary
```

## 3.2 C1 — Local Operational Node

Додає:

```text
Context CSU
Reduction CSU
Execution CSU
Verification CSU
Evidence CSU
Verified Result Artifact
```

## 3.3 Partial C2 — Local Protocol Compatibility

Додає локальні реалізації:

```text
Event Protocol
Artifact Protocol
Identity Protocol
Discovery Protocol
```

Повний C2 із мережевою взаємодією не входить у MVP.

Conformance Specification уже визначає профілі C0–C5 і відповідні групи тестів. 

---

# 4. MVP Architecture

```text
aira-node
 ├── aira-core
 │   ├── object-runtime
 │   ├── handle-registry
 │   ├── event-runtime
 │   ├── policy-gate
 │   ├── invariant-checker
 │   ├── artifact-runtime
 │   └── csu-runtime
 │
 ├── aira-schema
 │   ├── core-object.schema.json
 │   ├── artifact.schema.json
 │   ├── event.schema.json
 │   ├── policy.schema.json
 │   ├── capability.schema.json
 │   ├── csu-manifest.schema.json
 │   └── verified-result.schema.json
 │
 ├── csu
 │   ├── context-basic
 │   ├── reduction-basic
 │   ├── execution-basic
 │   ├── verification-basic
 │   ├── evidence-basic
 │   └── artifact-basic
 │
 ├── api
 │   ├── local-http-api
 │   └── cli
 │
 ├── storage
 │   ├── sqlite-object-store
 │   ├── sqlite-event-log
 │   └── filesystem-artifact-store
 │
 └── tests
     ├── conformance
     ├── integration
     └── fixtures
```

Book I визначає ядро як `Object Model + Stable ABI + Event Runtime + Policy Gate + Invariant Checker + Security Boundary + CSU Runtime`, а Book IV дає аналогічну структуру reference node.  

---

# 5. Recommended Technology Stack

## 5.1 Core

```text
Language: Rust
Async runtime: tokio
Storage: SQLite
Artifact store: local content-addressed filesystem
Hashing: SHA-256
Signing: Ed25519
Schemas: JSON Schema 2020-12
CLI: Rust clap
HTTP API: axum
```

## 5.2 Why Rust

```text
memory safety
strict type system
good async support
good CLI/server tooling
strong binary distribution story
future WASM compatibility
```

Book IV already recommends Rust for Core Runtime and allows CLI/Test tooling around it. 

---

# 6. Repository Layout

```text
aira/
 ├── Cargo.toml
 ├── README.md
 ├── LICENSE
 ├── specs/
 │   ├── book-0.md
 │   ├── book-i.md
 │   ├── book-ii.md
 │   ├── book-iii.md
 │   ├── book-iv.md
 │   ├── book-v.md
 │   ├── schema-pack.md
 │   ├── conformance.md
 │   └── rfc-process.md
 │
 ├── crates/
 │   ├── aira-core/
 │   ├── aira-object/
 │   ├── aira-event/
 │   ├── aira-artifact/
 │   ├── aira-policy/
 │   ├── aira-csu/
 │   ├── aira-schema/
 │   ├── aira-conformance/
 │   ├── aira-node/
 │   └── aira-cli/
 │
 ├── csu/
 │   ├── context-basic/
 │   ├── reduction-basic/
 │   ├── execution-basic/
 │   ├── verification-basic/
 │   ├── evidence-basic/
 │   └── artifact-basic/
 │
 ├── schemas/
 │   ├── common/
 │   ├── core/
 │   ├── event/
 │   ├── artifact/
 │   ├── policy/
 │   ├── capability/
 │   ├── csu/
 │   ├── result/
 │   └── conformance/
 │
 ├── fixtures/
 │   ├── valid/
 │   ├── invalid/
 │   └── flows/
 │
 └── tests/
     ├── unit/
     ├── integration/
     └── conformance/
```

---

# 7. Milestone M0 — Specification Freeze Snapshot

## Goal

Підготувати чистий стартовий корпус для реалізації.

## Deliverables

```text
Book 0–V у specs/
Schema Pack у schemas/
Conformance Specification у tests/conformance/spec/
RFC Process у specs/
README з поточною місією AIRA
```

## Exit Criteria

```text
усі активні специфікації присутні;
застарілі early docs не використовуються як implementation source;
усі references ведуть на Book 0–V / Schema / Conformance;
є один canonical README.
```

---

# 8. Milestone M1 — Schema Registry

## Goal

Реалізувати машинну валідацію базових контрактів.

## Required Schemas

```text
Core Object Descriptor
Event Descriptor
Artifact Descriptor
Policy Query
Policy Decision
Capability Descriptor
CSU Manifest
Execution Capsule
Evidence Artifact
Verified Result Artifact
Conformance Report
```

Schema Pack визначає ці структури як мінімальний data contract layer. 

## Deliverables

```text
schemas/*.schema.json
aira-schema crate
schema validation CLI
valid/invalid fixtures
```

## CLI

```bash
aira schema validate --schema verified-result --file result.json
aira schema list
```

## Exit Criteria

```text
valid fixtures pass;
invalid fixtures fail;
hash/signature fields structurally validated;
schema IDs stable;
no forbidden gpu/node/driver/scheduler/blockchain coupling.
```

---

# 9. Milestone M2 — Core Object Runtime

## Goal

Реалізувати immutable Core Object Model.

## Components

```text
ObjectDescriptor
ObjectStore
HandleRegistry
ContentHash
ProducerIdentity
ProvenanceRefs
PolicyRefs
```

## Required Behavior

```text
create object;
open object by handle;
read object descriptor;
reject mutation;
preserve provenance;
validate schema version;
return opaque handle.
```

## Exit Criteria

```text
object immutability test passes;
handle opacity test passes;
object descriptor schema passes;
attempted mutation emits InvariantViolation.
```

---

# 10. Milestone M3 — Artifact Runtime

## Goal

Реалізувати immutable content-addressed Artifact Store.

## Components

```text
ArtifactDescriptor
ArtifactStore
ContentAddress
ArtifactResolver
ArtifactPublisher
Supersession metadata
```

## Storage

```text
.artifacts/
  sha256/
    ab/
      cd/
        <hash>.artifact
```

## Required Behavior

```text
publish artifact;
resolve artifact;
validate hash;
validate descriptor;
preserve dependency_refs;
preserve provenance_refs;
support supersession without mutation.
```

## Exit Criteria

```text
artifact hash validation passes;
mutation attempt fails;
supersession creates new artifact;
artifact descriptor schema passes.
```

Artifact immutability and provenance are central both in Book I and Schema Pack.  

---

# 11. Milestone M4 — Event Runtime

## Goal

Реалізувати local append-only Event Runtime.

## Components

```text
EventDescriptor
EventLog
EventPublisher
EventSubscriber
CausalRefs
IdempotencyKey
```

## Required Behavior

```text
append event;
validate event schema;
preserve causal_refs;
notify subscribers;
reject unsigned invalid events;
avoid duplicate semantic effects;
query events by object_ref / artifact_ref.
```

## Exit Criteria

```text
event causality chain passes;
duplicate delivery is idempotent;
event descriptor schema passes;
invalid signature event rejected;
no global total ordering required.
```

Book II requires event causality preservation and no mandatory global total order. 

---

# 12. Milestone M5 — Policy Gate

## Goal

Реалізувати мінімальний Policy Gate.

## Decision Set

```text
ALLOW
DENY
REQUIRE
```

## Default Policy

```text
private artifacts: DENY unless explicitly allowed
local public artifacts: ALLOW
external export: REQUIRE
unknown action: DENY
```

## Required Behavior

```text
policy_check(query);
emit PolicyEvaluated event;
deny unauthorized artifact access;
block controlled action without policy check.
```

## Exit Criteria

```text
Policy Query schema passes;
Policy Decision schema passes;
Policy bypass emits InvariantViolation;
DENY is treated as valid outcome, not system failure.
```

---

# 13. Milestone M6 — Invariant Checker

## Goal

Реалізувати базовий набір інваріантів Book 0–I.

## Invariants

```text
object immutability;
artifact immutability;
event signature;
event causality;
policy-before-action;
CSU isolation;
no direct CSU-to-CSU mutation;
Verified Result Artifact completeness;
failure-to-evidence behavior;
Operational/Evolution separation stub.
```

## Exit Criteria

```text
all C0 invariant tests pass;
InvariantViolation event generated when needed;
Invariant Checker can block operation.
```

---

# 14. Milestone M7 — CSU Runtime

## Goal

Реалізувати мінімальну систему реєстрації й запуску CSU.

## MVP Packaging

```text
in-process Rust trait implementation
```

## Future-compatible Packaging

```text
WASM
subprocess
container
```

## CSU Trait

```rust
pub trait Csu {
    fn manifest(&self) -> CsuManifest;

    fn on_event(
        &mut self,
        event: EventDescriptor,
        ctx: CsuExecutionContext,
    ) -> Result<Vec<CsuOutput>, CsuError>;
}
```

## Required Behavior

```text
register CSU manifest;
validate manifest schema;
validate signature;
check ABI version;
activate CSU;
dispatch events;
collect outputs;
emit failures.
```

## Exit Criteria

```text
valid CSU manifest accepted;
unsigned manifest rejected;
CSU cannot mutate Core directly;
CSU failure emits Failure Event.
```

Book III defines CSU as isolated, replaceable modules using ABI, Events, Artifacts and Policy. 

---

# 15. Milestone M8 — Minimal CSU Set

## 15.1 Context CSU

```text
Input:
  Problem Statement

Output:
  Context Artifact
  ContextResolved Event

Implementation:
  deterministic rule-based extraction
```

Must:

```text
detect language;
extract explicit constraints;
mark unresolved ambiguity;
not execute;
not produce result.
```

---

## 15.2 Reduction CSU

```text
Input:
  Problem Statement
  Context Artifact

Output:
  Ready Solution Artifact OR Negative Lookup Artifact OR Execution Capsule
```

Must:

```text
check local ready solutions;
check local knowledge artifacts;
prefer reuse before compute;
emit reason for escalation.
```

---

## 15.3 Execution CSU

```text
Input:
  Execution Capsule

Output:
  Output Artifact
  CapsuleCompleted Event OR CapsuleFailed Event
```

MVP execution kinds:

```text
math.eval.safe
text.echo
text.uppercase
json.identity
```

Must not:

```text
execute arbitrary shell;
use network by default;
access private artifacts without policy.
```

---

## 15.4 Verification CSU

```text
Input:
  Output Artifact
  Problem Statement
  Context Artifact

Output:
  Verified Result Artifact OR Rejected Output Artifact
```

MVP verification modes:

```text
schema validation
deterministic expected-output validation
manual accept
hash validation
```

Must distinguish:

```text
Output Artifact != Verified Result Artifact
```

---

## 15.5 Evidence CSU

```text
Input:
  ResultPublished Event
  VerificationFailed Event
  CapsuleFailed Event

Output:
  Evidence Artifact
  FailureEvidenceCreated Event
```

Must:

```text
convert failure into evidence;
preserve provenance;
not assign final truth.
```

---

## 15.6 Artifact CSU

```text
Input:
  Artifact descriptors / resolve requests

Output:
  ArtifactPublished
  ArtifactResolved
  ArtifactInvalid
  ArtifactSuperseded
```

Must:

```text
validate hash;
validate signature;
preserve provenance;
support supersession.
```

---

# 16. Milestone M9 — Local Operational Flow

## Goal

Зібрати перший end-to-end flow.

## Demo Input

```text
Problem Statement: "Calculate 2 + 2"
```

## Expected Flow

```text
ProblemSubmitted
↓
ContextResolved
↓
ReductionCompleted
↓
CapsuleCreated
↓
CapsuleCompleted
↓
VerificationCompleted
↓
ResultPublished
↓
EvidenceCreated
```

## Expected Output

```text
Verified Result Artifact:
  result = 4
  verification_status = VERIFIED
  confidence = 1.0
  scope = local_reference
```

## Exit Criteria

```text
full flow reproducible;
all objects/artifacts/events are signed;
all descriptors schema-valid;
event chain queryable;
result has evidence_refs and provenance_refs.
```

This directly proves the Book 0 operational goal and Book IV reference claim.  

---

# 17. Milestone M10 — CLI

## Required Commands

```bash
aira init
aira identity create
aira schema validate
aira csu list
aira csu register <manifest>
aira problem submit --text "..."
aira problem status <problem_id>
aira result get <result_id>
aira artifact get <artifact_ref>
aira event tail
aira conformance run --profile C0
aira conformance run --profile C1
```

## Exit Criteria

```text
clean local init works;
identity created;
CSU registered;
problem submitted;
result retrieved;
events visible;
conformance runnable.
```

---

# 18. Milestone M11 — Local HTTP API

## Required Endpoints

```text
POST /v1/problems
GET  /v1/problems/{id}
GET  /v1/results/{id}
GET  /v1/artifacts/{id}
GET  /v1/events
GET  /v1/capabilities
GET  /v1/csu
POST /v1/csu/register
POST /v1/conformance/run
```

## Non-Goals

```text
authentication hardening
multi-tenant deployment
public network exposure
federation API
production authorization model
```

---

# 19. Milestone M12 — Conformance Harness

## Goal

Реалізувати автоматизовану перевірку C0 і C1.

## Initial Test Groups

```text
B0 ontology tests
B1 object / ABI tests
artifact immutability tests
event causality tests
policy gate tests
CSU manifest tests
minimal operational pipeline tests
failure-to-evidence tests
Verified Result Artifact tests
```

## Required Fixtures

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
```

## Exit Criteria

```text
aira conformance run --profile C0 passes;
aira conformance run --profile C1 passes;
Conformance Report Artifact created;
failures have evidence_refs.
```

Conformance Specification already defines these exact validation targets. 

---

# 20. Milestone M13 — Partial C2 Local Protocols

## Goal

Підготувати основу для protocol compatibility без реальної мережі.

## Implement Locally

```text
AIRA-EP   Event Protocol
AIRA-AP   Artifact Protocol
AIRA-ID   Identity Protocol
AIRA-DP   Discovery Protocol
```

## Scope

```text
local protocol envelope;
local protocol response;
identity descriptor;
local discovery registry;
capability advertisement.
```

## Exit Criteria

```text
Protocol Envelope schema passes;
Protocol Response schema passes;
Identity Descriptor schema passes;
local Discovery returns Capability, not Node;
unsupported version returns UNSUPPORTED_VERSION without side effects.
```

Book II defines protocol contracts and explicitly keeps algorithms out of protocol requirements. 

---

# 21. MVP Build Order

Оптимальний порядок реалізації:

```text
1. specs snapshot
2. schema registry
3. object runtime
4. artifact runtime
5. event runtime
6. policy gate
7. invariant checker
8. CSU runtime
9. basic CSU set
10. end-to-end local flow
11. CLI
12. HTTP API
13. conformance harness
14. partial C2 local protocols
```

Не починати з API, federation, network або distributed execution.

Причина: MVP має спочатку довести ядро `Problem → Verified Result`, а не мережеву інфраструктуру.

---

# 22. Data Model Minimum

MVP storage tables:

```text
objects
artifacts
events
identities
policies
csu_manifests
capabilities
conformance_reports
```

## objects

```text
object_id
object_type
schema_version
content_hash
descriptor_json
created_at
signature
```

## artifacts

```text
artifact_id
artifact_type
content_hash
content_ref
descriptor_json
created_at
signature
```

## events

```text
event_id
event_type
causal_refs_json
object_refs_json
artifact_refs_json
descriptor_json
created_at
signature
```

---

# 23. Security Baseline

MVP defaults:

```text
network disabled for CSU;
shell execution disabled;
unsigned CSU rejected;
unsigned Artifact rejected;
unknown ABI rejected;
private Artifact denied by default;
Policy Gate required for controlled actions;
secret material never emitted into Events.
```

MVP security is not production security, but it must preserve the architectural invariants.

---

# 24. Implementation Constraints

MVP **MUST NOT**:

```text
add new Core entities;
introduce Node as required ontology;
introduce Driver as canonical role;
introduce Scheduler as required component;
require GPU;
require LLM;
require blockchain;
require distributed execution;
mutate Artifacts;
hide failures;
skip Policy Gate;
allow direct CSU-to-CSU mutation.
```

These constraints protect the project from regression into the early P2P LLM/GPU-centric architecture. The early project started as decentralized CPU/GPU sharing for LLM inference, but the current canonical architecture is broader and centered on Problem Statement → Verified Result Artifact.  

---

# 25. First Demo Scenario

## Scenario

```text
User submits:
  "Calculate 2 + 2"
```

## Expected Behavior

```text
AIRA creates Problem Statement.
Context CSU creates Context Artifact.
Reduction CSU finds no Ready Solution.
Reduction CSU creates Execution Capsule.
Execution CSU runs math.eval.safe.
Execution CSU publishes Output Artifact.
Verification CSU validates output.
Verification CSU publishes Verified Result Artifact.
Evidence CSU creates Evidence Artifact.
Conformance Harness validates the flow.
```

## Expected Proof

```text
Every step is an Event.
Every durable product is an Artifact.
Every Artifact is immutable.
Every Result has Evidence.
Every failure path is observable.
```

---

# 26. Second Demo Scenario

## Scenario

```text
User submits:
  "Return the same text: hello"
```

## Goal

Перевірити не обчислення, а pipeline.

## Expected Behavior

```text
Problem Statement
↓
Context
↓
Execution Capsule
↓
Output Artifact
↓
Verified Result Artifact
```

This validates that AIRA can solve trivial tasks without unnecessary escalation.

---

# 27. Third Demo Scenario — Failure to Evidence

## Scenario

```text
Execution Capsule references missing input Artifact.
```

## Expected Behavior

```text
Artifact resolve fails.
Execution CSU emits CapsuleFailed.
Failure Event is created.
Evidence CSU creates Failure Evidence Artifact.
No Verified Result Artifact is published.
```

This proves the rule:

```text
Failure is Evidence.
```

---

# 28. Fourth Demo Scenario — Ready Solution Reuse

## Setup

Local Artifact Store already contains:

```text
Ready Solution Artifact:
  Problem: "Calculate 2 + 2"
  Result: 4
```

## Expected Behavior

```text
Reduction CSU finds Ready Solution.
Execution CSU is not invoked.
Verification CSU verifies ready solution.
ResultPublished emitted.
```

This proves:

```text
Reuse before compute.
```

---

# 29. Fifth Demo Scenario — Normative Split Stub

## Scenario

```text
Problem Statement contains two equivalent normative alternatives.
```

## MVP Behavior

MVP may not implement full ethical reasoning, but must support the artifact shape:

```text
Differentiated Solution Field Artifact
requires_human_collapse = true
```

It must not silently choose one alternative.

This preserves Human Final Collapse from Book 0. 

---

# 30. MVP Definition of Done

MVP is complete when:

```text
1. Clean checkout builds successfully.

2. `aira init` creates local node state.

3. Identity can be created.

4. Core schemas validate.

5. CSU manifests validate.

6. Basic CSU set registers.

7. User can submit Problem Statement.

8. System produces Context Artifact.

9. System reuses Ready Solution if available.

10. System creates Execution Capsule if needed.

11. Execution CSU produces Output Artifact.

12. Verification CSU produces Verified Result Artifact.

13. Evidence CSU produces Evidence Artifact.

14. All Events are persisted and queryable.

15. All Artifacts are immutable and content-addressed.

16. Policy Gate is enforced.

17. Failure emits Event.

18. Failure creates Evidence Artifact.

19. C0 conformance passes.

20. C1 conformance passes.

21. Demo scenarios pass.

22. No ML/GPU/blockchain/network dependency is required.
```

---

# 31. MVP Risk Register

## R1 — Scope Creep

Risk:

```text
спроба додати federation, network, GPU, LLM або optimizer до MVP.
```

Mitigation:

```text
MVP target = C0 + C1 + partial local C2 only.
```

---

## R2 — Core Pollution

Risk:

```text
додавання domain-specific logic у Core.
```

Mitigation:

```text
усе domain-specific — тільки CSU.
```

---

## R3 — Schema Drift

Risk:

```text
реалізація почне відрізнятися від Schema Pack.
```

Mitigation:

```text
schema validation в CI;
fixtures first;
schema-driven tests.
```

---

## R4 — Hidden Scheduler Regression

Risk:

```text
Reduction CSU перетвориться на новий central scheduler.
```

Mitigation:

```text
Reduction CSU тільки рекомендує / ескалує;
Execution binding через Capability / Capsule;
Optimization optional.
```

---

## R5 — Artifact Mutation

Risk:

```text
для зручності реалізація почне оновлювати artifact in-place.
```

Mitigation:

```text
content-addressed store;
mutation tests;
supersession only.
```

---

## R6 — Conformance Afterthought

Risk:

```text
тести будуть додані після реалізації.
```

Mitigation:

```text
Conformance fixtures before feature completion.
```

---

# 32. CI Pipeline

```text
format
↓
lint
↓
unit tests
↓
schema validation
↓
fixture validation
↓
integration flow tests
↓
conformance C0
↓
conformance C1
↓
artifact immutability tests
↓
security baseline tests
```

Recommended commands:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
aira schema validate --fixtures fixtures/valid
aira conformance run --profile C0
aira conformance run --profile C1
```

---

# 33. Branching Model

```text
main        — stable passing conformance
develop     — integration branch
feature/*   — implementation branches
rfc/*       — changes tied to accepted RFC
experiment/* — non-reference experiments
```

Rules:

```text
main must pass C0/C1;
feature branches must include tests;
experiment branches must not affect conformance;
schema changes require RFC-S or explicit draft marker.
```

---

# 34. MVP Release Artifact Set

MVP release must produce:

```text
aira-node binary
aira CLI
schema pack files
basic CSU manifests
conformance fixtures
demo flows
README
developer guide
conformance report
release notes
```

Release tag:

```text
aira-mvp-v0.1.0
```

---

# 35. Documentation Minimum

Required docs:

```text
README.md
docs/architecture.md
docs/local-node.md
docs/csu-development.md
docs/artifact-store.md
docs/event-runtime.md
docs/conformance.md
docs/demo.md
```

README must state:

```text
AIRA MVP is not an AI runtime.
AIRA MVP is not a GPU scheduler.
AIRA MVP is not a blockchain network.
AIRA MVP is a minimal local proof of Problem Statement → Verified Result Artifact.
```

---

# 36. Post-MVP Path

After MVP:

```text
MVP v0.1  → C0/C1 local
v0.2      → full C2 local protocols
v0.3      → C3 federation prototype
v0.4      → CRP multi-node prototype
v0.5      → settlement/audit prototype
v0.6      → research/evolution sandbox
```

No step should expand Core unless a valid RFC-B passes Core Change Gate.

---

# 37. Canonical MVP Formula

```text
MVP =
    Book 0 semantics
  + Book I local Core
  + Book II local protocol subset
  + Book III basic CSU contracts
  + Schema Pack validation
  + C0/C1 Conformance
  + deterministic demo flows
```

Not:

```text
MVP ≠ distributed AI network
MVP ≠ GPU marketplace
MVP ≠ LLM platform
MVP ≠ blockchain settlement system
MVP ≠ research cognition engine
```

---

# 38. Status

AIRA MVP Implementation Roadmap v0.1 defines the practical implementation path from specification to first runnable system.

It preserves the project goal:

```text
найкраще вирішення задачі мінімально необхідними ресурсами
```

and implements it through:

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

Наступний нормативний документ:

```text
AIRA Repository Bootstrap Plan v0.1
```

Scope:

```text
initial repo creation;
Cargo workspace;
schema directory;
first crates;
first fixtures;
first CLI commands;
first end-to-end test;
initial CI;
coding standards;
commit conventions;
MVP issue breakdown.
```
