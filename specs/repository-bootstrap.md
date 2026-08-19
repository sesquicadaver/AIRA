# AIRA Repository Bootstrap Plan v0.1

```text
Type: Reference / Planning / Engineering
Scope: AIRA Repository Initialization
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
Exports:
  - Repository Structure
  - Initial Cargo Workspace
  - Crate Boundaries
  - Schema Layout
  - Fixture Layout
  - First CLI Surface
  - First End-to-End Test
  - Initial CI Pipeline
  - Coding Standards
  - Commit / Branch Rules
  - MVP Issue Breakdown
```

AIRA Repository Bootstrap Plan визначає **першу інженерну структуру репозиторію**, достатню для реалізації MVP. Мета MVP уже зафіксована: довести локальний, мінімальний, deterministic, event-native, artifact-based, CSU-driven pipeline `Problem Statement → Verified Result Artifact`, без залежності від ML, GPU, blockchain, federation або distributed execution. 

---

# 1. Bootstrap Mission

Перший репозиторій AIRA має довести лише одне:

```text
Problem Statement
↓
Context Artifact
↓
Execution Capsule
↓
Output Artifact
↓
Verified Result Artifact
↓
Evidence Artifact
```

Не треба починати з network, federation, CRP, scheduler, GPU, LLM або blockchain.

Перший репозиторій повинен бути:

```text
small
typed
testable
schema-driven
conformance-first
offline-capable
deterministic
safe-by-default
```

Це відповідає Book 0, де AIRA визначена не як GPU/LLM marketplace, а як система перетворення `Problem Statement` на `Verified Result Artifact` з мінімальною сукупною вартістю. 

---

# 2. Non-Goals

Bootstrap **не включає**:

```text
multi-node execution
real federation
real distributed discovery
real CRP routing
real settlement ledger
real PHM
real optimizer
LLM inference
GPU backend
blockchain
cloud deployment
Kubernetes manifests
production auth
web UI
research plane
```

Будь-який такий компонент не входить у bootstrap, якщо він не потрібен для C0/C1 conformance.

---

# 3. Repository Name

Canonical repository name:

```text
aira
```

Alternative reserved names:

```text
aira-core
aira-reference
aira-node
```

Primary recommendation:

```text
github.com/<org>/aira
```

Починати з monorepo. На цьому етапі split-repo створить зайву координаційну вартість.

---

# 4. Root Layout

```text
aira/
 ├── .github/
 │   ├── workflows/
 │   │   ├── ci.yml
 │   │   ├── conformance.yml
 │   │   └── security.yml
 │   ├── ISSUE_TEMPLATE/
 │   └── pull_request_template.md
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
 │   ├── artifact/
 │   ├── event/
 │   ├── policy/
 │   ├── capability/
 │   ├── csu/
 │   ├── execution/
 │   ├── evidence/
 │   ├── result/
 │   └── conformance/
 │
 ├── fixtures/
 │   ├── valid/
 │   ├── invalid/
 │   └── flows/
 │
 ├── analysis/
 │   ├── README.md
 │   └── Analyze-N/          # analytical cycles (research, review, Living Spec)
 │
 ├── specs/
 │   ├── book-0.md
 │   ├── book-i.md
 │   ├── book-ii.md
 │   ├── book-iii.md
 │   ├── book-iv.md
 │   ├── book-v.md
 │   ├── specification-control.md
 │   ├── conformance.md
 │   ├── schema-pack.md
 │   ├── rfc-process.md
 │   ├── mvp-roadmap.md
 │   └── repository-bootstrap.md
 │
 ├── rfcs/
 │   ├── accepted/
 │   ├── draft/
 │   ├── rejected/
 │   ├── superseded/
 │   └── templates/
 │
 ├── tests/
 │   ├── conformance/
 │   ├── integration/
 │   └── e2e/
 │
 ├── docs/
 │   ├── architecture.md
 │   ├── local-node.md
 │   ├── csu-development.md
 │   ├── artifact-store.md
 │   ├── event-runtime.md
 │   ├── conformance.md
 │   └── demo.md
 │
 ├── examples/
 │   ├── calculate-2-plus-2/
 │   ├── ready-solution-reuse/
 │   └── failure-to-evidence/
 │
 ├── Cargo.toml
 ├── Cargo.lock
 ├── rust-toolchain.toml
 ├── rustfmt.toml
 ├── clippy.toml
 ├── deny.toml
 ├── README.md
 ├── CONTRIBUTING.md
 ├── SECURITY.md
 ├── LICENSE
 └── CHANGELOG.md
```

