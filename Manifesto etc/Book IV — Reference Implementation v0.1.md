# Book IV — Reference Implementation v0.1

```text
Type: Reference / Non-Normative
Scope: AIRA Reference Implementation
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
Exports:
  - Minimal Runnable AIRA Node
  - Reference Core Runtime
  - Reference Event Runtime
  - Reference Artifact Store
  - Reference CSU Registry
  - Reference CSU Set
  - Local API / CLI
  - Conformance Test Harness
```

Book IV описує **референсну реалізацію AIRA**, а не новий стандарт. Нормативними залишаються Book 0–III: Book 0 фіксує фундамент і місію `Problem Statement → Verified Result Artifact`, Book I визначає Core/ABI, Book II — протоколи, Book III — CSU-контракти.    

---

# 1. Purpose

Reference Implementation має довести, що AIRA можна реалізувати мінімально, без:

```text
ML
GPU
Blockchain
Federation
PHM
Routing Optimizer
Distributed Execution
Research Plane
```

Мінімальна AIRA повинна працювати локально:

```text
Problem Statement
        ↓
Context CSU
        ↓
Reduction CSU
        ↓
Execution CSU
        ↓
Verification CSU
        ↓
Verified Result Artifact
```

---

# 2. Non-Goals

Reference Implementation **не є**:

```text
production-grade distributed network
cloud platform
model hosting system
GPU scheduler
blockchain network
AI agent framework
research-grade optimizer
```

Вона демонструє сумісність із Book 0–III і створює базу для незалежних реалізацій.

---

# 3. Reference Architecture

```text
aira-node
 ├── core-runtime
 │   ├── object-runtime
 │   ├── handle-registry
 │   ├── event-runtime
 │   ├── policy-gate
 │   ├── invariant-checker
 │   ├── artifact-runtime
 │   └── csu-runtime
 │
 ├── protocols
 │   ├── event-protocol
 │   ├── artifact-protocol
 │   ├── identity-protocol
 │   ├── discovery-protocol
 │   └── local-crp
 │
 ├── csu
 │   ├── context-csu
 │   ├── reduction-csu
 │   ├── evidence-csu
 │   ├── epistemic-csu
 │   ├── execution-csu
 │   ├── verification-csu
 │   └── artifact-csu
 │
 ├── storage
 │   ├── object-store
 │   ├── artifact-store
 │   ├── event-log
 │   └── identity-store
 │
 ├── api
 │   ├── local-http-api
 │   └── cli
 │
 └── tests
     ├── conformance
     ├── integration
     └── fixtures
```

---

# 4. Implementation Language

Reference Implementation **SHOULD** use a memory-safe systems language.

Recommended:

```text
Rust
```

Acceptable alternatives:

```text
Go
TypeScript / Node.js
Python for prototype only
```

Canonical recommendation:

```text
Core Runtime: Rust
CLI: Rust or TypeScript
Test harness: Python or Rust
```

Rationale:

```text
memory safety
static typing
good async runtime
WASM support
FFI compatibility
strong tooling
```

---

# 5. Process Model

Minimal implementation може працювати як один процес:

```text
aira-node
```

але логічно всі компоненти залишаються ізольованими.

```text
Single process
  ≠
Single responsibility
```

Навіть якщо CSU реалізовані як in-process plugins, вони повинні взаємодіяти лише через Core ABI abstraction.

---

# 6. Storage Model

## 6.1 Object Store

Зберігає Core Objects:

```text
Problem Statement
Context
Execution Intent
Execution Capsule
Capability
Policy
Event
Verified Result Artifact descriptor
```

Recommended implementation:

```text
SQLite
```

Alternative:

```text
RocksDB
PostgreSQL
sled
```

## 6.2 Artifact Store

Зберігає immutable artifact payloads.

Reference mode:

```text
content-addressed local filesystem
```

Path format:

```text
.artifacts/
  sha256/
    ab/
      cd/
        <hash>.artifact
```

## 6.3 Event Log

Append-only log.

