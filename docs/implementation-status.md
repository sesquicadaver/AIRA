# Implementation status

**Status:** **Reference v0.3-strict** (Analyze-243 / QUEUE `#208`; Phase J `#199`–`#208` **DONE** @ RFC-0096). Phase I **Reference v0.3-stable** (`#184`–`#198` **DONE** @ RFC-0078) is the prior semantic-contract posture. Phase H **Reference v0.3** (`#152`–`#183` **DONE** @ RFC-0077) is the prior protocol-depth posture. Phase G **Reference v0.2** (`#120`–`#151` **DONE** @ RFC-0069) is the prior posture. Map of what this repository implements versus Book 0–IV, Schema Pack, Conformance, and the basic CSU set. This is **not** a new architecture and **does not** add code to fill gaps beyond the active QUEUE atom.

**Navigation:** [`docs/README.md`](README.md) · **Queue:** [`QUEUE.md`](../QUEUE.md) (Phase N `#231`–`#239` **DONE**; first OPEN `#240`; Phase M `#224`–`#230` **DONE** @ RFC-0117; QUEUE M closed; L `#217`–`#223` **DONE** @ RFC-0111; QUEUE L closed; K `#209`–`#216` **DONE** @ RFC-0104; QUEUE K closed) · **Phase H plan:** [`phase-h-plan.md`](phase-h-plan.md) · **RFC-P:** [`rfc-p-promotion.md`](rfc-p-promotion.md) · **Phase I:** [`phase-i-plan.md`](phase-i-plan.md) · **Phase J:** [`phase-j-plan.md`](phase-j-plan.md) · **Phase K:** [`phase-k-plan.md`](phase-k-plan.md) · **Phase L:** [`phase-l-plan.md`](phase-l-plan.md) · **Phase M:** [`phase-m-plan.md`](phase-m-plan.md) · **Phase N:** [`phase-n-plan.md`](phase-n-plan.md) · **Phase G:** [`phase-g-plan.md`](phase-g-plan.md) · **RFC:** [`AIRA-RFC-0069`](../specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md) (G); [`AIRA-RFC-0077`](../specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md) (H); [`AIRA-RFC-0078`](../specs/rfc/AIRA-RFC-0078-phase-i-semantic-stabilization.md) (I); [`AIRA-RFC-0096`](../specs/rfc/AIRA-RFC-0096-phase-j-book-gap-local.md) (J); [`AIRA-RFC-0104`](../specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md) (K); [`AIRA-RFC-0111`](../specs/rfc/AIRA-RFC-0111-phase-l-execution-boundary.md) (L); [`AIRA-RFC-0117`](../specs/rfc/AIRA-RFC-0117-phase-m-os-child-sandbox.md) (M); [`AIRA-RFC-0097`](../specs/rfc/AIRA-RFC-0097-seal-object-store-access.md) (`#201`); [`AIRA-RFC-0098`](../specs/rfc/AIRA-RFC-0098-vra-runtime-b1-010.md) (`#202`); [`AIRA-RFC-0099`](../specs/rfc/AIRA-RFC-0099-event-log-authority.md) (`#203`); [`AIRA-RFC-0100`](../specs/rfc/AIRA-RFC-0100-reduction-catalog-bind.md) (`#204`); [`AIRA-RFC-0101`](../specs/rfc/AIRA-RFC-0101-semantic-verify-text.md) (`#205`); [`AIRA-RFC-0102`](../specs/rfc/AIRA-RFC-0102-evidence-primacy-runtime.md) (`#206`); [`AIRA-RFC-0103`](../specs/rfc/AIRA-RFC-0103-epistemic-emit-c1.md) (`#207`); [`AIRA-RFC-0105`](../specs/rfc/AIRA-RFC-0105-generate-local-payload-schema.md) (`#210`); [`AIRA-RFC-0106`](../specs/rfc/AIRA-RFC-0106-execution-llm-mock.md) (`#211`); [`AIRA-RFC-0107`](../specs/rfc/AIRA-RFC-0107-reduction-generate-local.md) (`#212`); [`AIRA-RFC-0108`](../specs/rfc/AIRA-RFC-0108-plane-register-execution-llm.md) (`#213`); [`AIRA-RFC-0109`](../specs/rfc/AIRA-RFC-0109-activate-gate.md) (`#214`); [`AIRA-RFC-0110`](../specs/rfc/AIRA-RFC-0110-process-backend.md) (`#215`); [`AIRA-RFC-0112`](../specs/rfc/AIRA-RFC-0112-activate-evidence-gate.md) (`#218`); [`AIRA-RFC-0113`](../specs/rfc/AIRA-RFC-0113-child-env-whitelist.md) (`#219`); [`AIRA-RFC-0114`](../specs/rfc/AIRA-RFC-0114-bounded-pipes.md) (`#220`); [`AIRA-RFC-0115`](../specs/rfc/AIRA-RFC-0115-problem-record-split.md) (`#221`); [`AIRA-RFC-0116`](../specs/rfc/AIRA-RFC-0116-network-none-contract.md) (`#222`); [`AIRA-RFC-0118`](../specs/rfc/AIRA-RFC-0118-landlock-fs.md) (`#225`); [`AIRA-RFC-0119`](../specs/rfc/AIRA-RFC-0119-seccomp.md) (`#226`); [`AIRA-RFC-0120`](../specs/rfc/AIRA-RFC-0120-netns.md) (`#227`); [`AIRA-RFC-0121`](../specs/rfc/AIRA-RFC-0121-sandbox-required.md) (`#228`); [`AIRA-RFC-0122`](../specs/rfc/AIRA-RFC-0122-os-vs-aira-mediated.md) (`#229`)

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

Operator entry: [README](../README.md) → [specs/](../specs/) → this file → [demo.md](demo.md). Plane role: [operational-plane.md](operational-plane.md). Conformance index: [conformance.md](conformance.md) (C0–C5).

**v0.3 protocol-depth posture (`#182`–`#183`):** durable event/object paths (H1); local C3 + optional CI job (H2); in-process CRP PARTIAL (H3); settlement audit receipts PARTIAL (H4); RFC-P + non-operational gate + `run_c5` (H5). Not a production distributed plane. Consolidating RFC: [`AIRA-RFC-0077`](../specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md).

---

