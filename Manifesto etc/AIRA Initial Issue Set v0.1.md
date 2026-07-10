# AIRA Initial Issue Set v0.1

```text
Type: Reference / Planning / Engineering
Scope: AIRA MVP Issue Planning
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - AIRA Schema Pack
  - AIRA Conformance Test Specification
  - AIRA MVP Implementation Roadmap
  - AIRA Repository Bootstrap Plan
Exports:
  - GitHub-ready initial issues
  - Labels
  - Milestones
  - Dependencies
  - Acceptance Criteria
  - First PR Sequence
```

Цей документ перетворює MVP Roadmap і Repository Bootstrap Plan на практичний GitHub issue backlog. Мета не змінюється: перший MVP має довести локальний, мінімальний, deterministic, event-native, artifact-based, CSU-driven pipeline `Problem Statement → Verified Result Artifact`, без ML, GPU, blockchain, federation або distributed execution.  

---

# 1. Issue Set Mission

Initial Issue Set має забезпечити перехід від специфікації до реалізації без розщеплення концепції.

Перший backlog повинен:

```text
створити репозиторій;
зафіксувати специфікації;
реалізувати Schema Registry;
реалізувати C0 Core;
реалізувати C1 local operational node;
реалізувати мінімальні CSU;
довести flow Calculate 2 + 2;
запустити C0/C1 conformance;
не допустити повернення до GPU/LLM/Node/Scheduler-first архітектури.
```

AIRA MVP не є GPU marketplace, LLM runtime або blockchain network. Це локальне підтвердження формули `Problem Statement → Verified Result Artifact → Evidence → Lower-cost future Resolution`, зафіксованої в Book 0 і MVP Roadmap.  

---

# 2. Label Set

## 2.1 Type Labels

```text
type:spec
type:schema
type:core
type:artifact
type:event
type:policy
type:csu
type:conformance
type:cli
type:api
type:storage
type:security
type:docs
type:ci
type:demo
```

## 2.2 Priority Labels

```text
priority:p0
priority:p1
priority:p2
```

## 2.3 Profile Labels

```text
profile:c0
profile:c1
profile:c2-local
```

## 2.4 Status Labels

```text
status:blocked
status:ready
status:in-progress
status:needs-review
```

## 2.5 Risk Labels

```text
risk:core-pollution
risk:schema-drift
risk:scope-creep
risk:security
risk:conformance
```

---

# 3. Milestones

```text
M0 — Repository Bootstrap
M1 — Specification Snapshot
M2 — Schema Registry
M3 — C0 Core Runtime
M4 — Artifact / Event / Policy Runtime
M5 — CSU Runtime
M6 — Basic CSU Set
M7 — Local Operational Flow
M8 — CLI / Local Node
M9 — Conformance C0/C1
M10 — Partial Local C2
M11 — MVP Alpha Release
```

---

# 4. Epic 0 — Repository Bootstrap

## Issue #1 — Initialize repository

```yaml
title: Initialize AIRA repository
labels: [type:ci, priority:p0]
milestone: M0
depends_on: []
```

Description:

```text
Create the initial repository structure for AIRA MVP.
```

Acceptance Criteria:

```text
- Repository exists.
- Default branch is main.
- develop branch exists.
- README.md placeholder exists.
- LICENSE exists.
- .gitignore exists.
```

---

## Issue #2 — Add repository metadata

```yaml
title: Add README, LICENSE, CONTRIBUTING, SECURITY and CHANGELOG
labels: [type:docs, priority:p0]
milestone: M0
depends_on: [#1]
```

Acceptance Criteria:

```text
- README states AIRA MVP mission.
- README explicitly says AIRA is not GPU marketplace / LLM runtime / blockchain.
- CONTRIBUTING.md exists.
- SECURITY.md exists.
- CHANGELOG.md exists.
```

---

## Issue #3 — Add Cargo workspace

```yaml
title: Add initial Rust Cargo workspace
labels: [type:core, priority:p0]
milestone: M0
depends_on: [#1]
```

Acceptance Criteria:

```text
- Root Cargo.toml exists.
- Workspace resolver = 2.
- Empty MVP crates are declared.
- cargo metadata succeeds.
```

---

## Issue #4 — Add Rust toolchain and formatting config

```yaml
title: Add rust-toolchain, rustfmt and clippy config
labels: [type:ci, priority:p0]
milestone: M0
depends_on: [#3]
```

Acceptance Criteria:

```text
- rust-toolchain.toml exists.
- rustfmt.toml exists.
- clippy.toml exists.
- cargo fmt --check succeeds.
- cargo clippy succeeds on empty workspace.
```

---

## Issue #5 — Add initial CI pipeline

```yaml
title: Add GitHub Actions CI for format, lint and tests
labels: [type:ci, priority:p0]
milestone: M0
depends_on: [#3, #4]
```

Acceptance Criteria:

```text
- .github/workflows/ci.yml exists.
- CI runs cargo fmt.
- CI runs cargo clippy.
- CI runs cargo test.
- CI passes on main.
```

---

# 5. Epic 1 — Specification Snapshot

## Issue #6 — Add Book 0–V to specs/