Reference mode:

```text
SQLite table
```

Required properties:

```text
append-only
causal refs
idempotent event insertion
signature verification
query by object_ref
query by artifact_ref
```

---

# 7. Core Runtime

## 7.1 Object Runtime

Responsibilities:

```text
create object descriptors
validate schema
assign object_id
calculate content_hash
store immutable object
return opaque handle
```

Reference object descriptor:

```json
{
  "object_id": "obj_...",
  "object_type": "ProblemStatement",
  "schema_version": "0.1",
  "created_at": "2026-07-09T00:00:00Z",
  "producer_identity": "id_...",
  "policy_refs": [],
  "provenance_refs": [],
  "content_hash": "sha256:...",
  "signature": "sig_..."
}
```

## 7.2 Handle Registry

Reference handle format:

```text
aira:<type>:<hash-or-id>
```

Examples:

```text
aira:problem:01H...
aira:context:01H...
aira:artifact:sha256...
aira:event:01H...
aira:csu:ctx.basic
```

Handles are opaque to CSU.

---

# 8. Event Runtime

Reference Event Runtime uses pub/sub over local async channels.

Minimal internal model:

```text
append event
validate event
persist event
notify subscribers
```

Reference event descriptor:

```json
{
  "event_id": "evt_...",
  "event_type": "ProblemSubmitted",
  "schema_version": "0.1",
  "producer_identity": "id_...",
  "causal_refs": [],
  "object_refs": ["aira:problem:..."],
  "artifact_refs": [],
  "policy_refs": [],
  "payload_hash": "sha256:...",
  "signature": "sig_...",
  "created_at": "2026-07-09T00:00:00Z"
}
```

Required event properties:

```text
idempotent publish
causal_refs preservation
signature validation
append-only persistence
policy visibility
```

No global total ordering is required.

---

# 9. Policy Gate

Reference Policy Gate supports a minimal declarative policy model.

## 9.1 Default Policy

```text
deny by default for private data
allow by default for local public test artifacts
require explicit policy for external export
```

## 9.2 Policy Decision

```json
{
  "decision": "ALLOW",
  "requirements": [],
  "reason_refs": [],
  "signature": "sig_..."
}
```

Allowed values:

```text
ALLOW
DENY
REQUIRE
```

Policy Gate must never execute domain logic.

---

# 10. Invariant Checker

Reference Invariant Checker validates:

```text
object immutability
artifact hash integrity
event signature
policy decision presence
CSU isolation
no direct CSU-to-CSU mutation
Verified Result Artifact completeness
Operational/Evolution separation
```

Minimal failure event:

```json
{
  "event_type": "InvariantViolation",
  "object_refs": ["..."],
  "payload": {
    "invariant": "artifact_immutability",
    "reason": "attempted mutation"
  }
}
```

---

# 11. CSU Runtime

Reference CSU Runtime loads CSU from local manifests.

Supported CSU packaging v0.1:

```text
native binary
WASM module
in-process Rust trait implementation
external process over local IPC
```

Recommended for v0.1:

```text
in-process Rust trait
```

Future-compatible option:

```text
WASM sandbox
```

---

# 12. CSU Runtime Interface

Reference internal interface:

```rust
trait Csu {
    fn manifest(&self) -> CsuManifest;

    fn on_event(
        &mut self,
        event: EventDescriptor,
        ctx: CsuExecutionContext
    ) -> Result<Vec<CsuOutput>, CsuError>;
}
```

Where output can be:

```text
Event
Artifact
CapabilityAdvertisement
PolicyQuery
Failure
```

This is implementation detail. The normative contract remains Book III.

---

# 13. Minimal Reference CSU Set

## 13.1 Context CSU

Purpose:

```text
Problem Statement → Context Artifact
```

Reference implementation:

```text
rule-based context extraction
no ML required
```

Example:

```text
detect language
detect domain keywords
extract explicit constraints
mark unresolved ambiguity
```

Outputs:

```text
Context Artifact
ContextResolved Event
```