---

# 5. Initial Cargo Workspace

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/aira-core",
    "crates/aira-object",
    "crates/aira-event",
    "crates/aira-artifact",
    "crates/aira-policy",
    "crates/aira-csu",
    "crates/aira-schema",
    "crates/aira-conformance",
    "crates/aira-node",
    "crates/aira-cli",

    "csu/context-basic",
    "csu/reduction-basic",
    "csu/execution-basic",
    "csu/verification-basic",
    "csu/evidence-basic",
    "csu/artifact-basic",
]

[workspace.package]
edition = "2021"
license = "Apache-2.0 OR MIT"
repository = "https://github.com/<org>/aira"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
thiserror = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonschema = "0.18"
uuid = { version = "1", features = ["v7", "serde"] }
sha2 = "0.10"
ed25519-dalek = "2"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

---

# 6. Crate Boundaries

## 6.1 `aira-core`

Відповідальність:

```text
Core runtime orchestration
Invariant Checker
Handle Registry
Core error model
```

Must not contain:

```text
domain logic
scheduler logic
ML logic
LLM logic
GPU logic
network routing
```

---

## 6.2 `aira-object`

Відповідальність:

```text
Core Object Descriptor
Object Store interface
Object immutability
Object handles
Content hash
Provenance refs
```

---

## 6.3 `aira-event`

Відповідальність:

```text
Event Descriptor
Append-only Event Log
Event subscription
Causal refs
Idempotency
```

Book I визначає Event Runtime як механізм координації Core/CSU, а Book II фіксує Event Protocol без вимоги глобального total ordering.  

---

## 6.4 `aira-artifact`

Відповідальність:

```text
Artifact Descriptor
Content-addressed storage
Artifact resolve
Artifact publish
Artifact hash validation
Supersession
```

Artifacts immutable. Mutation заборонена.

---

## 6.5 `aira-policy`

Відповідальність:

```text
Policy Query
Policy Decision
Policy Gate
ALLOW / DENY / REQUIRE
```

Policy Gate не виконує дії. Він лише повертає рішення.

---

## 6.6 `aira-csu`

Відповідальність:

```text
CSU Manifest
CSU lifecycle
CSU trait
CSU registration
CSU event dispatch
CSU output handling
```

Book III фіксує CSU як ізольовані, замінні модулі, що працюють через ABI, Events, Artifacts і Policy. 

---

## 6.7 `aira-schema`

Відповідальність:

```text
JSON Schema loading
Schema registry
Fixture validation
Schema validation CLI support
```

Schema Pack визначає мінімальні структури даних для machine-checkable compatibility. 

---

## 6.8 `aira-conformance`

Відповідальність:

```text
C0/C1 test runner
Conformance report generation
Fixture loading
Profile selection
```

Conformance Specification вимагає перевіряти ontology, Core/ABI, protocols, CSU contracts, failure-to-evidence behavior та Operational/Evolution separation. 

---

## 6.9 `aira-node`

Відповідальність:

```text
Local node binary
Core runtime assembly
Local storage setup
CSU autoload
HTTP API
```

---

## 6.10 `aira-cli`

Відповідальність:

```text
CLI commands
Local node control
Schema validation command
Problem submission command
Conformance command
```

---

# 7. First Schemas to Commit

Minimum schema files:

```text
schemas/common/ref.schema.json
schemas/common/hash.schema.json
schemas/common/signature.schema.json
schemas/common/scope-descriptor.schema.json

schemas/core/object-descriptor.schema.json
schemas/core/problem-statement.schema.json

schemas/artifact/artifact-descriptor.schema.json
schemas/artifact/context-artifact.schema.json

schemas/event/event-descriptor.schema.json

schemas/policy/policy-query.schema.json
schemas/policy/policy-decision.schema.json

schemas/capability/capability-descriptor.schema.json
schemas/csu/manifest.schema.json
schemas/execution/capsule.schema.json
schemas/evidence/evidence-artifact.schema.json
schemas/result/verified-result-artifact.schema.json
schemas/conformance/report.schema.json
```

Do not add schemas for:

```text
gpu
node
driver
scheduler
blockchain
wallet
llm_model
global_state
```

as required Core structures.

---

# 8. First Fixtures

## 8.1 Valid Fixtures

```text
fixtures/valid/problem-statement.calculate-2-plus-2.json
fixtures/valid/context-artifact.local-math.json
fixtures/valid/execution-capsule.math-eval.json
fixtures/valid/output-artifact.number-4.json
fixtures/valid/verified-result.calculate-2-plus-2.json
fixtures/valid/event.problem-submitted.json
fixtures/valid/event.result-published.json
fixtures/valid/policy.allow-local.json
fixtures/valid/csu-manifest.context-basic.json
```