```yaml
title: Add Book 0–V canonical specs
labels: [type:spec, priority:p0]
milestone: M1
depends_on: [#1]
```

Acceptance Criteria:

```text
- specs/book-0.md exists.
- specs/book-i.md exists.
- specs/book-ii.md exists.
- specs/book-iii.md exists.
- specs/book-iv.md exists.
- specs/book-v.md exists.
- No early P2P/GPU-first documents are used as normative specs.
```

---

## Issue #7 — Add governance and control documents

```yaml
title: Add Specification Control, RFC Process, Schema Pack and Conformance docs
labels: [type:spec, priority:p0]
milestone: M1
depends_on: [#6]
```

Acceptance Criteria:

```text
- specs/specification-control.md exists.
- specs/rfc-process.md exists.
- specs/schema-pack.md exists.
- specs/conformance.md exists.
- specs/mvp-roadmap.md exists.
- specs/repository-bootstrap.md exists.
```

---

## Issue #8 — Add canonical terminology guardrail

```yaml
title: Add canonical terminology guardrail document
labels: [type:spec, priority:p1, risk:core-pollution]
milestone: M1
depends_on: [#6]
```

Acceptance Criteria:

```text
- docs/canonical-terminology.md exists.
- Forbidden core terms are listed: GPU, Node, Driver, Scheduler, Blockchain, Wallet, LLM model.
- Canonical replacements are listed: CSU, Capability, Artifact, Event, Policy, Execution Capsule.
```

Canonical terminology already replaced `Driver → CSU`, `Job → Execution Capsule`, `Scheduler → Optimization CSU`, and moved blockchain/tokenomics into settlement/audit concepts. 

---

# 6. Epic 2 — Schema Registry

## Issue #9 — Create schema directory structure

```yaml
title: Create schema directory layout
labels: [type:schema, priority:p0]
milestone: M2
depends_on: [#7]
```

Acceptance Criteria:

```text
- schemas/common exists.
- schemas/core exists.
- schemas/artifact exists.
- schemas/event exists.
- schemas/policy exists.
- schemas/capability exists.
- schemas/csu exists.
- schemas/execution exists.
- schemas/evidence exists.
- schemas/result exists.
- schemas/conformance exists.
```

---

## Issue #10 — Add common schemas

```yaml
title: Add common ref, hash, signature and scope schemas
labels: [type:schema, priority:p0]
milestone: M2
depends_on: [#9]
```

Acceptance Criteria:

```text
- ref.schema.json exists.
- hash.schema.json exists.
- signature.schema.json exists.
- timestamp.schema.json exists.
- scope-descriptor.schema.json exists.
- Valid fixtures pass.
- Invalid fixtures fail.
```

---

## Issue #11 — Add Core Object Descriptor schema

```yaml
title: Add Core Object Descriptor schema
labels: [type:schema, priority:p0, profile:c0]
milestone: M2
depends_on: [#10]
```

Acceptance Criteria:

```text
- core/object-descriptor.schema.json exists.
- Allows only canonical Book 0 object types.
- Rejects GPU, Node, Driver, Scheduler, Blockchain as core object types.
```

---

## Issue #12 — Add Problem Statement schema

```yaml
title: Add Problem Statement schema
labels: [type:schema, priority:p0, profile:c0]
milestone: M2
depends_on: [#10]
```

Acceptance Criteria:

```text
- core/problem-statement.schema.json exists.
- Requires statement.
- Supports input_artifact_refs.
- Supports constraints.
- Supports policy_refs.
```

---

## Issue #13 — Add Artifact Descriptor schema

```yaml
title: Add Artifact Descriptor schema
labels: [type:schema, type:artifact, priority:p0, profile:c0]
milestone: M2
depends_on: [#10]
```

Acceptance Criteria:

```text
- artifact/artifact-descriptor.schema.json exists.
- Requires content_hash.
- Requires provenance_refs.
- Requires dependency_refs.
- Requires signature.
```

---

## Issue #14 — Add Event Descriptor schema

```yaml
title: Add Event Descriptor schema
labels: [type:schema, type:event, priority:p0, profile:c0]
milestone: M2
depends_on: [#10]
```

Acceptance Criteria:

```text
- event/event-descriptor.schema.json exists.
- Requires causal_refs.
- Requires payload_hash.
- Requires signature.
- Supports required MVP event types.
```

---

## Issue #15 — Add Policy Query and Decision schemas

```yaml
title: Add Policy Query and Policy Decision schemas
labels: [type:schema, type:policy, priority:p0, profile:c0]
milestone: M2
depends_on: [#10]
```

Acceptance Criteria:

```text
- policy-query.schema.json exists.
- policy-decision.schema.json exists.
- Decision enum is exactly ALLOW, DENY, REQUIRE.
```

---

## Issue #16 — Add CSU Manifest schema

```yaml
title: Add CSU Manifest schema
labels: [type:schema, type:csu, priority:p0, profile:c1]
milestone: M2
depends_on: [#10, #15]
```

Acceptance Criteria:

```text
- csu/manifest.schema.json exists.
- Requires csu_id, csu_type, abi_version, capabilities, permissions, event_subscriptions, signature.
- Rejects unsigned manifest fixture.
```

---

## Issue #17 — Add Capability Descriptor schema

