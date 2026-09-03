# AIRA-RFC-0112 — Activate evidence/hash gate (RFC-D)

## 1. Summary

Phase L `#218`: `ActivatedPointerGate` must verify Phase D activation **evidence**, not pointer presence. A forged `{"model_ref":"…"}` in `models/activated.latest.json` MUST fail-closed (`CapsuleFailed`), never VERIFIED.

## 2. Problem Statement

`#214` treated `activated.latest.json` as sufficient if `model_ref` was non-empty. Phase D `activate_verified` writes `content_hash`, cache path, and a signed Evidence artifact. Pointer-exists is not activation.

## 3. Motivation

Post-K audit: the generate path could be unlocked by writing a one-field JSON file. That is not Phase D activate.

## 4. Scope

- Plane [`ActivatedPointerGate`](../../crates/aira-flow/src/activate_gate.rs): required pointer fields, cache under `models/`, cache bytes `content_hash`, Evidence artifact resolve, `activated=true`, cryptographic payload signature
- Fixture helper `install_fixture` for tests/HTTP
- CSU ↛ CSU: still no Cargo dep on `model-acquisition`
- RFC-0111 stays file-free until `#223`

## 5. Non-Goals

```text
child env_clear (#219)
bounded stdout/stderr (#220)
ProblemRecord execution_artifact_id (#221)
network=none RFC (#222)
RFC-0111 body (#223)
Landlock / seccomp
llama/ggml in aira-core
GPU marketplace
```

## 6. Current Behavior

Gate accepted any JSON object with a non-empty `model_ref`.

## 7. Proposed Change

```text
ActivatedPointerGate:
  missing/invalid pointer → CapsuleFailed
  cache missing or hash mismatch → CapsuleFailed
  evidence missing / activated!=true / signature fail → CapsuleFailed
  forged {model_ref} only → CapsuleFailed
  real Phase D-shaped fixture + MockBackend → CapsuleCompleted / Executed
AlwaysActivated tests unchanged
C1 2+2 unchanged
```

## 8. Affected Books / Schemas / Tests

- Book I §2: LLM Backend CSU; activation is Phase D
- Schema: none
- Tests: `forged_model_ref_only_pointer_is_denied`; `fixture_pointer_allows_generate`; `cache_hash_mismatch_is_denied`; `forged_model_ref_pointer_is_capsule_failed`; `phase_d_activated_pointer_allows_mock_generate`; `http_post_problem_generate_with_activate_is_executed_not_verified`; `phase_l_activate_evidence_218`

## 9. Compatibility Impact

Hosts with a real Phase D activate keep working. Test fixtures that wrote `model_ref` only must use `install_fixture`.

## 10. Security Impact

Fail-closed on forged pointers. Cache path cannot escape `models/`. Evidence signature must be cryptographic (not TESTSIG).

## 11. Privacy Impact

None beyond existing generate-local.

## 12. Policy Impact

None. Acquisition policy unchanged.

## 13. Failure Semantics

MUST emit `CapsuleFailed` (and plane FailureEvidence). MUST NOT mint VerifiedResultArtifact.

## 14. Rollback Plan

Revert `activate_gate.rs` to pointer-exists `#214` behavior. Do not create RFC-0111.

## 15. Conformance Tests

```text
cargo test -p aira-flow --lib
cargo test -p aira-node
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_l_doc --test phase_k_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.

## 16. Migration Plan

Re-run Phase D `activate` if a host only has a stub pointer.

## 17. Alternatives Considered

- Cargo-dep plane on model-acquisition — rejected (keep types duplicated / JSON-shaped).
- Trust pointer `content_hash` without hashing cache bytes — rejected (audit).
- OS sandbox — out (`#222` / post-L).

## 18. Evidence

- Phase D `#64` / RFC-0013 activate_verified pointer fields
- QUEUE `#218`; [`docs/phase-l-plan.md`](../../docs/phase-l-plan.md) L1
- RFC-0109 pointer-exists gate (superseded for verification depth, not removed)

## 19. Open Questions

None for this atom.