## 8.2 Invalid Fixtures

```text
fixtures/invalid/object-missing-signature.json
fixtures/invalid/artifact-mutated-content-hash.json
fixtures/invalid/event-invalid-signature.json
fixtures/invalid/csu-manifest-missing-abi-version.json
fixtures/invalid/verified-result-missing-evidence.json
fixtures/invalid/policy-decision-invalid-value.json
```

## 8.3 Flow Fixtures

```text
fixtures/flows/calculate-2-plus-2.flow.json
fixtures/flows/ready-solution-reuse.flow.json
fixtures/flows/failure-to-evidence.flow.json
```

---

# 9. First CLI Surface

Initial commands:

```bash
aira init
aira identity create
aira schema list
aira schema validate --schema <schema-id> --file <path>
aira csu list
aira csu register <manifest>
aira problem submit --text "Calculate 2 + 2"
aira problem status <problem-ref>
aira result get <result-ref>
aira artifact get <artifact-ref>
aira event tail
aira conformance run --profile C0
aira conformance run --profile C1
```

Commands explicitly not included:

```bash
aira federation join
aira crp route
aira settlement submit
aira research run
aira gpu register
aira model serve
```

---

# 10. Local Node State Layout

After `aira init`:

```text
.aira/
 ├── config.yaml
 ├── identity/
 │   ├── local.identity.json
 │   └── local.ed25519
 │
 ├── db/
 │   └── aira.sqlite
 │
 ├── artifacts/
 │   └── sha256/
 │
 ├── csu/
 │   └── registry/
 │
 ├── events/
 │   └── event-log.sqlite
 │
 └── conformance/
     └── reports/
```

Default config:

```yaml
node:
  mode: local
  profile: C1

security:
  allow_network_for_csu: false
  allow_shell_for_csu: false
  require_signed_artifacts: true
  require_signed_events: true
  require_signed_csu_manifests: true

storage:
  object_store: sqlite
  event_log: sqlite
  artifact_store: filesystem

csu:
  autoload:
    - context-basic
    - reduction-basic
    - execution-basic
    - verification-basic
    - evidence-basic
    - artifact-basic
```

---

# 11. First End-to-End Test

Test name:

```text
e2e_calculate_2_plus_2_produces_verified_result
```

Expected event chain:

```text
ProblemSubmitted
ContextResolved
ReductionCompleted
CapsuleCreated
CapsuleCompleted
VerificationCompleted
ResultPublished
FailureEvidenceCreated? = false
EvidenceCreated
```

Expected artifacts:

```text
Problem Statement Object
Context Artifact
Execution Capsule
Output Artifact
Verified Result Artifact
Evidence Artifact
```

Expected result:

```json
{
  "verification_status": "VERIFIED",
  "confidence": 1.0,
  "scope": {
    "scope_type": "local"
  }
}
```

Hard assertions:

```text
all descriptors schema-valid;
all artifacts immutable;
all events have causal refs where applicable;
result has evidence_refs;
result has provenance_refs;
no shell execution;
no network access;
no GPU/LLM/blockchain dependency.
```

---

# 12. Initial CI Pipeline

`.github/workflows/ci.yml`:

```yaml
name: ci

on:
  pull_request:
  push:
    branches: [main, develop]

jobs:
  rust:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Format
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Test
        run: cargo test --workspace

      - name: Schema fixtures
        run: cargo run -p aira-cli -- schema validate --fixtures fixtures

      - name: Conformance C0
        run: cargo run -p aira-cli -- conformance run --profile C0

      - name: Conformance C1
        run: cargo run -p aira-cli -- conformance run --profile C1
```

---

# 13. Security CI

`.github/workflows/security.yml`:

```yaml
name: security

on:
  pull_request:
  push:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install cargo-deny
        run: cargo install cargo-deny

      - name: Check dependencies
        run: cargo deny check
```

Security baseline:

```text
no unsigned CSU accepted;
no unsigned Artifact accepted;
no raw private payload in Events;
no shell execution by default;
no network access by default;
no secret material in logs.
```

---

# 14. Coding Standards

## 14.1 General

```text
small modules
explicit types
no hidden global state
no shared mutable state across CSU
no panics in library code
no unwrap/expect outside tests and bootstrap-only paths
errors must be typed
schemas must be tested
failure must be observable
```

## 14.2 Rust Rules