```yaml
title: Add Capability Descriptor schema
labels: [type:schema, type:csu, priority:p0, profile:c1]
milestone: M2
depends_on: [#10, #16]
```

Acceptance Criteria:

```text
- capability/capability-descriptor.schema.json exists.
- Describes ability, not hardware.
- Does not require node_id, gpu_id, model_id.
```

---

## Issue #18 — Add Execution Capsule schema

```yaml
title: Add Execution Capsule schema
labels: [type:schema, priority:p0, profile:c1]
milestone: M2
depends_on: [#11, #12, #17]
```

Acceptance Criteria:

```text
- execution/capsule.schema.json exists.
- Requires problem_statement_ref.
- Requires context_ref.
- Requires required_capabilities.
- Requires policy_refs.
```

---

## Issue #19 — Add Evidence and Verified Result schemas

```yaml
title: Add Evidence Artifact and Verified Result Artifact schemas
labels: [type:schema, priority:p0, profile:c1]
milestone: M2
depends_on: [#13]
```

Acceptance Criteria:

```text
- evidence/evidence-artifact.schema.json exists.
- result/verified-result-artifact.schema.json exists.
- Verified Result requires evidence_refs and provenance_refs.
- Missing evidence fixture fails.
```

---

## Issue #20 — Implement aira-schema crate

```yaml
title: Implement schema registry and validation crate
labels: [type:schema, priority:p0]
milestone: M2
depends_on: [#10, #11, #12, #13, #14, #15, #16, #17, #18, #19]
```

Acceptance Criteria:

```text
- aira-schema crate loads schemas.
- Can validate JSON file by schema id.
- Unit tests cover valid and invalid fixtures.
```

---

## Issue #21 — Add schema validation CLI

```yaml
title: Add `aira schema validate` command
labels: [type:cli, type:schema, priority:p0]
milestone: M2
depends_on: [#20]
```

Acceptance Criteria:

```text
- `aira schema list` works.
- `aira schema validate --schema <id> --file <path>` works.
- `aira schema validate --fixtures fixtures` works.
```

Schema Pack exists specifically to make AIRA compatibility machine-checkable through canonical descriptors, protocol envelopes, CSU manifests, result artifacts, evidence artifacts and conformance reports. 

---

# 7. Epic 3 — C0 Core Runtime

## Issue #22 — Implement common identifiers and references

```yaml
title: Implement AIRA ref, id, hash and signature value types
labels: [type:core, priority:p0, profile:c0]
milestone: M3
depends_on: [#20]
```

Acceptance Criteria:

```text
- AiraRef type exists.
- Hash type exists.
- Signature type exists.
- Serialization/deserialization tests pass.
```

---

## Issue #23 — Implement opaque Handle model

```yaml
title: Implement opaque handle model
labels: [type:core, priority:p0, profile:c0]
milestone: M3
depends_on: [#22]
```

Acceptance Criteria:

```text
- Handle type exists.
- CSU cannot access internal storage path from Handle.
- Handle opacity test exists.
```

---

## Issue #24 — Implement Object Descriptor

```yaml
title: Implement Core Object Descriptor type
labels: [type:core, priority:p0, profile:c0]
milestone: M3
depends_on: [#11, #22]
```

Acceptance Criteria:

```text
- ObjectDescriptor maps to schema.
- Schema validation test passes.
- Forbidden object types rejected.
```

---

## Issue #25 — Implement immutable Object Store

```yaml
title: Implement immutable Object Store
labels: [type:core, type:storage, priority:p0, profile:c0]
milestone: M3
depends_on: [#23, #24]
```

Acceptance Criteria:

```text
- Object can be created.
- Object can be opened by handle.
- Object cannot be mutated in-place.
- Mutation attempt returns InvariantViolation candidate.
```

---

## Issue #26 — Implement basic SQLite storage adapter

```yaml
title: Implement SQLite adapter for local Object Store
labels: [type:storage, priority:p1, profile:c0]
milestone: M3
depends_on: [#25]
```

Acceptance Criteria:

```text
- objects table created.
- object descriptor persisted.
- object lookup by object_id works.
- duplicate insert is idempotent or rejected deterministically.
```

---

# 8. Epic 4 — Artifact / Event / Policy Runtime

## Issue #27 — Implement Artifact Descriptor type

```yaml
title: Implement Artifact Descriptor
labels: [type:artifact, priority:p0, profile:c0]
milestone: M4
depends_on: [#13, #22]
```

Acceptance Criteria:

```text
- ArtifactDescriptor maps to schema.
- content_hash is required.
- provenance_refs are required.
```

---

## Issue #28 — Implement content-addressed Artifact Store

```yaml
title: Implement local content-addressed Artifact Store
labels: [type:artifact, type:storage, priority:p0, profile:c0]
milestone: M4
depends_on: [#27]
```

Acceptance Criteria:

```text
- Artifact payload stored by SHA-256 path.
- Resolve by artifact_ref works.
- Hash mismatch rejected.
- Artifact mutation attempt fails.
```

---

## Issue #29 — Implement Artifact supersession metadata

```yaml
title: Implement artifact supersession without mutation
labels: [type:artifact, priority:p1, profile:c0]
milestone: M4
depends_on: [#28]
```

