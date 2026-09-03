# AIRA-RFC-0115 — ProblemRecord execution vs verified split (RFC-D)

## 1. Summary

Phase L `#221`: persisted `ProblemRecord` MUST store generate-local output as `execution_artifact_id`. GET `/v1/problems/:id` MUST NOT put an executed id in `verified_artifact_id`.

## 2. Problem Statement

POST `/v1/problems` already returns `execution_artifact_id` for `SubmitOutcome::Executed` and omits `verified_artifact_id`. Persist wrote the execution id into `verified_artifact_id`, so GET after generate-local labelled ExecutionArtifact as a Verified Result Artifact.

## 3. Motivation

Post-K audit: disk `ProblemRecord` lied about VERIFIED. Clients that poll GET (Desktop Work, CLI status) inherited the lie even when POST was honest.

## 4. Scope

- [`ProblemRecord`](../../crates/aira-flow/src/local.rs) field `execution_artifact_id`
- Persist: Completed → `verified_artifact_id` only; Executed → `execution_artifact_id` only
- Read remap: `status == "executed"` with a legacy verified id moves it to `execution_artifact_id`
- RFC-0111 stays file-free until `#223`

## 5. Non-Goals

```text
network=none RFC (#222)
RFC-0111 body (#223)
Landlock / seccomp
reuse-index for executed records
llama/ggml in aira-core
GPU marketplace
```

## 6. Current Behavior

`persist_after_submit` for Executed set `verified_artifact_id: Some(execution_artifact_id)`. GET serialized that field. Reuse-index still recorded only Completed (unchanged).

## 7. Proposed Change

```text
Completed: verified_artifact_id = Some(vra); execution_artifact_id = None
Executed:  verified_artifact_id = None; execution_artifact_id = Some(exec)
NeedsHumanCollapse: both None
GET/status: split_executed_verified_lie on read for old index files
C1 2+2 unchanged (Completed + verified_artifact_id)
```

## 8. Affected Books / Schemas / Tests

- Book I: Verified Result Artifact vs ExecutionArtifact
- Schema: none (session JSON, not Schema Pack)
- Tests: `local_session_generate_persists_execution_not_verified`; `problem_status_remaps_legacy_executed_verified_lie`; `http_get_problem_generate_parity_not_verified`; `phase_l_problem_record_221`

## 9. Compatibility Impact

Old `problems/index.json` rows with `status=executed` and a verified id are remapped on read. Writers of GET clients that treated any `verified_artifact_id` as VRA stop seeing that field for generate-local.

## 10. Security Impact

Stops presenting unverified generate-local output as a Verified Result Artifact on the status surface.

## 11. Privacy Impact

None.

## 12. Policy Impact

None.

## 13. Failure Semantics

Must not mint VerifiedResultArtifact for generate-local. Must not persist executed ids under `verified_artifact_id`. CapsuleFailed paths are unchanged.

## 14. Rollback Plan

Revert `ProblemRecord` persist mapping. Do not create RFC-0111.

## 15. Conformance Tests

```text
cargo test -p aira-flow --lib -- local_session_generate_persists_execution_not_verified
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
cargo test -p aira-node --lib -- http_get_problem_generate_parity_not_verified
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_l_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

Read-side remap for legacy executed rows. No schema bump.

## 17. Alternatives Considered

- Dual-write both fields with the same id — rejected (GET would still look verified).
- Rewrite index.json on every GET — rejected for this atom (status honesty without mutating disk).
- Drop GET `verified_artifact_id` entirely — rejected (C1 Completed still needs it).

## 18. Evidence

- QUEUE `#221`; [`docs/phase-l-plan.md`](../../docs/phase-l-plan.md) L4
- POST generate already omitted `verified_artifact_id` (`http_post_problem_generate_with_activate_is_executed_not_verified`)

## 19. Open Questions

None for this atom.