---

## 13.2 Reduction CSU

Purpose:

```text
Context + Problem → Solution Candidate / Escalation
```

Reference implementation:

```text
lookup local Ready Solution Artifacts
lookup local Knowledge Artifacts
fallback to Execution Capsule
```

No external search required in minimal version.

Outputs:

```text
ReadySolutionFound
NegativeLookupArtifact
ExecutionRequiredEvent
```

---

## 13.3 Execution CSU

Purpose:

```text
Execution Capsule → Output Artifact
```

Reference implementation supports three minimal execution types:

```text
echo
shell-disabled transform
deterministic local function
```

No arbitrary shell execution in default mode.

Example execution kinds:

```text
text.transform.uppercase
text.summarize.stub
math.eval.safe
```

---

## 13.4 Verification CSU

Purpose:

```text
Output Artifact → Verified Result Artifact
```

Reference verification modes:

```text
schema validation
hash validation
deterministic expected-output check
policy validation
manual acceptance
```

Verification CSU must distinguish:

```text
Output Artifact
≠
Verified Result Artifact
```

---

## 13.5 Artifact CSU

Purpose:

```text
publish
resolve
validate
supersede
```

Reference implementation:

```text
content-addressed filesystem
SQLite metadata index
SHA-256 hash
Ed25519 signature
```

---

## 13.6 Evidence CSU

Purpose:

```text
Events / Failures / Outputs → Evidence Artifact
```

Reference implementation:

```text
convert failure events to evidence
convert verification results to evidence
preserve provenance
```

---

## 13.7 Epistemic CSU

Purpose:

```text
Evidence + Claim → Epistemic Assessment
```

Reference implementation:

```text
simple rule-based mapping
```

Example:

```text
single unverified observation → Observation
verified local result → Protocol Fact
failed verification → Contradicted
manual assumption → Assumption
```

---

# 14. Local Protocol Implementations

Book IV implements local versions of Book II protocols.

## 14.1 Event Protocol

Reference:

```text
local event bus + persisted event log
```

Transport:

```text
in-process async channel
```

## 14.2 Artifact Protocol

Reference:

```text
local content-addressed artifact store
```

## 14.3 Identity Protocol

Reference:

```text
local Ed25519 keypair
local identity descriptor
signed events and artifacts
```

## 14.4 Discovery Protocol

Reference:

```text
local registry lookup
```

## 14.5 CRP

Reference:

```text
single-node capability binding
```

No distributed routing required.

## 14.6 Federation Protocol

Reference:

```text
not required for Level R0
stub implementation for conformance
```

## 14.7 Settlement Protocol

Reference:

```text
local signed receipt
no blockchain
```

---

# 15. Minimal Operational Flow

## 15.1 Submit Problem

```text
CLI/API receives Problem Statement
↓
Object Runtime creates Problem Object
↓
Event Runtime emits ProblemSubmitted
```

## 15.2 Interpret

```text
Context CSU observes ProblemSubmitted
↓
creates Context Artifact
↓
emits ContextResolved
```

## 15.3 Reduce

```text
Reduction CSU observes ContextResolved
↓
checks Ready Solution / Knowledge
↓
if found: emits CandidateResolved
↓
if not: creates Execution Capsule
```

## 15.4 Materialize

```text
Execution CSU observes CapsuleCreated
↓
runs allowed deterministic action
↓
publishes Output Artifact
↓
emits CapsuleCompleted
```

## 15.5 Verify

```text
Verification CSU observes CapsuleCompleted
↓
validates Output Artifact
↓
publishes Verified Result Artifact
↓
emits ResultPublished
```

## 15.6 Evolve

```text
Evidence CSU observes ResultPublished / Failure
↓
creates Evidence Artifact
↓
Epistemic CSU may update Knowledge Candidate
```

---

# 16. API Surface

## 16.1 Local HTTP API

Reference API endpoints:

```text
POST /v1/problems
GET  /v1/problems/{id}
GET  /v1/results/{id}
GET  /v1/artifacts/{id}
GET  /v1/events?object_ref=...
GET  /v1/capabilities
GET  /v1/csu
POST /v1/csu/register
```

## 16.2 Submit Problem

Request:

```json
{
  "problem_statement": "Summarize this document",
  "inputs": [
    {
      "artifact_ref": "aira:artifact:sha256..."
    }
  ],
  "constraints": {
    "max_cost": null,
    "privacy_class": "local"
  }
}
```

Response:

```json
{
  "problem_ref": "aira:problem:...",
  "status": "ACCEPTED"
}
```

## 16.3 Get Result

Response:

```json
{
  "result_ref": "aira:result:...",
  "verification_status": "VERIFIED",
  "confidence": 1.0,
  "scope": {
    "mode": "local_reference"
  },
  "artifact_ref": "aira:artifact:sha256..."
}
```

---

# 17. CLI

Reference CLI:

```text
aira init
aira identity create
aira csu list
aira csu register <manifest>
aira problem submit <file-or-text>
aira problem status <problem_id>
aira result get <result_id>
aira artifact get <artifact_ref>
aira event tail
aira conformance run
```

Example:

```bash
aira init
aira identity create --name local-user
aira csu register ./csu/context-basic/manifest.yaml
aira problem submit --text "Calculate 2 + 2"
aira event tail
```

---

# 18. Configuration

Reference config file:

```yaml
node:
  mode: local
  data_dir: .aira

identity:
  key_path: .aira/identity.ed25519

core:
  abi_version: "0.1"
  strict_invariants: true

storage:
  object_store: sqlite
  artifact_store: filesystem
  event_log: sqlite

security:
  sandbox: process
  allow_network: false
  allow_shell: false

csu:
  autoload:
    - context-basic
    - reduction-basic
    - execution-basic
    - verification-basic
    - artifact-basic
```

---

# 19. Security Defaults

Reference Implementation defaults:

```text
network disabled for CSU
shell execution disabled
filesystem access scoped
private artifacts policy-protected
unsigned CSU rejected
unsigned artifacts rejected
unknown ABI rejected
```

Any relaxation must be explicit config.

---

# 20. Reference Data Formats

Recommended serialization:

```text
JSON for v0.1
CBOR optional
YAML allowed for manifests only
```

Canonical hashing:

```text
stable canonical JSON → SHA-256
```

Canonical signing:

```text
Ed25519 over canonical descriptor hash
```

---

# 21. Testing Strategy

## 21.1 Unit Tests

```text
object immutability
handle resolution
event append
artifact hash
policy decision
signature verification
```

## 21.2 Integration Tests

```text
problem → context
context → capsule
capsule → output
output → verified result
failure → evidence
artifact supersession
```

## 21.3 Conformance Tests

```text
Book I object descriptor compatibility
Book I event descriptor compatibility
Book II envelope compatibility
Book III CSU manifest compatibility
Verified Result Artifact compatibility
```

---

# 22. Reference Conformance Harness

Command:

```bash
aira conformance run --profile local-core
```

Profiles:

```text
R0 minimal-local
R1 local-with-artifacts
R2 local-with-csu-registry
R3 protocol-compatible
R4 federation-ready
```

Output:

```json
{
  "profile": "R1",
  "passed": true,
  "tests": {
    "object_immutability": "passed",
    "event_causality": "passed",
    "artifact_hash": "passed",
    "verified_result_contract": "passed"
  }
}
```

---

# 23. Reference Profiles

## R0 — Minimal Local Core

Includes:

```text
Object Runtime
Event Runtime
Policy Gate
Invariant Checker
Artifact Runtime
manual CSU registration
```

## R1 — Minimal Operational Node

Adds:

```text
Context CSU
Reduction CSU
Execution CSU
Verification CSU
Evidence CSU
```

## R2 — Local Protocol Node

Adds:

```text
local Discovery
local CRP
Identity Protocol
Settlement receipt stub
```

## R3 — Federation-Capable Node