Acceptance Criteria:

```text
- New artifact can supersede previous artifact.
- Previous artifact remains unchanged.
- Supersession event can reference both artifacts.
```

---

## Issue #30 — Implement Event Descriptor type

```yaml
title: Implement Event Descriptor
labels: [type:event, priority:p0, profile:c0]
milestone: M4
depends_on: [#14, #22]
```

Acceptance Criteria:

```text
- EventDescriptor maps to schema.
- causal_refs are preserved.
- payload_hash is required.
```

---

## Issue #31 — Implement append-only Event Log

```yaml
title: Implement local append-only Event Log
labels: [type:event, type:storage, priority:p0, profile:c0]
milestone: M4
depends_on: [#30]
```

Acceptance Criteria:

```text
- Event can be appended.
- Event cannot be modified.
- Event query by object_ref works.
- Event query by artifact_ref works.
```

---

## Issue #32 — Implement local event subscriptions

```yaml
title: Implement local Event Runtime subscriptions
labels: [type:event, priority:p0, profile:c0]
milestone: M4
depends_on: [#31]
```

Acceptance Criteria:

```text
- CSU/runtime can subscribe by event type.
- Duplicate event delivery is idempotent.
- No global total ordering is required.
```

Event Runtime is the coordination mechanism between Core and CSU, and protocol conformance requires causal references without mandatory global total ordering.  

---

## Issue #33 — Implement Policy Gate

```yaml
title: Implement minimal Policy Gate
labels: [type:policy, priority:p0, profile:c0]
milestone: M4
depends_on: [#15, #31]
```

Acceptance Criteria:

```text
- policy_check(query) returns ALLOW, DENY or REQUIRE.
- PolicyEvaluated event emitted.
- Unknown controlled action defaults to DENY.
```

---

## Issue #34 — Implement Invariant Checker

```yaml
title: Implement C0 Invariant Checker
labels: [type:core, priority:p0, profile:c0]
milestone: M4
depends_on: [#25, #28, #31, #33]
```

Acceptance Criteria:

```text
- Object immutability invariant checked.
- Artifact immutability invariant checked.
- Event signature presence checked.
- Policy-before-action invariant checked.
- InvariantViolation event emitted when needed.
```

---

# 9. Epic 5 — CSU Runtime

## Issue #35 — Implement CSU Manifest type

```yaml
title: Implement CSU Manifest type
labels: [type:csu, priority:p0, profile:c1]
milestone: M5
depends_on: [#16, #22]
```

Acceptance Criteria:

```text
- Manifest maps to schema.
- Required fields validated.
- Unsigned manifest fixture rejected.
```

---

## Issue #36 — Implement CSU Registry

```yaml
title: Implement local CSU Registry
labels: [type:csu, priority:p0, profile:c1]
milestone: M5
depends_on: [#35]
```

Acceptance Criteria:

```text
- CSU can be registered.
- ABI version checked.
- Manifest signature presence checked.
- Registered CSU appears in `aira csu list`.
```

---

## Issue #37 — Implement CSU lifecycle states

```yaml
title: Implement CSU lifecycle state machine
labels: [type:csu, priority:p1, profile:c1]
milestone: M5
depends_on: [#36]
```

Acceptance Criteria:

```text
- Discovered, Registered, Verified, Active, Suspended, Revoked, Archived states represented.
- Invalid transitions rejected.
- Lifecycle transition emits event.
```

---

## Issue #38 — Implement CSU trait

```yaml
title: Implement in-process CSU trait
labels: [type:csu, priority:p0, profile:c1]
milestone: M5
depends_on: [#36, #32]
```

Acceptance Criteria:

```text
- Csu trait exposes manifest().
- Csu trait exposes on_event().
- CSU output supports Event, Artifact, PolicyQuery, Failure.
```

---

## Issue #39 — Implement CSU event dispatch

```yaml
title: Dispatch Events to active CSU
labels: [type:csu, type:event, priority:p0, profile:c1]
milestone: M5
depends_on: [#32, #38]
```

Acceptance Criteria:

```text
- Active CSU receives subscribed events.
- Suspended CSU does not receive events.
- Dispatch failure emits CSUFailed event.
```

---

## Issue #40 — Enforce CSU isolation baseline

```yaml
title: Enforce MVP CSU isolation baseline
labels: [type:csu, type:security, priority:p0, profile:c1]
milestone: M5
depends_on: [#38, #39]
```

Acceptance Criteria:

```text
- CSU cannot mutate Core Object directly.
- CSU cannot mutate Artifact directly.
- CSU cannot call another CSU directly.
- CSU uses Event/Artifact/Core APIs only.
```

Book III defines CSU as isolated, replaceable modules using ABI, Events, Artifacts and Policy, not direct peer dependencies or Core mutation. 

---

# 10. Epic 6 — Basic CSU Set

## Issue #41 — Implement Context CSU

```yaml
title: Implement context-basic CSU
labels: [type:csu, priority:p0, profile:c1]
milestone: M6
depends_on: [#38, #39]
```

Acceptance Criteria:

```text
- Observes ProblemSubmitted.
- Creates Context Artifact.
- Emits ContextResolved.
- Marks unresolved ambiguity.
- Does not execute.
- Does not produce Result.
```

