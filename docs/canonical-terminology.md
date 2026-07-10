# Canonical Terminology Guardrail

**Issue:** #8  
**Purpose:** Prevent regression into superseded GPU/Node/Driver/Scheduler-first vocabulary in Core.

## Forbidden as Core / ontology terms

Do **not** introduce these as required Core entities, Core object types, or Stable ABI concepts:

| Forbidden | Why |
|-----------|-----|
| GPU | Hardware resource, not Core ontology |
| Node | Physical/network host, not Core identity |
| Driver | Superseded by **CSU** |
| Scheduler | Superseded by Optimization CSU / Capability binding (optional) |
| Blockchain | Optional settlement backend only; never Core requirement |
| Wallet | Not mandatory identity |
| LLM model / `llm_model_id` | Implementation detail; not Core dependency |
| HIE | Research / historical; not Core |
| Goal | User/Research plane; Core starts at **Problem Statement** |
| Knowledge Graph | Not Core Knowledge Graph requirement |
| Execution Plan | Prefer **Execution Capsule** + Capability chain |

## Canonical replacements

| Prefer | Instead of |
|--------|------------|
| **CSU** | Driver / plugin / agent module |
| **Capability** | Node skill / GPU offer |
| **Artifact** | Mutable blob / shared memory result |
| **Event** | Ad-hoc log line without causality |
| **Policy** | Hard-coded ACL in Core |
| **Execution Capsule** | Job / task / Execution Plan |
| **Problem Statement** | Goal (in Core) |
| **Verified Result Artifact** | Raw model output |
| **Evidence** | Unreferenced claim |
| **Settlement / Audit** | Mandatory blockchain ledger |

## Schema couplings (MUST NOT)

From Schema Pack §28 — schemas must not require:

`gpu_id`, `node_id` as core identity, `driver_id` as canonical role, `scheduler_id`, `wallet_address` as mandatory identity, `blockchain_height` as mandatory settlement field, `llm_model_id` as core dependency, `global_state_version`.

## Enforcement

- Object descriptor `object_type` enum rejects GPU/Node/Driver/…
- CI / schema fixtures include invalid forbidden-type cases
- CONTRIBUTING MVP freeze aligns with this guardrail