## Book 0 — foundations

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Canonical ontology (Problem, Context, Evidence, Capsule, Artifact, Event, Policy, CSU, VRA, …) | Book 0 §3; Conformance B0-001 | `aira-object`, `aira-event`, `aira-artifact`, `aira-schema`, schemas under `schemas/` | `c0.ontology.schemas` | **DONE** | Schema presence + example Problem descriptor |
| Forbidden Core entities (GPU/LLM/Node/Driver/Scheduler/…) | Book 0 §3.2; B0-002 | Schema + fixture reject | `fixtures/invalid/core/object-descriptor-gpu.json`; `scripts/dep_firewall.py` | **DONE** | Core must not import node/peer/concrete CSU |
| Operational pipeline PS → Interpret → Reduce → Materialize → Verify → VRA | Book 0 §4; B0-003; OP-001 | `aira_flow::OperationalPlane` (C1 **reference/demo**) | `c1.pipeline.calculate_2_plus_2`; `aira-flow` `calculate_two_plus_two_demo`; `alloc_run_nonce_concurrent_is_unique` | **DONE** | Plane is not production event/scheduler/federation runtime; `#193` / RFC-0091 runtime Clock for operational `created_at`; `#195` / RFC-0093 UUIDv7 run nonce (no racy `run-counter`) |
| Human Final Collapse / no silent pick | Book 0 A6; B0-004 | `is_normative_split` string heuristic + OperationalArtifact | `normative_split_stub_does_not_autocollapse` | **STUB** | Not a full Differentiated Solution Field CSU |
| Evidence primacy | Book 0 A5; B0-005 | `schemas/evidence/claim-artifact.schema.json`; `csu/evidence-basic`; `OperationalPlane` inject/drain | `schema validate --fixtures` claim/assumption (#125); `claim_without_evidence_rejected_as_operational_input`; `assumption_without_evidence_is_operational_input`; `claim_with_evidence_is_operational_input`; `c1.failure.to_evidence`; `failure_creates_failure_evidence` | **DONE** | `#125` schema gate; `#206` / RFC-0102: runtime `claim_kind: Claim` without evidence → `EvidencePrimacy`. Assumption/Hypothesis may omit evidence |
| Epistemic Status as distinct coordinate | Book 0 §6 | `schemas/evidence/epistemic-assessment.schema.json`; `csu/epistemic-basic` | schema fixtures (#108); `epi_001_…`; `epistemic_assessment_roundtrip_via_plane_and_session`; `calculate_two_plus_two_emits_epistemic_assessment`; `c1.pipeline.calculate_2_plus_2` | **PARTIAL** | `#207` / RFC-0103: C1 2+2 Completed emits assessment; full Epistemic plane still out |
| Evolution / Research plane | Book 0 §10; Book V | RFC-P doc + plane gate + `run_c5` | `phase_h_rfc_p_promotion_doc`; `research_artifact_rejected_as_operational_input`; `c5.research.separation` | **RESEARCH** | Isolated; not mixed into Core; [`rfc-p-promotion.md`](rfc-p-promotion.md) (`#177`–`#181`); Book V → operational лише через promotion; gate RFC-0082; C5 RFC-0083 (not a merge gate); **H5 DONE**; no promoted research item |
| Cost / escalation / Goal Compiler | Book 0 §8–9; Book V | — | — | **RESEARCH** | Out of MVP |

---

## Book I — Core / ABI

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Immutable Object Store | Book I §4–5; B1-001 | `aira-core` `MemoryObjectStore` / `SqliteObjectStore` | `c0.object.immutability`; `c0.object.verify_on_read`; `sqlite_migrate_idempotent_reopen_preserves_rows`; `sqlite_corrupt_descriptor_json_integrity`; `init_node_sqlite_object_path_migrate_and_persist`; `plane_memory_beside_node_sqlite_object_path` (#158) | **DONE** | Verify-on-read (#112); SQLite migrate (#143); node path beside plane memory (#158) |
| Opaque Handle | Book I §6; B1-003 | `aira_object::Handle` | `c0.object.handle_opacity`; `handle_is_opaque`; `object_store_access_is_not_in_the_default_prelude`; `store_backend_feature_is_only_enabled_by_aira_core`; `csu_sources_do_not_import_object_store_access`; `handle_forged_unknown_token_open_fails`; `handle_cross_object_token_bind_rejects`; `handle_cross_store_open_fails` | **DONE** | `#186` / RFC-0084: `Handle::new` / `storage_token()` are `pub(crate)`; `open` bind `object_id == handle.object_ref`. `#201` / RFC-0097: `object_store_access` is feature `store-backend` (`aira-core` only); not on the default `aira-object` prelude; C0 opacity uses `ObjectStore::create` |
| Event runtime, local causal order, no global total order | Book I §8–9; B1-004/005 | `aira-event` `MemoryEventLog` + `EventHashChain` (#154) | `c0.event.causality`; `event_log_hash_chain_tip_append_verify_and_mid_tamper_detect`; `aira-event` unit tests | **DONE** | Plane drain in-process (`drain_from` 256); tip/mid-tamper fail-closed (#154) |
| Durable event log | Book I / Book IV §6.3 | `LocalSession` → `file-chain-log.json` (#157) + legacy `event-log.json`; `FileChainEventLog` (#156); prefix recovery (#155) | `session_durable_file_chain_roundtrip`; `event_tail_after_reopen_reads_file_chain_not_memory_or_legacy`; `corrupt_event_log_recovered_and_writable`; `corrupt_trailing_event_log_recovers_valid_prefix`; `file_chain_event_log_persists_across_reopen` | **DONE** | `#203` / RFC-0099: `event_tail` fail-closed on file-chain; legacy JSON is recovery helper only; plane `drain_from` stays memory (C1 reference) |
| Policy Gate ALLOW/DENY/REQUIRE | Book I §10; B1-006/007 | `aira-policy` | `c0.policy.gate`; `invariant_checker_emits_event_on_policy_deny` | **DONE** | |
| Invariant Checker can block | Book I §11; B1-008 | `aira-core` `InvariantChecker` | C0 immutability cases | **DONE** | |
| Security boundary (no Core memory / foreign CSU / secrets) | Book I §12; B1-009; B3-004 | `aira-csu` isolation + event secret scan | `isolation_baseline_denies_direct_mutation_and_peer_call`; `run_security_baseline` | **DONE** | MVP sandbox: in-process, no FS/net |
| Artifact runtime + immutability | Book I §16; B1-002 | `aira-artifact` CAS | `c0.artifact.immutability`; `c0.artifact.verify_on_read`; `mutation_fails_supersession_keeps_old`; `unsigned_artifact_rejected_and_private_denied`; `resolve_rejects_tampered_index_descriptor` | **DONE** | Verify-on-read re-checks signature + CAS hash (QUEUE #113) |
| Verified Result completeness | Book I §17; B1-010 | `schemas/result/verified-result-artifact.schema.json`; verification-basic + plane | `c1.pipeline.calculate_2_plus_2`; `c1.result.verified_completeness`; `c1.result.extended_fields` (#126); `verified_result_body_has_b1_010_required_keys`; `missing_vra_required` | **DONE** | `#202` / RFC-0098: C1 2+2 runtime body has every schema `required[]`; demo extras `result` / `artifact_kind` remain (C1 reads `result == 4.0`); C1 also emits epistemic-assessment (`#207` / RFC-0103) |
| CSU runtime + lifecycle | Book I §13–14 | `aira-csu` `CsuRuntime` / registry | `lifecycle_transitions_and_events`; `dispatch_active_only_and_failure_event`; `dispatch_fail_closed_without_policy_gate_and_on_deny`; `invoke_binds_policy_gate_check_policy_allows`; `check_policy_fail_closed_without_bound_gate`; `c0.csu.dispatch_policy` | **DONE** | Policy gate required for dispatch (QUEUE #114); invoke binds gate for `check_policy` (`#188` / RFC-0086) |
| Canonical descriptor signatures | Book I security; EVO-2 | `aira_object::canonical` + Event/Artifact/Object/CSU | mutation tests in those crates | **DONE** | QUEUE #39–#44; no runtime `LOCAL_TEST_DOMAIN_MSG` fallback |
| Federated Core (C2+/C3) | Book I §22 L2 | `aira-protocol`, `aira-peer`, federation join | C2 local; `federation::join_*` | **POST-MVP** | Not C0/C1 CI |

---

## Book II — protocols

`#200` ceiling honesty: envelope / EP / AP / identity / discovery / CAP / CRP / settlement stay **PARTIAL**. In-process or file-local implementation is the v0.3 ceiling, not Book II wire mesh. Federation stays **POST-MVP**. C2 CI job stays **DONE**.

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Common envelope + signature | Book II §6; B2-001/002 | `aira-protocol` + schemas | `c2.protocol.envelope_schema`; `c2.protocol.envelope_unsigned`; `c2.protocol.envelope_canonical_mutations`; `c2.protocol.response_canonical_mutations`; `envelope_rejects_local_test_domain_fallback`; `recv_envelope_rejects_expired`; `recv_envelope_rejects_replayed_message_id` | **PARTIAL** | SEC-2 canonical descriptor (#135); `#194` / RFC-0092 receive freshness + replay window; `#200`: local adapter = v0.3 ceiling |
| Unsupported version without side effects | B2-003 | `EventProtocolAdapter` | `c2.ep.unsupported_version_no_side_effects` | **PARTIAL** | Local adapter; `#200`: local adapter = v0.3 ceiling |
| Event Protocol publish idempotency | Book II §12; B2-008 | `aira-protocol` event adapter | `c2.event.publish_idempotent`; `event_protocol_publish_idempotent_and_unsupported_version` | **PARTIAL** | In-process; C2 case #122; `#200`: local adapter = v0.3 ceiling |
| Artifact Protocol publish/resolve + hash | Book II §11; B2-007 | `aira-protocol` artifact adapter | `c2.artifact.hash_mismatch`; `artifact_protocol_publish_resolve_and_hash_check` | **PARTIAL** | In-process; C2 case #123; `#200`: local adapter = v0.3 ceiling |
| Identity descriptor | Book II §13 | schema + `aira-object` keyring | `c2.identity.descriptor_schema`; `aira-object` crypto/keyring tests; `thread_crypto_scopes_do_not_leak` | **PARTIAL** | Local identity; rotation is node/tenant CLI; `#196` / RFC-0094 thread-local `bind_thread_crypto`; `#200`: local adapter = v0.3 ceiling |
| Discovery by Capability, not Node | Book II §8; B2-004 | `DiscoveryRegistry` | `c2.discovery.capability_not_node`; `discovery_returns_capability_not_node` | **PARTIAL** | Local registry file; `#200`: local adapter = v0.3 ceiling |
| Capability Advertisement | Book II §9; B2-005 | `CapabilityAdvertisementStore` (`capability/advertisements.json`, #160) + capability schema + HTTP discovery list | `capability_ad_persist_roundtrip`; schema fixtures; HTTP tests | **PARTIAL** | Local CAP persist (#160); discovery list ≠ CAP ads; no network ad protocol; `#200`: local adapter = v0.3 ceiling |
| CRP | Book II §10; B2-006 | schemas + `LocalCrpAdapter` + C3 `#165`–`#170` | `c3.crp.route_candidate`; `crp_multi_candidate_and_policy_gate_bind`; RFC-0079 | **PARTIAL** | Local in-process only; no multi-node mesh / marketplace; `#200`: local adapter = v0.3 ceiling |
| Federation protocol (full) | Book II §14; B2-010 | join prototype only | `aira-protocol` `federation::join_*` | **POST-MVP** | Local pin + trust; no leave/Join Request/CRP; not a v0.3 protocol **DONE** |
| Settlement / Audit protocol | Book II §15; B2-011 | receipt schema + `SettlementReceiptStore` JSONL (`aira:settlement:receipts-jsonl:v1`) + privacy + `run_c4` (`#172`–`#175`); RFC-0080/0081 | `settlement_receipt_schema_loads`; `settlement_receipt_store_append_roundtrip_and_verify_on_read`; `b2_011_settlement_privacy_smoke`; `validate_settlement_privacy`; `SETTLEMENT_PRIVACY_FORBIDDEN_KEYS`; `c4.settlement.receipt_emit_verify` | **PARTIAL** | Local audit receipts only; no blockchain ledger / federation settlement; `#200`: local adapter = v0.3 ceiling |
| C2 profile in CI | Conformance C2 | `run_c2` | GitHub Actions job `conformance-c2` (#117); local `aira-conformance` | **DONE** | C0/C1 remain primary gate; C2 is additional regression |

---

## Book III — CSU contracts (basic set)

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| Signed CSU manifest | Book III §5; B3-001/002 | `aira-csu` + `schemas/csu/manifest.schema.json` | `c1.csu.manifests`; `manifest_schema_valid_and_unsigned_rejected`; `canonical_verify_fails_when_manifest_fields_change` | **DONE** | |
| No direct CSU→CSU call | B3-004; firewall | `csu/*` crates; `scripts/dep_firewall.py` | firewall CI; `isolation_baseline_*` | **DONE** | |
| Failure Event + Evidence | B3-007/008; FAIL-* | evidence-basic + plane inject | `c1.failure.to_evidence`; `failure_creates_failure_evidence` | **DONE** | C1 SHOULD evidence: implemented |
| Context CSU (no final Result) | CTX-001 | `csu/context-basic` | `problem_submitted_creates_context_not_result` | **DONE** | |
| Reduction / reuse before execute | RED-001; OP-002 | `csu/reduction-basic` | `ready_solution_reuse_skips_execution`; `plane_reduction_binds_reuse_index_without_enable_ready_solution`; `open_with_reuse_index`; `creates_negative_lookup_and_capsule_when_no_reuse`; `local_session_repeat_problem_reuses_without_execution`; `calculate_2_plus_2_binds_math_eval_safe`; `non_math_prompt_binds_generate_local` | **DONE** | `#204` / RFC-0100: `LocalSession::submit_problem` binds `reuse-index.json`; no manual `enable_ready_solution`. `#212` / RFC-0107: non-math/echo/upper → `text.generate.local`; C1 2+2 stays `math.eval.safe`. Knowledge-vec still unused |
| Execution CSU authorized capsules | EXE-001 | `csu/execution-basic` | `math_eval_safe_completes`; `rejects_shell_action` | **DONE** | Safe math/text only; C1 2+2 unchanged |
| LLM Execution CSU (local generate) | Book I §2; Phase K `#211`–`#216` | `csu/execution-llm`; `csu/reduction-basic` catalog; `aira-flow` plane; Desktop Work `POST /v1/problems` | `inactive_model_is_capsule_failed`; `missing_process_binary_is_capsule_failed`; `mock_backend_completes_valid_generate_local`; `default_plane_keeps_mock_backend`; `generate_without_activate_is_capsule_failed`; `non_math_prompt_completes_via_execution_llm_mock`; `calculate_two_plus_two_stays_execution_basic`; `executed_generate_local_leads_with_result_not_verified`; `http_post_problem_generate_with_activate_is_executed_not_verified`; `http_post_problem_generate_without_activate_is_not_verified`; `c1.pipeline.calculate_2_plus_2` | **PARTIAL** | Phase K **DONE** @ RFC-0104. Process backend (`#215` / RFC-0110) fail-closes on missing CLI; MockBackend remains plane/CI default; activate gate (`#214` / RFC-0109); generate is CapsuleCompleted / `executed`, not VERIFIED; Desktop Work generate (`#216`) uses existing `POST /v1/problems`. Full semantic LLM verify still out |
| Verification CSU | VER-001 | `csu/verification-basic` | `verifies_math_output_as_verified_result`; `wrong_finite_math_result_is_not_verified`; `math_expression_from_capsule_artifact`; `wrong_text_echo_result_is_not_verified`; `wrong_text_uppercase_result_is_not_verified`; `text_echo_expression_from_capsule_artifact` | **DONE** | `#185` honesty (audit `b66bcf1`): previously `is_finite()` only. `#187` / RFC-0085: independent `math_eval_safe` vs claimed result; wrong finite (e.g. 2+2→5) is not **VERIFIED**. `#205` / RFC-0101: wrong `text.echo` / `text.uppercase` string is not **VERIFIED** |
| Evidence CSU | EVD-001 | `csu/evidence-basic` | crate + C1 failure path | **DONE** | Does not assign Epistemic Status |
| Artifact CSU publish/resolve/supersede | ART-001 | `csu/artifact-basic` | `publish_resolve_supersede_events` | **DONE** | |
| Model acquisition policy gate | D4; QUEUE #60/#115 | `csu/model-acquisition` | `deny_without_policy_*`; `fail_closed_audit_*`; `c0.acquisition.fail_closed` | **DONE** | Default DENY download/publish; see `docs/model-acquisition-policy.md` |
| Epistemic CSU | EPI-001; Book III §16; Book IV §13.7 | `csu/epistemic-basic` + plane wiring | `epi_001_…`; `epistemic_assessment_roundtrip_via_plane_and_session`; `calculate_two_plus_two_emits_epistemic_assessment`; `c1.pipeline.calculate_2_plus_2` | **PARTIAL** | `#207` / RFC-0103: C1 2+2 writes assessment; full Epistemic plane still out |
| Discovery / Federation / Settlement / Optimization / PHM / Evolution / Research / Human Interaction CSUs | Book III §20–27 | — | — | **ABSENT** / **RESEARCH** | Human collapse is plane STUB, not HUM CSU |
| Durable per-CSU tenant signing | Book III identity; crypto docs | `aira-object` `tenant/` | tenant module tests; CLI tenant_secret tests | **POST-MVP** | Beyond C1 basic set |

---

## Book IV — reference implementation

| Requirement | Source | Implemented in | Tested by | Status | Notes |
|-------------|--------|----------------|-----------|--------|-------|
| R0 Minimal Local Core | Book IV §23 R0 | `aira-core`, `aira-event`, `aira-artifact`, `aira-policy`, `aira-schema` | C0 + crate tests | **DONE** | |
| R1 Minimal Operational Node | Book IV §23 R1 | `aira-flow`, `aira-cli`, basic CSUs, `.aira` layout | C1; `local_init_submit_status_and_artifact`; `run_alpha_acceptance`; `two_submits_allocate_distinct_problem_ids` | **DONE** | `config.node.profile = "C1"`; `#195` UUIDv7 nonce; leftover `run-counter` ignored |
| R2 Local Protocol Node | Book IV §23 R2 | `aira-protocol` | `run_c2` + CI `conformance-c2` (#117) | **PARTIAL** | Partial C2 local; CI regression gate; `#200`: local adapter = v0.3 ceiling |
| R3 Federation-Capable Node | Book IV §23 R3 | join/leave + IO policy deny (#162) | federation tests; `federation_export_import_deny_by_default_audits` | **POST-MVP** | Local ceremony + deny-by-default export/import; not a federation runtime |
| R4 Research-Capable Node | Book IV §23 R4 | RFC-P + `OperationalPlane` reject + `run_c5` | `c5.research.separation`; `c5.promotion.gate_reject`; RFC-0082/0083 | **RESEARCH** | Book V; local C5 scaffold + non-operational gate; not a research runtime; no RFC-P instance promoted |
| Object / Artifact / Event stores | Book IV §6 | plane memory objects + node `SqliteObjectStore` (#158); CAS artifacts; file-chain events (#157 / `#203`) | `plane_memory_beside_node_sqlite_object_path`; `session_durable_file_chain_roundtrip`; `event_tail_after_reopen_reads_file_chain_not_memory_or_legacy`; core/artifact tests | **PARTIAL** | **H1 DONE** (`#154`–`#159`); `#203` event_tail = file-chain; plane drain still memory objects/events; `#200`: local adapter = v0.3 ceiling |
| Policy + Invariant + CSU runtime | Book IV §9–12 | matching crates | C0/C1 | **DONE** | |
| Minimal operational flow submit→verify | Book IV §15 | `OperationalPlane` | C1 + demos | **DONE** | Reference-local only ([operational-plane.md](operational-plane.md)) |
| Local HTTP API surface | Book IV §16 | `aira-node` `http/` | `http` module tests | **POST-MVP** | Roadmap M11; same reference plane |
| CLI (`init`, identity, csu, problem, result, artifact, event, conformance) | Book IV §17 | `aira-cli` `commands/` | CLI tests; alpha | **DONE** | Extra post-MVP: peer, federation, tenant |
| Security defaults (unsigned reject, private deny) | Book IV §19 | artifact + CSU + events | `run_security_baseline` | **DONE** | |
| Conformance harness + report artifact | Book IV §21–22 | `aira-conformance` | C0/C1 CI; C2 job `conformance-c2` (#117) | **DONE** | C2 additional regression gate |
| Suggested crate layout | Book IV §25–26 | `crates/`, `csu/` | firewall | **DONE** | |
| `OperationalPlane` ≠ production runtime | Book IV + EVO-2 §3.3 | rustdoc + docs | docs review Analyze-86 | **DONE** | QUEUE #51 |

---

## Schema Pack

CI: `cargo run -p aira-cli -- schema validate --fixtures fixtures` (QUEUE #38).

| Schema `$id` | File | Fixture in `fixtures/manifest.json` | Status |
|--------------|------|--------------------------------------|--------|
| `aira:schema:common:identifier:0.1` | `schemas/common/identifier.schema.json` | valid + invalid | **DONE** |
| `aira:schema:common:ref:0.1` | `common/ref.schema.json` | valid + invalid | **DONE** |
| `aira:schema:common:hash:0.1` | `common/hash.schema.json` | valid + invalid | **DONE** |
| `aira:schema:common:timestamp:0.1` | `common/timestamp.schema.json` | valid | **DONE** |
| `aira:schema:common:signature:0.1` | `common/signature.schema.json` | valid | **DONE** |
| `aira:schema:common:scope-descriptor:0.1` | `common/scope-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:core:object-descriptor:0.1` | `core/object-descriptor.schema.json` | valid + GPU invalid | **DONE** |
| `aira:schema:core:problem-statement:0.1` | `core/problem-statement.schema.json` | valid | **DONE** |
| `aira:schema:artifact:artifact-descriptor:0.1` | `artifact/artifact-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:artifact:context-artifact:0.1` | `artifact/context-artifact.schema.json` | valid + invalid | **DONE** |
| `aira:schema:event:event-descriptor:0.1` | `event/event-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:policy:query:0.1` | `policy/query.schema.json` (+ duplicate `policy-query.schema.json`) | valid | **DONE** |
| `aira:schema:policy:decision:0.1` | `policy/decision.schema.json` (+ duplicate `policy-decision.schema.json`) | valid | **DONE** |
| `aira:schema:capability:descriptor:0.1` | `capability/capability-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:csu:manifest:0.1` | `csu/manifest.schema.json` | valid + unsigned invalid | **DONE** |
| `aira:schema:execution:capsule:0.1` | `execution/capsule.schema.json` | valid | **DONE** |
| `aira:schema:execution:generate-local:0.1` | `execution/generate-local.schema.json` | valid + missing-prompt invalid | **DONE** (#210) |
| `aira:schema:evidence:evidence-artifact:0.1` | `evidence/evidence-artifact.schema.json` | valid | **DONE** |
| `aira:schema:epistemic:assessment:0.1` | `evidence/epistemic-assessment.schema.json` | valid + invalid | **DONE** |
| `aira:schema:result:verified-result-artifact:0.1` | `result/verified-result-artifact.schema.json` | valid + extended + invalid fixtures | **DONE** |
| `aira:schema:conformance:report:0.1` | `conformance/report.schema.json` | valid | **DONE** |
| `aira:schema:protocol:envelope:0.1` | `protocol/envelope.schema.json` | valid + unsigned invalid | **DONE** |
| `aira:schema:protocol:response:0.1` | `protocol/response.schema.json` | valid | **DONE** |
| `aira:schema:protocol:crp-route-request:0.1` | `protocol/crp-route-request.schema.json` | valid + unsigned invalid | **DONE** (#165) |
| `aira:schema:protocol:crp-route-candidate:0.1` | `protocol/crp-route-candidate.schema.json` | valid + empty-chain invalid | **DONE** (#165) |
| `aira:schema:settlement:receipt:0.1` | `settlement/receipt.schema.json` | valid + unsigned + raw_prompt invalid | **DONE** (#172) |
| `aira:schema:research:promotion-candidate:0.1` | `research/promotion-candidate.schema.json` | valid + unsigned + missing-source invalid | **DONE** (#178) |
| `aira:schema:identity:identity-descriptor:0.1` | `identity/identity-descriptor.schema.json` | valid | **DONE** |
| `aira:schema:desktop:settings:0.1` | `desktop/settings.schema.json` | valid + missing-instance-id invalid | **DONE** |

Duplicate policy schema files share the same `$id`; they are not two contracts.

Conformance spec §19 lists extra fixture *names* (event chain, policy deny, research unpromoted, …) that are **not** all present as standalone JSON files; coverage is via C0/C1 runners instead.

---

## Conformance profiles

| Profile | Spec | Runner | Cases | CI | Status |
|---------|------|--------|-------|----|--------|
| C0 | Local Core | `run_c0` | `c0.ontology.schemas`, `c0.object.immutability`, `c0.object.handle_opacity`, `c0.object.verify_on_read`, `c0.artifact.immutability`, `c0.artifact.verify_on_read`, `c0.event.causality`, `c0.policy.gate`, `c0.csu.dispatch_policy`, `c0.acquisition.fail_closed` | **yes** (QUEUE #38) | **DONE** |
| C1 | Local operational node | `run_c1` | `c1.pipeline.calculate_2_plus_2`, `c1.csu.manifests`, `c1.result.verified_completeness`, `c1.result.extended_fields`, `c1.failure.to_evidence` | **yes** | **DONE** |
| C2 | Protocol-compatible | `run_c2` | 11 cases incl. `c2.event.publish_equivocation` (SEC-4 #137); `c2.protocol.envelope_canonical_mutations`, `c2.protocol.response_canonical_mutations` (SEC-2 #135) | **yes** (`conformance-c2`, #117) | **DONE** |
| C3 | Local federation + CAP + IO deny + CRP | `run_c3` | **8** cases incl. `c3.federation.export_deny` + `c3.capability.advertisement` + `c3.crp.reject_node_route` + `c3.crp.route_candidate`; optional CI `conformance-c3` (#164, non-gate) | **optional** (`conformance-c3`; not merge gate) | **DONE** (scaffold + `#153` + `#161` + `#163` + `#164` + `#167` + `#170`) |
| C4 | Settlement audit receipts | `run_c4` | **3** cases: `c4.settlement.receipt_emit_verify`, `c4.settlement.privacy_reject`, `c4.settlement.link_prior_route`; RFC-0081 | **no** | **DONE** scaffold (`#175`) |
| C5 | Research separation + promotion gate | `run_c5` | **3** cases: `c5.research.separation`, `c5.promotion.gate_reject`, `c5.promotion.candidate_schema`; RFC-0083 | **no** | **DONE** scaffold (`#180`); not a merge gate; Research remains **RESEARCH** (`#181`) |
| Security baseline | Conformance §14 subset | `run_security_baseline` | unsigned CSU/artifact; private deny; secret in events | crate tests | **DONE** |
| Alpha acceptance | MVP | `run_alpha_acceptance` | init layout, 2+2, failure evidence, C0/C1 | crate tests | **DONE** |

C0/C1 are a **minimal** encoding of B0/B1/OP/CSU MUST tests, not a 1:1 clone of every Conformance `B*-*` id. Remaining gaps (B2 network wire; VRA `additionalProperties` extras on the C1 demo body) stay documented here.

RFC: [`AIRA-RFC-0068`](../specs/rfc/AIRA-RFC-0068-phase-f-stabilization.md) (Phase F closure `#119`).

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

## Phase F gates (DONE)

| QUEUE | Gate | Evidence |
|-------|------|----------|
| #107 | Rust 1.94.0 pinned | `rust-toolchain.toml`; CI `dtolnay/rust-toolchain@1.94.0` |
| #108 | Schema manifest gaps closed | `fixtures/manifest.json` identifier/epistemic/context-artifact |
| #109 | CI governance documented | [`docs/ci-governance.md`](ci-governance.md) |
| #110 | SECURITY.md posture | [`SECURITY.md`](../SECURITY.md); RFC-0059 |
| #111 | Handle opacity C0 | `c0.object.handle_opacity` |
| #112 | Object verify-on-read | `c0.object.verify_on_read` |
| #113 | Artifact verify-on-read | `c0.artifact.verify_on_read` |
| #114 | Policy Gate dispatch | `c0.csu.dispatch_policy` |
| #115 | Acquisition fail-closed | `c0.acquisition.fail_closed`; [`model-acquisition-policy.md`](model-acquisition-policy.md) |
| #116 | model-acquisition split | modular `csu/model-acquisition/src/*` |
| #117 | C2 CI job | `.github/workflows/ci.yml` job `conformance-c2` |
| #118 | peer_lifecycle_p2 stable | `serial_test` + port retry in integration tests |
| #119 | Phase F docs + RFC | this file + README; RFC-0068 |

Plan: [`phase-f-plan.md`](phase-f-plan.md).

---

## Phase G gates (OPEN)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #120 | CI branch protection sync | `ci-governance.md` + `ci_governance_doc.rs`; RFC-0070 | **DONE** @ PR #83 |
| #121 | Living spec smoke | `living_spec_smoke.rs`; RFC-0071 | **DONE** @ PR #84 |
| #122 | C2 event idempotency | `c2.event.publish_idempotent`; RFC-0072 | **DONE** @ PR #85 |
| #123 | C2 artifact hash mismatch | `c2.artifact.hash_mismatch`; RFC-0073 | **DONE** @ PR #86 |
| #124 | C2 envelope unsigned | `c2.protocol.envelope_unsigned`; RFC-0074 | **DONE** @ PR #87 |
| #125 | Claim/Assumption fixtures | `claim-artifact.schema.json` + fixtures; RFC-0075 | **DONE** @ PR #88 |
| #126 | VRA extended fields | `c1.result.extended_fields`; RFC-0076 | **DONE** @ PR #89 |
| #127 | Split crypto mod | `crypto/smoke_tests.rs` mechanical split | **DONE** @ PR #90 |
| #128 | Split aira-peer lib | `smoke_tests.rs` mechanical split | **DONE** @ PR #91 |
| #129 | Split cli peer | `commands/peer/` mechanical split | **DONE** @ PR #92 |
| #130 | Split desktop app | `app/` mechanical split | **DONE** @ PR #93 |
| #131 | peer_lifecycle_p3 stable | `serial_test` + port retry; RFC-0067 | **DONE** @ PR #94 |
| #132 | peer_lifecycle_p4 stable | `serial_test` + port retry; RFC-0067 | **DONE** @ PR #95 |
| #133 | Invite QR camera | GUI camera + `import_invite_qr_luma` | **DONE** @ PR #96 |
| #134–#138 | SEC hardening (audit) | local-test trust; envelope sign; identity bind; equivocation; artifact admission |
| #139–#141 | Federation + C3 scaffold | leave CLI; join hardening; `run_c3` local |
| #142 | Event log durability | corrupt JSON backup + reset (`event-log.json.corrupt`); test in `aira-flow` | **DONE** @ PR #105 |
| #143 | SQLite object path | `init_node` `db/aira.sqlite` migrate idempotent; corrupt JSON fail-closed; tests | **DONE** @ PR #106 |
| #144 | External CSU fixture | `manifest-external-partner.json`; `external_third_party_csu_fixture_registry_smoke` | **DONE** @ PR #107 |
| #145 | CSU SDK doc | `docs/csu-sdk.md`; `c1.csu.external_partner_fixture` | **DONE** @ PR #108 |
| #146 | epistemic-basic CSU | `csu/epistemic-basic`; EPI-001 smoke | **DONE** @ PR #109 |
| #147 | Epistemic assessment path | plane/session roundtrip; `epistemic_assessment_roundtrip_via_plane_and_session` | **DONE** @ PR #110 |
| #148 | macOS codesign | `scripts/macos-codesign-notarize.sh` + `desktop-packaging-macos.md` | **DONE** @ PR #111 |
| #149 | Windows MSI | `scripts/package-desktop-windows-msi.sh` + WiX stage | **DONE** @ PR #112 |
| #150 | Linux deb | `scripts/package-desktop-linux-deb.sh` + debian control | **DONE** @ PR #113 |
| #151 | Phase G docs + RFC | README + this file v0.2; `AIRA-RFC-0069`; QUEUE G closed | **DONE** @ PR #114 |

Plan: [`phase-g-plan.md`](phase-g-plan.md) **DONE**. RFC: [`AIRA-RFC-0069`](../specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md).

---


---

## Phase H gates (DONE)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #152 | Phase H wiring + contract | `phase_h_doc.rs`; `docs/phase-h-plan.md` | **DONE** @ PR #115 |
| #153 | C3 governance (optional) | `ci-governance.md` + living-spec C3 optional | **DONE** @ PR #116 |
| #154 | EventHashChain tip | `event_log_hash_chain_tip_append_verify_and_mid_tamper_detect` | **DONE** @ PR #117 |
| #155 | Event log prefix recovery | `corrupt_trailing_event_log_recovers_valid_prefix` | **DONE** @ PR #118 |
| #156 | FileChainEventLog | `file_chain_event_log_persists_across_reopen` | **DONE** @ PR #119 |
| #157 | Session wire file-chain | `session_durable_file_chain_roundtrip` | **DONE** @ PR #120 |
| #158 | Sqlite object path + docs | `plane_memory_beside_node_sqlite_object_path` | **DONE** @ PR #121 |
| #159 | Stores status rollup | this file H1 rows + Phase H table | **DONE** @ this PR |
| #160 | Capability ad persist | `capability_ad_persist_roundtrip` | **DONE** @ PR #123 |
| #161 | C3 capability advertisement | `c3.capability.advertisement` | **DONE** @ PR #124 |
| #162 | Federation IO export/import deny | `federation_export_import_deny_by_default_audits` | **DONE** @ PR #125 |
| #163 | C3 cases ≥6 | `run_c3` 6 named cases | **DONE** @ PR #127 |
| #164 | Optional C3 CI | `conformance-c3` in ci.yml (not required) | **DONE** @ PR #128 |
| #165 | CRP schema fixtures | route request/candidate schemas + fixtures | **DONE** @ PR #129 |
| #166 | CRP local adapter | `LocalCrpAdapter`; RFC-0079; `crp_local_adapter_routes_capability_not_node` | **DONE** @ PR #130 |
| #167 | CRP reject node route | `c3.crp.reject_node_route` | **DONE** @ PR #131 |
| #168 | CRP multi-candidate gate | `crp_multi_candidate_and_policy_gate_bind`; `crp.bind` | **DONE** @ PR #132 |
| #169 | CRP route events | `RouteSelected`/`Rejected`/`Failed`; `crp_route_events_selected_rejected_failure` | **DONE** @ PR #133 |
| #170 | B2-006 C3 case | `c3.crp.route_candidate` | **DONE** @ PR #134 |
| #171 | CRP status PARTIAL | Book II CRP row ABSENT→**PARTIAL** | **DONE** @ PR #135 |
| #172 | Settlement receipt fixtures | `settlement/receipt.schema.json` + privacy_class; PRIV-001 | **DONE** @ PR #136 |
| #173 | Settlement receipt store | `SettlementReceiptStore` JSONL; verify-on-read; RFC-0080 | **DONE** @ PR #137 |
| #174 | B2-011 privacy smoke | `validate_settlement_privacy`; `b2_011_settlement_privacy_smoke` | **DONE** @ PR #138 |
| #175 | `run_c4` scaffold | 3 cases; RFC-0081; `c4.settlement.receipt_emit_verify` | **DONE** @ PR #139 |
| #176 | Settlement status PARTIAL | Book II Settlement ABSENT→**PARTIAL** | **DONE** @ PR #140 |
| #177 | RFC-P promotion doc | `docs/rfc-p-promotion.md`; Book V → operational лише через promotion | **DONE** @ PR #141 |
| #178 | Promotion-candidate fixtures | `aira:schema:research:promotion-candidate:0.1`; `promotion_candidate_schema_loads` | **DONE** @ PR #142 |
| #179 | Non-operational gate | `ResearchNonOperational`; `research_artifact_rejected_as_operational_input`; RFC-0082 | **DONE** @ PR #143 |
| #180 | `run_c5` scaffold | 3 cases; RFC-0083; `c5.research.separation` | **DONE** @ PR #144 |
| #181 | Promotion status rollup | this file C5 / Research / R4 notes; **H5 DONE**; no promoted research item | **DONE** @ PR #145 |
| #182 | Reference v0.3 docs | README + this file **v0.3**; conformance index | **DONE** @ PR #146 |
| #183 | Phase H RFC-0077 | consolidating RFC; QUEUE H closed; no OPEN | **DONE** @ this PR |

Plan: [`phase-h-plan.md`](phase-h-plan.md). Consolidating RFC: [`AIRA-RFC-0077`](../specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md).

---

## Phase I gates (DONE)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #184 | Phase I wiring + contract | `phase_i_doc.rs`; `docs/phase-i-plan.md`; QUEUE `#184`–`#198` | **DONE** @ this PR |
| #185 | Status honesty rollup | Opaque Handle / Reduction / Verification **PARTIAL** vs audit `b66bcf1`; `Handle::new`; `is_finite()`; `LocalSession` `vec![]` | **DONE** @ this PR |
| #186 | Handle integrity | `Handle::new` crate-private; `object_store_access`; `HandleBindMismatch`; RFC-0084; `handle_cross_object_token_bind_rejects` | **DONE** @ this PR |
| #187 | Semantic verify math.eval.safe | independent eval; `wrong_finite_math_result_is_not_verified`; RFC-0085 | **DONE** @ this PR |
| #188 | CSU PolicyGate in invoke | `invoke_binds_policy_gate_check_policy_allows`; `check_policy_fail_closed_without_bound_gate`; RFC-0086 | **DONE** @ this PR |
| #189 | Durable reuse index | `local_session_repeat_problem_reuses_without_execution`; `reuse-index.json`; RFC-0087 | **DONE** @ this PR |
| #190 | Fail-closed signing | `active_signature_does_not_fallback_to_local_test`; `local_session_rejects_corrupt_identity`; RFC-0088 | **DONE** @ this PR |
| #191 | Atomic session persist | `local_session_corrupt_problems_index_is_not_silent_wipe`; RFC-0089 | **DONE** @ this PR |
| #192 | Artifact metadata recovery | `second_descriptor_same_content_hash_recoverable`; RFC-0090 | **DONE** @ this PR |
| #193 | Runtime Clock | `system_clock_is_not_the_mvp_fixed_timestamp`; `local_session_artifacts_are_not_all_mvp_fixed_timestamp`; RFC-0091 | **DONE** @ this PR |
| #194 | Envelope freshness/replay | `recv_envelope_rejects_expired`; `recv_envelope_rejects_replayed_message_id`; RFC-0092 | **DONE** @ this PR |
| #195 | Run nonce concurrency | `alloc_run_nonce_concurrent_is_unique`; leftover `run-counter` ignored; RFC-0093 | **DONE** @ this PR |
| #196 | Instance-scoped crypto | `thread_crypto_scopes_do_not_leak`; `bind_thread_crypto`; RFC-0094 | **DONE** @ this PR |
| #197 | MSRV + supply-chain CI | `workspace_rust_version_matches_ci_pin`; `ci_actions_are_sha_pinned`; `cargo_deny_optional_ci_job_is_not_merge_gate`; RFC-0095 | **DONE** @ this PR |
| #198 | Phase I docs + RFC-0078 | v0.3-stable; consolidating RFC; QUEUE I closed | **DONE** @ this PR |

Plan: [`phase-i-plan.md`](phase-i-plan.md). Consolidating RFC: [`AIRA-RFC-0078`](../specs/rfc/AIRA-RFC-0078-phase-i-semantic-stabilization.md) (id **confirmed free** at `#184`).

---

## Phase J gates (DONE)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #199 | Phase J wiring + contract | `phase_j_doc.rs`; `docs/phase-j-plan.md`; QUEUE `#199`–`#208` | **DONE** @ this PR |
| #200 | Book II ceiling honesty | envelope/EP/AP/identity/discovery/CAP/CRP/settlement **PARTIAL**; local adapter = v0.3 ceiling | **DONE** @ this PR |
| #201 | Seal `object_store_access` | `mint` not a public CSU prelude API; RFC-0097; `store-backend` | **DONE** @ this PR |
| #202 | VRA runtime B1-010 | C1 2+2 body matches `verified-result-artifact.schema.json` `required[]`; RFC-0098 | **DONE** @ this PR |
| #203 | Event-log authority | reopen `event_tail` from `events/file-chain-log.json`; RFC-0099 | **DONE** @ this PR |
| #204 | Reduction catalog bind | durable reuse without manual `enable_ready_solution`; RFC-0100 | **DONE** @ this PR |
| #205 | Semantic verify text.* | `text.echo` / `text.uppercase` wrong string not VERIFIED; RFC-0101 | **DONE** @ this PR |
| #206 | Evidence primacy runtime | Claim vs Assumption reject in runtime; RFC-0102 | **DONE** @ this PR |
| #207 | Epistemic emit on C1 | C1 2+2 writes epistemic-assessment artifact; RFC-0103; `calculate_two_plus_two_emits_epistemic_assessment` | **DONE** @ this PR |
| #208 | Phase J docs + RFC-0096 | v0.3-strict; consolidating RFC-0096; QUEUE J closed | **DONE** @ this PR |

Plan: [`phase-j-plan.md`](phase-j-plan.md). Consolidating RFC: [`AIRA-RFC-0096`](../specs/rfc/AIRA-RFC-0096-phase-j-book-gap-local.md) (id **confirmed free** at `#199`).

## Explicitly not this reference (anti-mission + out-of-scope)

README §«What AIRA is not» — **ніколи** в QUEUE / phase plans як «later phase»:

```text
GPU marketplace · LLM runtime · blockchain network · scheduler
Core що «вирішує задачі» замість CSU contracts
```

Навмисно **не** production reference (Book IV posture):

```text
production event runtime / scheduler / distributed OperationalPlane
CRP product pivot · settlement ledger product · PHM/HIE runtime
KnowledgeOps · Goal Compiler · DSM · full Book II wire mesh
```

**Phase G `#120`–`#151` DONE** (не змінює anti-mission): deeper local C2, mechanical splits, desktop peer CI, SEC hardening, federation leave, local C3 scaffold, CSU SDK doc, `epistemic-basic`, production packaging scripts. Plan: [`phase-g-plan.md`](phase-g-plan.md); closure: [`AIRA-RFC-0069`](../specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md).

**Phase H `#152`–`#183` labelled Reference v0.3** (не змінює anti-mission): protocol-depth docs in README / this file / [conformance.md](conformance.md); consolidating [`AIRA-RFC-0077`](../specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md). Plan: [`phase-h-plan.md`](phase-h-plan.md).

**Phase I `#184`–`#198` DONE** @ RFC-0078 (не змінює anti-mission): semantic contract stabilization; [`phase-i-plan.md`](phase-i-plan.md); **Reference v0.3-stable**; QUEUE I closed; no OPEN.

**Phase J `#199`–`#208` DONE** @ RFC-0096 (не змінює anti-mission): Book-gap local remainder closed; [`phase-j-plan.md`](phase-j-plan.md); **Reference v0.3-strict**; Book II local adapter = v0.3 ceiling; Opaque Handle sealed (`#201` / RFC-0097); VRA runtime B1-010 (`#202` / RFC-0098); event-log authority (`#203` / RFC-0099); Reduction catalog bind (`#204` / RFC-0100); semantic verify text.* (`#205` / RFC-0101); evidence primacy runtime (`#206` / RFC-0102); epistemic emit on C1 (`#207` / RFC-0103); QUEUE J closed.

**Phase K `#209`–`#216` **DONE** @ RFC-0104 (не змінює anti-mission): Local LLM Execution CSU; [`phase-k-plan.md`](phase-k-plan.md); `#209` wiring **DONE**; `#210` generate-local schema **DONE** (RFC-0105); `#211` `execution-llm` mock **DONE** (RFC-0106); `#212` Reduction bind **DONE** (RFC-0107); `#213` Plane register **DONE** (RFC-0108); `#214` Activate gate **DONE** (RFC-0109); `#215` Process backend **DONE** (RFC-0110); `#216` Desktop Work generate **DONE**; consolidating [`AIRA-RFC-0104`](../specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md); QUEUE K closed.

**Phase L `#217`–`#223` **DONE** @ RFC-0111 (не змінює anti-mission): Execution-boundary hardening after K; [`phase-l-plan.md`](phase-l-plan.md); `#217` wiring **DONE**; `#218` Activate evidence **DONE** (RFC-0112); `#219` Child env whitelist **DONE** (RFC-0113); `#220` Bounded pipes **DONE** (RFC-0114); `#221` ProblemRecord split **DONE** (RFC-0115); `#222` `network=none` contract **DONE** (RFC-0116); `#223` consolidating RFC-0111 **DONE**; QUEUE L closed.

**Phase M `#224`–`#230` **DONE** @ RFC-0117 (не змінює anti-mission): Linux OS child sandbox after L; [`phase-m-plan.md`](phase-m-plan.md); `#224` wiring **DONE**; `#225` Landlock **DONE** (RFC-0118); `#226` seccomp **DONE** (RFC-0119); `#227` netns **DONE** (RFC-0120); `#228` sandbox-required **DONE** (RFC-0121); `#229` OS vs AIRA-mediated **DONE** (RFC-0122); `#230` consolidating RFC-0117 **DONE**; QUEUE M closed.

**Phase N `#231`–`#247` IN PROGRESS** (не змінює anti-mission): Global Node Rendezvous & Prime Connectivity; [`phase-n-plan.md`](phase-n-plan.md); `#231`–`#239` **DONE** @ RFC-0124…0131; first OPEN `#240`; consolidating RFC-0123 at `#247`.

## Phase K gates (DONE)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #209 | Phase K wiring + contract | `phase_k_doc.rs`; `docs/phase-k-plan.md`; QUEUE `#209`–`#216` | **DONE** @ this PR |
| #210 | Capsule `text.generate.local` | `aira:schema:execution:generate-local:0.1` + fixtures; RFC-0105 | **DONE** @ this PR |
| #211 | `execution-llm` CSU + mock | CapsuleCompleted via MockBackend; fail-closed without backend; RFC-0106 | **DONE** @ this PR |
| #212 | Reduction bind | non-math → generate; C1 2+2 unchanged; RFC-0107 | **DONE** @ this PR |
| #213 | Plane register | OperationalPlane dispatches generate to execution-llm; RFC-0108 | **DONE** @ this PR |
| #214 | Activate gate | no Phase D activate → CapsuleFailed | **DONE** @ this PR |
| #215 | Process backend | missing binary fail-closed; CI mock | **DONE** @ this PR |
| #216 | Desktop + RFC-0104 | Work generate path; QUEUE K closed | **DONE** @ this PR |

Plan: [`phase-k-plan.md`](phase-k-plan.md). Consolidating RFC: [`AIRA-RFC-0104`](../specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md) (id **confirmed free** at `#209`).

## Phase L gates (DONE)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #217 | Phase L wiring + contract | `phase_l_doc.rs`; `docs/phase-l-plan.md`; QUEUE `#217`–`#223` | **DONE** @ this PR |
| #218 | Activate evidence | hash + cache bytes + evidence artifact; RFC-0112 | **DONE** @ this PR |
| #219 | Child env whitelist | `env_clear`; no `AIRA_HTTP_TOKEN`; RFC-0113 | **DONE** @ this PR |
| #220 | Bounded pipes | cap during read; overflow fail-closed; RFC-0114 | **DONE** @ this PR |
| #221 | ProblemRecord split | `execution_artifact_id` persisted; GET honesty; RFC-0115 | **DONE** @ this PR |
| #222 | `network=none` contract | AIRA-mediated none; not OS sandbox; RFC-0116 | **DONE** @ this PR |
| #223 | RFC-0111 + close | consolidating RFC; QUEUE L closed | **DONE** @ this PR |

Plan: [`phase-l-plan.md`](phase-l-plan.md). Consolidating RFC: [`AIRA-RFC-0111`](../specs/rfc/AIRA-RFC-0111-phase-l-execution-boundary.md) (id **confirmed free** at `#217`).

## Phase M gates (DONE)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #224 | Phase M wiring + contract | `phase_m_doc.rs`; `docs/phase-m-plan.md`; QUEUE `#224`–`#230` | **DONE** @ this PR |
| #225 | Landlock FS | Linux Landlock on ProcessBackend child; RFC-0118 | **DONE** @ this PR |
| #226 | seccomp | syscall filter; forbidden syscall fail-closed; RFC-0119 | **DONE** @ this PR |
| #227 | netns | Linux network namespace for offline argv; RFC-0120 | **DONE** @ this PR |
| #228 | missing sandbox fail-closed | required + no kernel/OS → CapsuleFailed; RFC-0121 | **DONE** @ this PR |
| #229 | OS vs AIRA-mediated docs | schema/docs distinguish OS isolation from RFC-0116; RFC-0122 | **DONE** @ this PR |
| #230 | RFC-0117 + close | consolidating RFC; QUEUE M closed | **DONE** @ this PR |

Plan: [`phase-m-plan.md`](phase-m-plan.md). Consolidating RFC: [`AIRA-RFC-0117`](../specs/rfc/AIRA-RFC-0117-phase-m-os-child-sandbox.md) (id **confirmed free** at `#224`).

## Phase N gates (IN PROGRESS)

| QUEUE | Gate | Evidence (target) |
|-------|------|-------------------|
| #231 | Phase N wiring + contract | `phase_n_doc.rs`; `docs/phase-n-plan.md`; QUEUE `#231`–`#247` | **DONE** @ this PR |
| #232 | Prime Port invariant | `prime_port`; P_AIRA=1491; fail-closed; RFC-0124 | **DONE** @ this PR |
| #233 | Deterministic port selection | preferred_port; collision walk; RFC-0125 | **DONE** @ this PR |
| #234 | Presence Record | schema + Ed25519; RFC-0126 | **DONE** @ this PR |
| #235 | RendezvousProvider | trait + mock; RFC-0127 | **DONE** @ this PR |
| #236 | EVM ledger adapter | local double; Amoy/mainnet hooks; RFC-0128 | **DONE** @ this PR |
| #237 | Publish/query | TTL/sequence; RFC-0129 | **DONE** @ this PR |
| #238 | Reachability Probe | peer-assisted challenge; RFC-0130 | **DONE** @ this PR |
| #239 | Reachability states | reachability.json; RFC-0131 | **DONE** @ this PR |
| #240 | AddressBook promotion | trust-gated | OPEN |
| #241 | Relay integration | direct→relay | OPEN |
| #242 | Presence refresh | sequence++ | OPEN |
| #243 | CLI | port/reachability/rendezvous | OPEN |
| #244 | Desktop UX | Network panel | OPEN |
| #245 | Ab ovo integration | zero-knowledge discovery | OPEN |
| #246 | NAT/relay integration | relay Noise | OPEN |
| #247 | RFC-0123 + close | consolidating RFC; QUEUE N closed | OPEN |

Plan: [`phase-n-plan.md`](phase-n-plan.md). Consolidating RFC: `AIRA-RFC-0123` (id **confirmed free** at `#231`).


Model layer (EVO-3): D0–D7 `#53`–`#74` **DONE** @ d270b62. Not Core. Plan: [phase-d-plan.md](phase-d-plan.md).

Desktop one-click / GUI: `#75`–`#79` **DONE**. E1.1 `#80`–`#85` **DONE**. E2 macOS `#86`–`#89` **DONE**. E3 Windows `#90`–`#93` **DONE**. E4 P2–P6 `#94`–`#106` **DONE** (network profiles; see [`desktop-network-profiles.md`](desktop-network-profiles.md)). GUI Work tab submits `POST /v1/problems` and leads with `result.result` + `status` + `verification_status` (full VRA JSON is Details); uk/en chrome; `open_ui_on_start` applies only to `--from-autostart` (no Settings lock-out). Model layer (Phase D) is CLI/CSU inventory+policy, not Core inference. Phase K **DONE** @ RFC-0104: generate-local uses the same Work HTTP path (MockBackend in CI; process CLI opt-in `#215`); fail-closes without Phase D activate (`#214`) and never fakes VERIFIED. C1 2+2 still binds `execution-basic`.