---

## Issue #42 — Implement Reduction CSU

```yaml
title: Implement reduction-basic CSU
labels: [type:csu, priority:p0, profile:c1]
milestone: M6
depends_on: [#28, #41]
```

Acceptance Criteria:

```text
- Checks Ready Solution artifacts first.
- Checks local Knowledge artifacts second.
- Creates Negative Lookup Artifact if lookup fails.
- Creates Execution Capsule only if needed.
- Emits ReductionCompleted.
```

---

## Issue #43 — Implement Execution CSU

```yaml
title: Implement execution-basic CSU with safe deterministic actions
labels: [type:csu, priority:p0, profile:c1]
milestone: M6
depends_on: [#18, #39, #42]
```

Acceptance Criteria:

```text
- Supports math.eval.safe.
- Supports text.echo.
- Supports text.uppercase.
- Does not execute arbitrary shell.
- Does not access network.
- Emits CapsuleCompleted or CapsuleFailed.
```

---

## Issue #44 — Implement Verification CSU

```yaml
title: Implement verification-basic CSU
labels: [type:csu, priority:p0, profile:c1]
milestone: M6
depends_on: [#19, #43]
```

Acceptance Criteria:

```text
- Distinguishes Output Artifact from Verified Result Artifact.
- Verifies math.eval.safe output.
- Publishes Verified Result Artifact when valid.
- Emits VerificationCompleted.
- Emits VerificationFailed when invalid.
```

---

## Issue #45 — Implement Evidence CSU

```yaml
title: Implement evidence-basic CSU
labels: [type:csu, priority:p0, profile:c1]
milestone: M6
depends_on: [#19, #31, #44]
```

Acceptance Criteria:

```text
- Observes ResultPublished.
- Observes CapsuleFailed.
- Observes VerificationFailed.
- Creates Evidence Artifact.
- Creates Failure Evidence Artifact for failures.
```

---

## Issue #46 — Implement Artifact CSU

```yaml
title: Implement artifact-basic CSU
labels: [type:csu, type:artifact, priority:p1, profile:c1]
milestone: M6
depends_on: [#28, #29]
```

Acceptance Criteria:

```text
- Publishes artifacts through Artifact Runtime.
- Resolves artifacts through Artifact Runtime.
- Emits ArtifactPublished, ArtifactResolved, ArtifactInvalid, ArtifactSuperseded.
```

---

# 11. Epic 7 — Local Operational Flow

## Issue #47 — Implement problem submission service

```yaml
title: Implement local Problem Statement submission
labels: [type:core, priority:p0, profile:c1]
milestone: M7
depends_on: [#12, #25, #31]
```

Acceptance Criteria:

```text
- Problem Statement object created.
- ProblemSubmitted event emitted.
- Descriptor schema-valid.
```

---

## Issue #48 — Implement context creation flow

```yaml
title: Wire ProblemSubmitted to ContextResolved flow
labels: [type:csu, type:event, priority:p0, profile:c1]
milestone: M7
depends_on: [#41, #47]
```

Acceptance Criteria:

```text
- ProblemSubmitted triggers Context CSU.
- Context Artifact created.
- ContextResolved event emitted.
```

---

## Issue #49 — Implement execution capsule flow

```yaml
title: Wire ContextResolved to Execution Capsule creation
labels: [type:csu, priority:p0, profile:c1]
milestone: M7
depends_on: [#42, #48]
```

Acceptance Criteria:

```text
- Reduction CSU creates Execution Capsule when no Ready Solution exists.
- CapsuleCreated event emitted.
- Capsule schema-valid.
```

---

## Issue #50 — Implement capsule execution flow

```yaml
title: Wire CapsuleCreated to Output Artifact creation
labels: [type:csu, priority:p0, profile:c1]
milestone: M7
depends_on: [#43, #49]
```

Acceptance Criteria:

```text
- Execution CSU receives CapsuleCreated.
- Output Artifact created.
- CapsuleCompleted event emitted.
```

---

## Issue #51 — Implement verification and result publication flow

```yaml
title: Wire Output Artifact to Verified Result Artifact
labels: [type:csu, priority:p0, profile:c1]
milestone: M7
depends_on: [#44, #50]
```

Acceptance Criteria:

```text
- Verification CSU validates Output Artifact.
- Verified Result Artifact created.
- ResultPublished event emitted.
- Result has evidence_refs and provenance_refs.
```

---

## Issue #52 — Implement evidence creation flow

```yaml
title: Wire ResultPublished and failures to Evidence Artifact
labels: [type:csu, priority:p0, profile:c1]
milestone: M7
depends_on: [#45, #51]
```

Acceptance Criteria:

```text
- ResultPublished produces Evidence Artifact.
- Failure Event produces Failure Evidence Artifact.
- Evidence Artifact is queryable.
```

---

## Issue #53 — Implement Calculate 2 + 2 demo

```yaml
title: Implement first e2e demo: Calculate 2 + 2
labels: [type:demo, priority:p0, profile:c1]
milestone: M7
depends_on: [#47, #48, #49, #50, #51, #52]
```

Acceptance Criteria:

```text
- User submits "Calculate 2 + 2".
- System produces Verified Result Artifact.
- Result value is 4.
- verification_status = VERIFIED.
- confidence = 1.0.
- All events are queryable.
```

---

## Issue #54 — Implement Ready Solution reuse demo

```yaml
title: Implement Ready Solution reuse demo
labels: [type:demo, priority:p1, profile:c1]
milestone: M7
depends_on: [#42, #51, #53]
```

Acceptance Criteria:

```text
- Ready Solution Artifact exists before problem submission.
- Reduction CSU reuses Ready Solution.
- Execution CSU is not invoked.
- ResultPublished emitted.
```

This validates `Reuse before compute`, one of the core principles of AIRA MVP.  

---

## Issue #55 — Implement failure-to-evidence demo

```yaml
title: Implement failure-to-evidence demo
labels: [type:demo, priority:p1, profile:c1]
milestone: M7
depends_on: [#43, #45, #52]
```

Acceptance Criteria:

```text
- Execution Capsule references missing input artifact.
- CapsuleFailed event emitted.
- No Verified Result Artifact is created.
- Failure Evidence Artifact created.
```

---

## Issue #56 — Implement normative split stub

```yaml
title: Implement Differentiated Solution Field stub
labels: [type:demo, priority:p2, profile:c1]
milestone: M7
depends_on: [#41, #44]
```

Acceptance Criteria:

```text
- Test problem with two normative alternatives does not auto-collapse.
- Differentiated Solution Field Artifact created.
- requires_human_collapse = true.
```

---

# 12. Epic 8 — CLI / Local Node

## Issue #57 — Implement `aira init`

```yaml
title: Implement local node initialization
labels: [type:cli, priority:p0]
milestone: M8
depends_on: [#25, #28, #31]
```

Acceptance Criteria:

```text
- `aira init` creates .aira directory.
- SQLite DB initialized.
- Artifact directory initialized.
- Config file created.
```

---

## Issue #58 — Implement `aira identity create`

```yaml
title: Implement local identity creation
labels: [type:cli, type:security, priority:p0]
milestone: M8
depends_on: [#57]
```

Acceptance Criteria:

```text
- Ed25519 keypair generated.
- Identity descriptor created.
- Identity stored locally.
```

---

## Issue #59 — Implement CSU CLI commands

```yaml
title: Implement `aira csu list/register`
labels: [type:cli, type:csu, priority:p0]
milestone: M8
depends_on: [#36, #58]
```

Acceptance Criteria:

```text
- `aira csu register <manifest>` works.
- `aira csu list` works.
- Invalid manifest rejected.
```

---

## Issue #60 — Implement problem CLI commands

```yaml
title: Implement `aira problem submit/status`
labels: [type:cli, priority:p0]
milestone: M8
depends_on: [#47, #57]
```

Acceptance Criteria:

```text
- `aira problem submit --text "Calculate 2 + 2"` works.
- Problem ref returned.
- `aira problem status <problem-ref>` works.
```

---

## Issue #61 — Implement result, artifact and event CLI commands

```yaml
title: Implement result, artifact and event read commands
labels: [type:cli, priority:p0]
milestone: M8
depends_on: [#28, #31, #51]
```

Acceptance Criteria:

```text
- `aira result get <result-ref>` works.
- `aira artifact get <artifact-ref>` works.
- `aira event tail` works.
```

---

## Issue #62 — Implement local node binary

```yaml
title: Implement `aira-node` local runtime binary
labels: [type:core, type:cli, priority:p1]
milestone: M8
depends_on: [#57, #59, #60, #61]
```

Acceptance Criteria:

```text
- aira-node starts locally.
- Loads config.
- Loads registered CSU.
- Processes local events.
```

---

# 13. Epic 9 — Conformance C0/C1

## Issue #63 — Implement Conformance Report Artifact

```yaml
title: Implement Conformance Report Artifact
labels: [type:conformance, priority:p0]
milestone: M9
depends_on: [#19, #20]
```

Acceptance Criteria:

```text
- Conformance report schema implemented.
- Report includes implementation, profile, results, failures, signature.
- Report is immutable artifact.
```

---

## Issue #64 — Implement C0 conformance runner

```yaml
title: Implement C0 conformance runner
labels: [type:conformance, priority:p0, profile:c0]
milestone: M9
depends_on: [#34, #63]
```

Acceptance Criteria:

```text
- C0 runner executes ontology tests.
- C0 runner executes object immutability tests.
- C0 runner executes artifact immutability tests.
- C0 runner executes event causality tests.
- C0 runner emits Conformance Report Artifact.
```

---

## Issue #65 — Implement C1 conformance runner

```yaml
title: Implement C1 conformance runner
labels: [type:conformance, priority:p0, profile:c1]
milestone: M9
depends_on: [#53, #64]
```

Acceptance Criteria:

```text
- C1 runner executes minimal operational pipeline tests.
- C1 runner validates CSU manifests.
- C1 runner validates Verified Result Artifact completeness.
- C1 runner validates failure-to-evidence behavior.
```

Conformance Specification defines profile-based validation for Book 0 ontology, Book I Core/ABI, Book II protocols, Book III CSU contracts, Verified Result Artifact, and failure-to-evidence behavior. 

