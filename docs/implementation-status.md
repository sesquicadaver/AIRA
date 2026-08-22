# Implementation status

**Status (Analyze-87 / QUEUE #52):** map of what this repository implements versus Book 0–IV, Schema Pack, Conformance, and the basic CSU set. This is **not** a new architecture and **does not** add code to fill gaps.

```text
Requirement → Source spec → Implemented in → Tested by → Status → Notes
```

HTTP, peer/DHT/relay/gossip, and federation join exist in-tree and are labelled **POST-MVP** (outside C0/C1 MVP contract). Book V is **RESEARCH** and is not a Phase C delivery.

## Status values

| Status | Meaning |
|--------|---------|
| **DONE** | Present in the reference tree and covered by named tests or CI |
| **PARTIAL** | Present; incomplete versus the Book MUST |
| **STUB** | Heuristic / placeholder; must not be read as a full contract |
| **POST-MVP** | In-tree beyond local C0/C1 MVP; not the normative gate |
| **ABSENT** | Not implemented in this reference |
| **RESEARCH** | Book V / isolated track; not operational |

Living per-cycle matrices remain under `analysis/Analyze-N/LIVING_SPEC_MATRIX.md`. This page is the repo-level rollup.

Operator entry: [README](../README.md) → [specs/](../specs/) → this file → [demo.md](demo.md). Plane role: [operational-plane.md](operational-plane.md).

---

## Book 0 — foundations

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Canonical ontology (Problem, Context, Evidence, Capsule, Artifact, Event, Policy, CSU, VRA, …) | Book 0 §3; Conformance B0-001 | `aira-object`, `aira-event`, `aira-artifact`, `aira-schema`, schemas under `schemas/` | `c0.ontology.schemas` | **DONE** | Schema presence + example Problem descriptor |
| Forbidden Core entities (GPU/LLM/Node/Driver/Scheduler/…) | Book 0 §3.2; B0-002 | Schema + fixture reject | `fixtures/invalid/core/object-descriptor-gpu.json`; `scripts/dep_firewall.py` | **DONE** | Core must not import node/peer/concrete CSU |
| Operational pipeline PS → Interpret → Reduce → Materialize → Verify → VRA | Book 0 §4; B0-003; OP-001 | `aira_flow::OperationalPlane` (C1 **reference/demo**) | `c1.pipeline.calculate_2_plus_2`; `aira-flow` `calculate_two_plus_two_demo` | **DONE** | Plane is not production event/scheduler/federation runtime |
| Human Final Collapse / no silent pick | Book 0 A6; B0-004 | `is_normative_split` string heuristic + OperationalArtifact | `normative_split_stub_does_not_autocollapse` | **STUB** | Not a full Differentiated Solution Field CSU |
| Evidence primacy | Book 0 A5; B0-005 | `csu/evidence-basic`; failure evidence path | `c1.failure.to_evidence`; `failure_creates_failure_evidence` | **PARTIAL** | Failure→evidence for C1; no standalone Claim/Assumption suite |
| Epistemic Status as distinct coordinate | Book 0 §6 | `schemas/evidence/epistemic-assessment.schema.json` | schema load only (no fixture in manifest) | **PARTIAL** | No Epistemic CSU (`csu/` has no epistemic crate) |
| Evolution / Research plane | Book 0 §10; Book V | — | — | **RESEARCH** | Isolated; not mixed into Core |
| Cost / escalation / Goal Compiler | Book 0 §8–9; Book V | — | — | **RESEARCH** | Out of MVP |

---

## Book I — Core / ABI

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Immutable Object Store | Book I §4–5; B1-001 | `aira-core` `MemoryObjectStore` / `SqliteObjectStore` | `c0.object.immutability`; `memory_rejects_in_place_mutation`; `create_rejects_unsigned_and_mutated_object_signature` | **DONE** | Canonical object signatures (QUEUE #42) |
| Opaque Handle | Book I §6; B1-003 | `aira_object::Handle` | crate Debug omits paths; `memory_create_open` | **PARTIAL** | `storage_token` is an internal id, not a path; no dedicated CSU “infer path” attack test |
| Event runtime, local causal order, no global total order | Book I §8–9; B1-004/005 | `aira-event` `MemoryEventLog` | `c0.event.causality`; `aira-event` unit tests | **DONE** | Plane drain is in-process demo (`drain_from` bound 256) |
| Durable event log | Book I / Book IV §6.3 | `LocalSession` → `.aira/events/event-log.json` | `local_init_submit_status_and_artifact`; alpha layout | **PARTIAL** | JSON file, not a production event engine |
| Policy Gate ALLOW/DENY/REQUIRE | Book I §10; B1-006/007 | `aira-policy` | `c0.policy.gate`; `invariant_checker_emits_event_on_policy_deny` | **DONE** | |
| Invariant Checker can block | Book I §11; B1-008 | `aira-core` `InvariantChecker` | C0 immutability cases | **DONE** | |
| Security boundary (no Core memory / foreign CSU / secrets) | Book I §12; B1-009; B3-004 | `aira-csu` isolation + event secret scan | `isolation_baseline_denies_direct_mutation_and_peer_call`; `run_security_baseline` | **DONE** | MVP sandbox: in-process, no FS/net |
| Artifact runtime + immutability | Book I §16; B1-002 | `aira-artifact` CAS | `c0.artifact.immutability`; `mutation_fails_supersession_keeps_old`; `unsigned_artifact_rejected_and_private_denied` | **DONE** | Canonical artifact signatures (QUEUE #41) |
| Verified Result completeness | Book I §17; B1-010 | verification-basic + plane | `c1.result.verified_completeness` | **PARTIAL** | C1 checks result/verification_status/confidence/evidence_refs/provenance_refs; not every Book field name 1:1 |
| CSU runtime + lifecycle | Book I §13–14 | `aira-csu` `CsuRuntime` / registry | `lifecycle_transitions_and_events`; `dispatch_active_only_and_failure_event` | **DONE** | |
| Canonical descriptor signatures | Book I security; EVO-2 | `aira_object::canonical` + Event/Artifact/Object/CSU | mutation tests in those crates | **DONE** | QUEUE #39–#44; no runtime `LOCAL_TEST_DOMAIN_MSG` fallback |
| Federated Core (C2+/C3) | Book I §22 L2 | `aira-protocol`, `aira-peer`, federation join | C2 local; `federation::join_*` | **POST-MVP** | Not C0/C1 CI |

---

## Book II — protocols

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Common envelope + signature | Book II §6; B2-001/002 | `aira-protocol` + schemas | `c2.protocol.envelope_schema`; `envelope_rejects_local_test_domain_fallback` | **PARTIAL** | Local C2; not a wire network |
| Unsupported version without side effects | B2-003 | `EventProtocolAdapter` | `c2.ep.unsupported_version_no_side_effects` | **PARTIAL** | Local adapter |
| Event Protocol publish idempotency | Book II §12; B2-008 | `aira-protocol` event adapter | `event_protocol_publish_idempotent_and_unsupported_version` | **PARTIAL** | In-process |
| Artifact Protocol publish/resolve + hash | Book II §11; B2-007 | `aira-protocol` artifact adapter | `artifact_protocol_publish_resolve_and_hash_check` | **PARTIAL** | In-process |
| Identity descriptor | Book II §13 | schema + `aira-object` keyring | `c2.identity.descriptor_schema`; `aira-object` crypto/keyring tests | **PARTIAL** | Local identity; rotation is node/tenant CLI |
| Discovery by Capability, not Node | Book II §8; B2-004 | `DiscoveryRegistry` | `c2.discovery.capability_not_node`; `discovery_returns_capability_not_node` | **PARTIAL** | Local registry file |
| Capability Advertisement | Book II §9; B2-005 | capability schema + HTTP `GET /v1/capabilities` | schema fixtures; HTTP tests | **PARTIAL** | No network advertisement protocol |
| CRP | Book II §10; B2-006 | — | — | **ABSENT** | Explicit Phase C Out |
| Federation protocol (full) | Book II §14; B2-010 | join prototype only | `aira-protocol` `federation::join_*` | **POST-MVP** | Local pin + trust; no leave/Join Request/CRP |
| Settlement / Audit protocol | Book II §15; B2-011 | — | — | **ABSENT** | Trust audit JSONL is node identity, not settlement receipts |
| C2 profile in CI | Conformance C2 | `run_c2` exists | local `aira-conformance` tests; **not** GitHub Actions | **PARTIAL** | QUEUE #38 Out: C2 not required CI |

---

## Book III — CSU contracts (basic set)

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Signed CSU manifest | Book III §5; B3-001/002 | `aira-csu` + `schemas/csu/manifest.schema.json` | `c1.csu.manifests`; `manifest_schema_valid_and_unsigned_rejected`; `canonical_verify_fails_when_manifest_fields_change` | **DONE** | |
| No direct CSU→CSU call | B3-004; firewall | `csu/*` crates; `scripts/dep_firewall.py` | firewall CI; `isolation_baseline_*` | **DONE** | |
| Failure Event + Evidence | B3-007/008; FAIL-* | evidence-basic + plane inject | `c1.failure.to_evidence`; `failure_creates_failure_evidence` | **DONE** | C1 SHOULD evidence: implemented |
| Context CSU (no final Result) | CTX-001 | `csu/context-basic` | `problem_submitted_creates_context_not_result` | **DONE** | |
| Reduction / reuse before execute | RED-001; OP-002 | `csu/reduction-basic` | `ready_solution_reuse_skips_execution`; `creates_negative_lookup_and_capsule_when_no_reuse` | **DONE** | |
| Execution CSU authorized capsules | EXE-001 | `csu/execution-basic` | `math_eval_safe_completes`; `rejects_shell_action` | **DONE** | Safe math/text only |
| Verification CSU | VER-001 | `csu/verification-basic` | `verifies_math_output_as_verified_result` | **DONE** | |
| Evidence CSU | EVD-001 | `csu/evidence-basic` | crate + C1 failure path | **DONE** | Does not assign Epistemic Status |
| Artifact CSU publish/resolve/supersede | ART-001 | `csu/artifact-basic` | `publish_resolve_supersede_events` | **DONE** | |
| Epistemic CSU | EPI-001; Book III §16; Book IV §13.7 | schema only | — | **ABSENT** | Not in `csu/` |
| Discovery / Federation / Settlement / Optimization / PHM / Evolution / Research / Human Interaction CSUs | Book III §20–27 | — | — | **ABSENT** / **RESEARCH** | Human collapse is plane STUB, not HUM CSU |
| Durable per-CSU tenant signing | Book III identity; crypto docs | `aira-object` `tenant/` | tenant module tests; CLI tenant_secret tests | **POST-MVP** | Beyond C1 basic set |

---

## Book IV — reference implementation

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| R0 Minimal Local Core | Book IV §23 R0 | `aira-core`, `aira-event`, `aira-artifact`, `aira-policy`, `aira-schema` | C0 + crate tests | **DONE** | |
| R1 Minimal Operational Node | Book IV §23 R1 | `aira-flow`, `aira-cli`, basic CSUs, `.aira` layout | C1; `local_init_submit_status_and_artifact`; `run_alpha_acceptance` | **DONE** | `config.node.profile = "C1"` |
| R2 Local Protocol Node | Book IV §23 R2 | `aira-protocol` | `run_c2` (local) | **PARTIAL** | Partial C2; not CI-gated |
| R3 Federation-Capable Node | Book IV §23 R3 | join prototype | federation tests | **POST-MVP** | Not a federation runtime |
| R4 Research-Capable Node | Book IV §23 R4 | — | — | **RESEARCH** | Book V |
| Object / Artifact / Event stores | Book IV §6 | memory + SQLite objects; CAS artifacts; memory+JSON events | core/artifact/flow tests | **PARTIAL** | Plane itself uses memory object/event stores |
| Policy + Invariant + CSU runtime | Book IV §9–12 | matching crates | C0/C1 | **DONE** | |
| Minimal operational flow submit→verify | Book IV §15 | `OperationalPlane` | C1 + demos | **DONE** | Reference-local only ([operational-plane.md](operational-plane.md)) |
| Local HTTP API surface | Book IV §16 | `aira-node` `http/` | `http` module tests | **POST-MVP** | Roadmap M11; same reference plane |
| CLI (`init`, identity, csu, problem, result, artifact, event, conformance) | Book IV §17 | `aira-cli` `commands/` | CLI tests; alpha | **DONE** | Extra post-MVP: peer, federation, tenant |
| Security defaults (unsigned reject, private deny) | Book IV §19 | artifact + CSU + events | `run_security_baseline` | **DONE** | |
| Conformance harness + report artifact | Book IV §21–22 | `aira-conformance` | C0/C1 CI; report schema | **DONE** | C2 optional |
| Suggested crate layout | Book IV §25–26 | `crates/`, `csu/` | firewall | **DONE** | |
| `OperationalPlane` ≠ production runtime | Book IV + EVO-2 §3.3 | rustdoc + docs | docs review Analyze-86 | **DONE** | QUEUE #51 |

---

## Schema Pack

CI: `cargo run -p aira-cli -- schema validate --fixtures fixtures` (QUEUE #38).

| Schema `$id` | File | Fixture in `fixtures/manifest.json` | Status |
|--------------|------|--------------------------------------|--------|
| `aira:schema:common:identifier:0.1` | `schemas/common/identifier.schema.json` | file exists, **not** listed in manifest | **PARTIAL** |
| `aira:schema:common:ref:0.1` | `common/ref.schema.json` | valid + invalid | **DONE** |
| `aira:schema:common:hash:0.1` | `common/hash.schema.json` | valid + invalid | **DONE** |
| `aira:schema:common:timestamp:0.1` | `common/timestamp.schema.json` | valid | **DONE** |
| `aira:schema:common:signature:0.1` | `common/signature.schema.json` | valid | **DONE** |
| `aira:schema:common:scope-descriptor:0.1` | `common/scope-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:core:object-descriptor:0.1` | `core/object-descriptor.schema.json` | valid + GPU invalid | **DONE** |
| `aira:schema:core:problem-statement:0.1` | `core/problem-statement.schema.json` | valid | **DONE** |
| `aira:schema:artifact:artifact-descriptor:0.1` | `artifact/artifact-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:artifact:context-artifact:0.1` | `artifact/context-artifact.schema.json` | **no** manifest entry | **PARTIAL** |
| `aira:schema:event:event-descriptor:0.1` | `event/event-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:policy:query:0.1` | `policy/query.schema.json` (+ duplicate `policy-query.schema.json`) | valid | **DONE** |
| `aira:schema:policy:decision:0.1` | `policy/decision.schema.json` (+ duplicate `policy-decision.schema.json`) | valid | **DONE** |
| `aira:schema:capability:descriptor:0.1` | `capability/capability-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:csu:manifest:0.1` | `csu/manifest.schema.json` | valid + unsigned invalid | **DONE** |
| `aira:schema:execution:capsule:0.1` | `execution/capsule.schema.json` | valid | **DONE** |
| `aira:schema:evidence:evidence-artifact:0.1` | `evidence/evidence-artifact.schema.json` | valid | **DONE** |
| `aira:schema:epistemic:assessment:0.1` | `evidence/epistemic-assessment.schema.json` | **no** manifest entry | **PARTIAL** |
| `aira:schema:result:verified-result-artifact:0.1` | `result/verified-result-artifact.schema.json` | valid + missing-evidence invalid | **DONE** |
| `aira:schema:conformance:report:0.1` | `conformance/report.schema.json` | valid | **DONE** |
| `aira:schema:protocol:envelope:0.1` | `protocol/envelope.schema.json` | valid + unsigned invalid | **DONE** |
| `aira:schema:protocol:response:0.1` | `protocol/response.schema.json` | valid | **DONE** |
| `aira:schema:identity:identity-descriptor:0.1` | `identity/identity-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:desktop:settings:0.1` | `desktop/settings.schema.json` | valid + missing-instance-id invalid | **DONE** |

Duplicate policy schema files share the same `$id`; they are not two contracts.

Conformance spec §19 lists extra fixture *names* (event chain, policy deny, research unpromoted, …) that are **not** all present as standalone JSON files; coverage is via C0/C1 runners instead.

---

## Conformance profiles

| Profile | Spec | Runner | Cases | CI | Status |
|---------|------|--------|-------|----|--------|
| C0 | Local Core | `run_c0` | `c0.ontology.schemas`, `c0.object.immutability`, `c0.artifact.immutability`, `c0.event.causality`, `c0.policy.gate` | **yes** (QUEUE #38) | **DONE** |
| C1 | Local operational node | `run_c1` | `c1.pipeline.calculate_2_plus_2`, `c1.csu.manifests`, `c1.result.verified_completeness`, `c1.failure.to_evidence` | **yes** | **DONE** |
| C2 | Protocol-compatible | `run_c2` | envelope/response/identity schemas; discovery ≠ Node; UNSUPPORTED_VERSION | **no** | **PARTIAL** |
| C3–C5 | Federated / settlement / research | — | — | no | **ABSENT** / **RESEARCH** |
| Security baseline | Conformance §14 subset | `run_security_baseline` | unsigned CSU/artifact; private deny; secret in events | crate tests | **DONE** |
| Alpha acceptance | MVP | `run_alpha_acceptance` | init layout, 2+2, failure evidence, C0/C1 | crate tests | **DONE** |

C0/C1 are a **minimal** encoding of B0/B1/OP/CSU MUST tests, not a 1:1 clone of every Conformance `B*-*` id. Gaps (handle-opacity attack, full VRA field list, B0-005 Claim artifacts, B2 network) stay documented here; filling them is out of this row.

---

## HTTP and peer (post-MVP)

Not part of C0/C1. Do not treat as production distributed runtime.

| Surface | Implemented in | Tested by | Status | Notes |
|---------|----------------|-----------|--------|-------|
| Local HTTP (`/health`, `/v1/problems`, results, artifacts, events, capabilities, CSU, conformance/run) | `crates/aira-node/src/http/` | `http` tests | **POST-MVP** | Book IV §16 surface; loopback default; `--allow-public-bind` |
| TLS / mTLS CN→TrustStore / separate health | `crates/aira-node/src/tls/` | `tls` tests; `main` health-listen tests | **POST-MVP** | |
| Bearer + multi-tenant CSU authz | `http/auth.rs` + tenant map | HTTP tests | **POST-MVP** | |
| Peer listen, Noise, trust-delta, gossip | `aira-peer` | `aira-peer` tests | **POST-MVP** | |
| Relay hub registry | `aira-peer` `relay` | relay restart tests | **POST-MVP** | |
| DHT announce / FIND_NODE / STUN | `aira-peer` `dht` / `discv` / `stun` | module tests | **POST-MVP** | Prototypes, not discv5 production |
| Federation join pin | `aira-protocol` federation + CLI | `join_*` tests | **POST-MVP** | No settlement/CRP |

---

## Phase C gates (already DONE)

| QUEUE | Gate | Evidence |
|-------|------|----------|
| #38 | Schema fixtures + C0 + C1 in GitHub Actions | `.github/workflows/ci.yml` |
| #39–#44 | Canonical descriptor hash / signatures / leftover test fallback | crate mutation tests |
| #45 | `aira-core` ↛ node/peer/CSU; CSU ↛ CSU; no import cycles | `scripts/dep_firewall.py` |
| #46–#50 | Mechanical splits CLI/crypto/tenant/http/tls | crate tests |
| #51 | OperationalPlane reference-local | [operational-plane.md](operational-plane.md) |
| #52 | This matrix | this file |

---

## Explicitly not this reference

```text
production event runtime / scheduler / distributed plane
CRP, settlement ledger, PHM, HIE, KnowledgeOps, Goal Compiler, DSM
GPU/LLM marketplace, blockchain-required settlement
CSU SDK / new basic CSU roles to “complete” Book III
C2 as merge-blocking CI
```

Model layer (EVO-3): D0–D7 `#53`–`#74` **DONE** @ d270b62. Not Core. Plan: [phase-d-plan.md](phase-d-plan.md).

Desktop one-click / GUI: `#75`–`#79` **DONE**. E1.1 `#80`–`#85` **DONE**. E2: `#86`–`#88` **DONE**; macOS docs `#89` OPEN.