Adds:

```text
Federation descriptor
cross-node artifact exchange
cross-node event exchange
policy-scoped export/import
```

## R4 — Research-Capable Node

Adds:

```text
Evolution CSU
Research CSU
background experimentation
artifact promotion pipeline
```

R4 is not required for Reference compatibility.

---

# 24. Minimal MVP

The first runnable MVP should implement only:

```text
Book 0-compatible ontology
Book I local core
Book II local event/artifact/identity protocol
Book III basic CSU contracts
CLI
local artifact store
conformance harness
```

MVP must prove:

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
↓
Evidence Artifact
```

---

# 25. Suggested Repository Layout

```text
aira/
 ├── crates/
 │   ├── aira-core/
 │   ├── aira-abi/
 │   ├── aira-object/
 │   ├── aira-event/
 │   ├── aira-artifact/
 │   ├── aira-policy/
 │   ├── aira-csu/
 │   ├── aira-protocol/
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
 ├── tests/
 │   ├── conformance/
 │   ├── integration/
 │   └── fixtures/
 │
 ├── specs/
 │   ├── book-0/
 │   ├── book-i/
 │   ├── book-ii/
 │   ├── book-iii/
 │   └── book-iv/
 │
 └── examples/
     ├── local-problem/
     ├── artifact-flow/
     └── failure-to-evidence/
```

---

# 26. Minimal Rust Crates

```text
aira-core       — object runtime, handles, invariants
aira-abi        — ABI traits and stable descriptors
aira-event      — event descriptors, log, subscriptions
aira-artifact   — artifact descriptors, hashing, store
aira-policy     — policy gate and decisions
aira-csu        — CSU manifest, lifecycle, runtime
aira-protocol   — Book II envelopes and local protocols
aira-cli        — command-line interface
```

---

# 27. Implementation Rules

Reference Implementation **MUST**:

```text
not add new Core ontology;
not bypass ABI;
not mutate artifacts;
not hide failures;
not require ML;
not require GPU;
not require network;
not require blockchain;
not require federation;
not require distributed execution.
```

Reference Implementation **MAY**:

```text
provide simple algorithms;
provide local-only protocol adapters;
provide stub CSU;
provide optional federation mode;
provide optional research mode;
```

---

# 28. Book IV vs Book V

Book IV may contain implementation choices.

Book V contains speculative or experimental work.

If a feature requires:

```text
new mathematical model
new ontology
new optimizer theory
new research method
non-proven ML behavior
long-running cognitive evolution
```

it belongs to Book V, not Book IV.

---

# 29. Deliverables

Book IV v0.1 implementation should produce:

```text
1. local aira-node binary
2. aira CLI
3. local object store
4. local event log
5. local artifact store
6. CSU registry
7. basic CSU set
8. conformance harness
9. example flows
10. developer documentation
```

---

# 30. Reference Implementation Success Criteria

Book IV v0.1 is successful when:

```text
a clean machine can initialize AIRA;
a user can submit a Problem Statement;
the system creates Context;
the system creates or reuses a Solution path;
the system materializes output if needed;
the system verifies output;
the system publishes Verified Result Artifact;
all actions are traceable through Events;
all artifacts are immutable and signed;
failures produce Evidence;
conformance tests pass.
```

---

# 31. Status

Book IV v0.1 defines the minimal practical implementation path for AIRA.

It intentionally avoids production-grade complexity and proves only the core architectural claim:

> **AIRA can operate as a local, minimal, deterministic, event-native, artifact-based, CSU-driven system that transforms Problem Statement into Verified Result Artifact without requiring ML, GPU, blockchain, federation or distributed execution.**

Наступний нормативний документ:

```text
Book V — Research Track v0.1
```

Scope:

```text
Exploratory Resolution
Cognitive Evolution
Distributed Scientific Method
Self-Organizing Constellations
Capability Algebra
Advanced PHM
Knowledge Field
Goal Compiler
Collective Wisdom
Research CSU
Long-running background experiments
```