---

## Issue #66 — Add object immutability conformance test

```yaml
title: Add object immutability conformance test
labels: [type:conformance, priority:p0, profile:c0]
milestone: M9
depends_on: [#64]
```

Acceptance Criteria:

```text
- In-place object mutation attempt fails.
- InvariantViolation event is created.
```

---

## Issue #67 — Add artifact immutability conformance test

```yaml
title: Add artifact immutability conformance test
labels: [type:conformance, priority:p0, profile:c0]
milestone: M9
depends_on: [#64]
```

Acceptance Criteria:

```text
- Artifact content mutation attempt fails.
- ArtifactInvalid or InvariantViolation event emitted.
```

---

## Issue #68 — Add event causality conformance test

```yaml
title: Add event causality conformance test
labels: [type:conformance, priority:p0, profile:c0]
milestone: M9
depends_on: [#64]
```

Acceptance Criteria:

```text
- ProblemSubmitted → ContextResolved → CapsuleCreated → CapsuleCompleted → ResultPublished chain valid.
- causal_refs preserved.
```

---

## Issue #69 — Add policy gate conformance test

```yaml
title: Add Policy Gate conformance test
labels: [type:conformance, type:policy, priority:p0, profile:c0]
milestone: M9
depends_on: [#64]
```

Acceptance Criteria:

```text
- Controlled action without Policy Check rejected.
- Policy decision enum limited to ALLOW/DENY/REQUIRE.
```

---

## Issue #70 — Add failure-to-evidence conformance test

```yaml
title: Add failure-to-evidence conformance test
labels: [type:conformance, priority:p0, profile:c1]
milestone: M9
depends_on: [#55, #65]
```

Acceptance Criteria:

```text
- Capsule failure emits Failure Event.
- Failure Evidence Artifact created.
- No fake Verified Result Artifact created.
```

---

# 14. Epic 10 — Partial Local C2

## Issue #71 — Implement Protocol Envelope type

```yaml
title: Implement local Protocol Envelope type
labels: [type:schema, type:protocol, priority:p1, profile:c2-local]
milestone: M10
depends_on: [#20]
```

Acceptance Criteria:

```text
- Protocol Envelope maps to Schema Pack.
- protocol_id supported for AIRA-EP, AIRA-AP, AIRA-ID, AIRA-DP.
- Invalid signature fixture rejected.
```

---

## Issue #72 — Implement local Event Protocol adapter

```yaml
title: Implement local AIRA-EP Event Protocol adapter
labels: [type:event, priority:p1, profile:c2-local]
milestone: M10
depends_on: [#31, #71]
```

Acceptance Criteria:

```text
- Local event publish maps to protocol envelope.
- Duplicate delivery is idempotent.
- Unsupported version returns UNSUPPORTED_VERSION.
```

---

## Issue #73 — Implement local Artifact Protocol adapter

```yaml
title: Implement local AIRA-AP Artifact Protocol adapter
labels: [type:artifact, priority:p1, profile:c2-local]
milestone: M10
depends_on: [#28, #71]
```

Acceptance Criteria:

```text
- Artifact resolve request supported.
- Artifact publish request supported.
- Hash validation enforced.
```

---

## Issue #74 — Implement local Identity Protocol descriptor

```yaml
title: Implement local AIRA-ID Identity Descriptor
labels: [type:security, priority:p1, profile:c2-local]
milestone: M10
depends_on: [#58, #71]
```

Acceptance Criteria:

```text
- Identity descriptor schema-valid.
- Public key included.
- Signature included.
```

---

## Issue #75 — Implement local Discovery registry

```yaml
title: Implement local capability-based Discovery registry
labels: [type:csu, type:protocol, priority:p1, profile:c2-local]
milestone: M10
depends_on: [#17, #36, #71]
```

Acceptance Criteria:

```text
- Discovery query returns Capability, not Node.
- Capability provider CSU returned.
- No global registry required.
```

Book II requires discovery and routing by Capability rather than Node, and keeps algorithms outside the protocol requirements. 

---

# 15. Epic 11 — MVP Alpha Release

## Issue #76 — Add demo documentation

```yaml
title: Add demo documentation for MVP flows
labels: [type:docs, type:demo, priority:p1]
milestone: M11
depends_on: [#53, #54, #55]
```

Acceptance Criteria:

```text
- docs/demo.md exists.
- Calculate 2 + 2 demo documented.
- Ready Solution reuse demo documented.
- Failure-to-evidence demo documented.
```

---

## Issue #77 — Add developer guide

```yaml
title: Add MVP developer guide
labels: [type:docs, priority:p1]
milestone: M11
depends_on: [#40, #46, #65]
```

Acceptance Criteria:

```text
- docs/csu-development.md exists.
- docs/local-node.md exists.
- docs/conformance.md exists.
- Local setup documented.
```

---

## Issue #78 — Add security baseline tests

```yaml
title: Add MVP security baseline tests
labels: [type:security, type:conformance, priority:p1]
milestone: M11
depends_on: [#33, #40, #65]
```

Acceptance Criteria:

```text
- Unsigned CSU rejected.
- Unsigned artifact rejected.
- Private artifact access denied by default.
- Secret material not emitted into events.
```