```text
#![forbid(unsafe_code)] for MVP crates
clippy warnings denied
public API documented
domain terms match Book 0–III
no Core dependency on CSU implementations
no circular crate dependencies
```

## 14.3 Error Handling

Use typed errors:

```rust
pub enum AiraError {
    SchemaValidationFailed,
    PolicyDenied,
    InvariantViolation,
    ArtifactInvalid,
    EventInvalid,
    CsuRegistrationFailed,
    CapabilityUnavailable,
    VerificationFailed,
}
```

Never hide errors.

Failure must become:

```text
Failure Event
and, when applicable, Evidence Artifact
```

---

# 15. Dependency Direction

Allowed dependency direction:

```text
aira-schema
  ↑
aira-object
aira-event
aira-artifact
aira-policy
  ↑
aira-core
  ↑
aira-csu
  ↑
aira-node
aira-cli
aira-conformance
```

Forbidden:

```text
aira-core → context-basic
aira-core → execution-basic
aira-core → verification-basic
aira-object → aira-node
aira-event → aira-node
any core crate → specific CSU implementation
```

CSU are plugins, not Core dependencies.

---

# 16. Branching Model

```text
main
  stable, always passing C0/C1

develop
  integration branch

feature/*
  normal implementation work

schema/*
  schema changes

conformance/*
  test harness changes

rfc/*
  accepted RFC implementation

experiment/*
  non-reference experiments, never merged into main without RFC
```

Rules:

```text
main must always pass;
develop should pass all tests before merge;
feature branches require tests;
schema changes require fixture updates;
conformance changes require expected-result documentation;
experiment branches cannot change Book 0–III behavior.
```

---

# 17. Commit Convention

Use Conventional Commits.

Allowed types:

```text
feat
fix
docs
test
refactor
chore
ci
schema
conformance
spec
rfc
security
```

Examples:

```text
spec(book0): add canonical foundations draft
schema(core): add object descriptor schema
feat(object): implement immutable object store
test(event): add causal chain fixture
conformance(c0): validate object immutability
security(policy): deny private artifact access by default
```

Forbidden commit styles:

```text
update stuff
misc fixes
wip
final
new version
```

---

# 18. Pull Request Rules

Each PR must state:

```text
Problem
Scope
Affected crates
Affected schemas
Affected tests
Compatibility impact
Security/privacy impact
Conformance impact
```

PR template:

```markdown
## Problem

## Scope

## Non-Goals

## Changes

## Tests

## Schema Impact

## Conformance Impact

## Security / Privacy Impact

## Checklist

- [ ] No Core pollution
- [ ] No forbidden ontology regression
- [ ] Schemas updated if needed
- [ ] Fixtures updated if needed
- [ ] C0/C1 tests pass
- [ ] Failure path observable
```

---

# 19. MVP Issue Breakdown

## Epic 0 — Repository Bootstrap

```text
#1  Initialize repository
#2  Add license, README, CONTRIBUTING, SECURITY
#3  Add Cargo workspace
#4  Add rust-toolchain / rustfmt / clippy config
#5  Add initial CI
```

## Epic 1 — Specification Snapshot

```text
#6  Add Book 0–V to specs/
#7  Add Specification Control Document
#8  Add Conformance Specification
#9  Add Schema Pack
#10 Add RFC Process
#11 Add MVP Roadmap
#12 Add Repository Bootstrap Plan
```

## Epic 2 — Schema Registry

```text
#13 Add common schemas
#14 Add core object schema
#15 Add artifact schema
#16 Add event schema
#17 Add policy schemas
#18 Add capability schema
#19 Add CSU manifest schema
#20 Add execution capsule schema
#21 Add verified result schema
#22 Add conformance report schema
#23 Add valid/invalid fixtures
#24 Implement aira-schema crate
#25 Add schema validate CLI
```

## Epic 3 — Core Runtime C0

```text
#26 Implement handle type
#27 Implement object descriptor
#28 Implement immutable object store
#29 Implement artifact descriptor
#30 Implement content-addressed artifact store
#31 Implement event descriptor
#32 Implement append-only event log
#33 Implement local event subscriptions
#34 Implement policy query/decision
#35 Implement policy gate
#36 Implement invariant checker
#37 Implement C0 conformance tests
```

## Epic 4 — CSU Runtime C1

```text
#38 Implement CSU manifest type
#39 Implement CSU registry
#40 Implement CSU lifecycle states
#41 Implement CSU trait
#42 Implement event dispatch to CSU
#43 Implement CSU output handling
#44 Implement CSU failure events
#45 Implement Context CSU
#46 Implement Reduction CSU
#47 Implement Execution CSU
#48 Implement Verification CSU
#49 Implement Evidence CSU
#50 Implement Artifact CSU
```