---

## Issue #79 — Prepare alpha release artifacts

```yaml
title: Prepare aira-mvp-v0.1.0-alpha.1 release artifacts
labels: [type:ci, priority:p1]
milestone: M11
depends_on: [#65, #76, #77, #78]
```

Acceptance Criteria:

```text
- aira-node binary builds.
- aira CLI builds.
- schema pack included.
- demo fixtures included.
- conformance report generated.
- release notes drafted.
```

---

## Issue #80 — Final MVP alpha acceptance

```yaml
title: Validate MVP alpha Definition of Done
labels: [type:conformance, priority:p0]
milestone: M11
depends_on: [#79]
```

Acceptance Criteria:

```text
- Clean checkout builds.
- `aira init` works.
- `aira identity create` works.
- Basic CSU set registers.
- `aira problem submit --text "Calculate 2 + 2"` produces Verified Result Artifact.
- Failure path produces Evidence Artifact.
- C0 conformance passes.
- C1 conformance passes.
- No ML/GPU/blockchain/network dependency required.
```

---

# 16. First PR Sequence

Оптимальна перша послідовність PR:

```text
PR-001: repository skeleton
  Issues: #1, #2, #3, #4, #5

PR-002: specification snapshot
  Issues: #6, #7, #8

PR-003: schema directory + common schemas
  Issues: #9, #10

PR-004: core schemas
  Issues: #11, #12, #13, #14, #15

PR-005: CSU/result/evidence schemas
  Issues: #16, #17, #18, #19

PR-006: schema registry crate + CLI
  Issues: #20, #21

PR-007: core object runtime
  Issues: #22, #23, #24, #25, #26

PR-008: artifact/event/policy runtime
  Issues: #27, #28, #29, #30, #31, #32, #33, #34

PR-009: CSU runtime
  Issues: #35, #36, #37, #38, #39, #40

PR-010: basic CSU set
  Issues: #41, #42, #43, #44, #45, #46

PR-011: local operational flow
  Issues: #47, #48, #49, #50, #51, #52, #53

PR-012: CLI and local node
  Issues: #57, #58, #59, #60, #61, #62

PR-013: C0/C1 conformance
  Issues: #63, #64, #65, #66, #67, #68, #69, #70

PR-014: demos and docs
  Issues: #54, #55, #56, #76, #77

PR-015: local C2 protocol stubs
  Issues: #71, #72, #73, #74, #75

PR-016: alpha release
  Issues: #78, #79, #80
```

---

# 17. Dependency Summary

Critical path:

```text
#1
↓
#3
↓
#20
↓
#25
↓
#28 + #31 + #33 + #34
↓
#36 + #38 + #39
↓
#41 → #42 → #43 → #44 → #45
↓
#47 → #48 → #49 → #50 → #51 → #52 → #53
↓
#64 → #65
↓
#80
```

Non-critical but required before release:

```text
#54 Ready Solution reuse
#55 Failure-to-evidence demo
#56 Normative split stub
#71–#75 Partial local C2
#76–#79 Docs/security/release
```

---

# 18. Issue Writing Rules

Кожен GitHub issue повинен містити:

```text
Problem
Scope
Non-Goals
Dependencies
Acceptance Criteria
Conformance Impact
Security / Privacy Impact
```

Forbidden issue patterns:

```text
"Implement AI"
"Add scheduler"
"Add GPU support"
"Add blockchain"
"Build network"
"Add LLM"
"Optimize everything"
```

Такі задачі порушують MVP scope і повертають проєкт до ранньої, вже перекритої GPU/LLM-first моделі. Початкова концепція справді стартувала як P2P LLM/GPU compute network, але актуальний корпус фіксує AIRA як систему `Problem Statement → Verified Result Artifact` із мінімально необхідними ресурсами.  

---

# 19. MVP Backlog Freeze Rule

До завершення Issue #80 заборонено додавати в MVP scope:

```text
real federation
multi-node CRP
GPU execution
LLM inference
blockchain settlement
PHM
Research Plane
self-optimization
web UI
cloud deployment
Kubernetes
```

Допускаються лише:

```text
bug fixes;
schema fixes;
test fixtures;
documentation;
local deterministic implementation;
C0/C1 conformance work;
partial local C2 stubs.
```

---

# 20. Status

AIRA Initial Issue Set v0.1 формалізує перший GitHub-ready backlog.

Він перетворює специфікації на послідовну інженерну роботу:

```text
specs
↓
schemas
↓
core
↓
artifact/event/policy runtime
↓
CSU runtime
↓
basic CSU
↓
local operational flow
↓
CLI/node
↓
conformance
↓
alpha release
```

Цей backlog зберігає головну ідею AIRA:

```text
найкраще вирішення задачі
мінімально необхідними ресурсами
через перевірні артефакти,
ізольовані CSU,
стабільне ядро
та conformance-first реалізацію.
```

Наступний документ:

```text
AIRA MVP PRD v0.1
```

Scope:

```text
product goal;
target users;
non-goals;
MVP user flows;
functional requirements;
non-functional requirements;
security baseline;
acceptance scenarios;
release criteria;
out-of-scope features.
```