## Epic 5 — Operational Flow

```text
#51 Implement problem submit
#52 Implement context creation flow
#53 Implement execution capsule flow
#54 Implement safe math.eval execution
#55 Implement output artifact publication
#56 Implement verification flow
#57 Implement result publication
#58 Implement evidence creation
#59 Implement ready solution reuse demo
#60 Implement failure-to-evidence demo
```

## Epic 6 — CLI / Node

```text
#61 Implement aira init
#62 Implement identity create
#63 Implement csu list/register
#64 Implement problem submit
#65 Implement problem status
#66 Implement result get
#67 Implement artifact get
#68 Implement event tail
#69 Implement conformance run
#70 Implement local node binary
```

## Epic 7 — Conformance

```text
#71 Implement C0 conformance runner
#72 Implement C1 conformance runner
#73 Add conformance report artifact
#74 Add object immutability test
#75 Add artifact immutability test
#76 Add event causality test
#77 Add policy gate test
#78 Add CSU manifest test
#79 Add verified result completeness test
#80 Add failure-to-evidence test
```

---

# 20. First Release Scope

Release:

```text
aira-mvp-v0.1.0-alpha.1
```

Must include:

```text
buildable workspace
schema validation
local object store
local artifact store
local event log
policy gate
invariant checker
CSU registry
basic CSU set
CLI
calculate 2 + 2 demo
ready solution reuse demo
failure-to-evidence demo
C0 conformance
C1 conformance
```

Must not include:

```text
federation
networked CRP
GPU execution
LLM inference
blockchain
production security
research plane
```

---

# 21. README Required Content

README must open with:

```text
AIRA is a minimal, event-native, artifact-based, CSU-driven system for transforming a Problem Statement into a Verified Result Artifact with minimal necessary resources.
```

README must explicitly state:

```text
AIRA is not a GPU marketplace.
AIRA is not an LLM runtime.
AIRA is not a blockchain network.
AIRA is not a scheduler.
AIRA Core does not solve tasks.
AIRA Core enables compatible CSU to solve tasks through stable contracts.
```

This prevents regression into the early P2P LLM/GPU architecture, which was superseded by the canonical AIRA model.  

---

# 22. First Demo Script

```bash
cargo build --workspace

aira init

aira identity create --name local-user

aira csu register csu/context-basic/manifest.json
aira csu register csu/reduction-basic/manifest.json
aira csu register csu/execution-basic/manifest.json
aira csu register csu/verification-basic/manifest.json
aira csu register csu/evidence-basic/manifest.json
aira csu register csu/artifact-basic/manifest.json

aira problem submit --text "Calculate 2 + 2"

aira event tail

aira result get <result-ref>

aira conformance run --profile C0
aira conformance run --profile C1
```

Expected proof:

```text
all events persisted;
all artifacts immutable;
result verified;
evidence present;
conformance passed;
no external compute required.
```

---

# 23. Repository Invariant

The repository itself must obey AIRA principles:

```text
No hidden state.
No undocumented behavior.
No untestable contracts.
No mutable canonical artifacts.
No implementation-specific concept in Core.
No research behavior in Operational path.
No complexity without measurable benefit.
```

---

# 24. Bootstrap Acceptance Criteria

Repository bootstrap is complete when:

```text
1. Repository builds from clean checkout.

2. CI runs format, clippy, tests, schema validation, C0/C1 conformance.

3. specs/ contains Book 0–V and control documents.

4. schemas/ contains minimum Schema Pack.

5. fixtures/ contains valid and invalid examples.

6. crates/ contains all MVP crates.

7. csu/ contains basic CSU manifests and implementations.

8. `aira init` creates local state.

9. `aira problem submit --text "Calculate 2 + 2"` produces Verified Result Artifact.

10. `aira conformance run --profile C0` passes.

11. `aira conformance run --profile C1` passes.

12. No ML/GPU/blockchain/network dependency is required.
```

---

# 25. Status

AIRA Repository Bootstrap Plan v0.1 defines the first practical engineering step after the MVP roadmap.

It converts the standard corpus into a buildable repository structure while preserving the central goal:

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
AIRA Initial Issue Set v0.1
```

Scope:

```text
GitHub-ready issue titles;
issue descriptions;
labels;
dependencies;
acceptance criteria;
milestone assignment;
implementation order;
first PR sequence.
```
