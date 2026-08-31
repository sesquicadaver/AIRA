# AIRA-RFC-0098 — VRA runtime body B1-010

## 1. Summary

Phase J `#202`: verification-basic emits a Verified Result Artifact whose **payload** contains every `required[]` key from [`schemas/result/verified-result-artifact.schema.json`](../../schemas/result/verified-result-artifact.schema.json). C1 `calculate_2_plus_2` / `c1.result.verified_completeness` assert those keys on the runtime body, not only on fixtures.

`result` and `artifact_kind` remain demo extras so C1/flow still read `result == 4.0`. Full schema validate with `additionalProperties: false` is not this atom.

## 5. Non-Goals

Event-log authority (`#203`); epistemic emit (`#207`); RFC-0096 (`#208`); stripping C1 `result` extras.

## 10. Contract

```text
C1 2+2 VRA body — all schema required[] present
result_id — artifact id
problem_statement_ref / context_ref — capsule body, else object_refs / aira:context:unresolved
solution_refs — [execution output]
artifact_hash — SHA-256 of canonical JSON without hash/signature
signature — Ed25519 over artifact_hash string (tenant / primary)
created_at — process Clock
```

## 15. Tests

```text
cargo test -p aira-csu-verification-basic --lib
cargo test -p aira-flow calculate_two_plus_two_demo
cargo test -p aira-conformance --lib
cargo test -p aira-desktop-runtime --test phase_j_doc --test phase_i_doc --test phase_h_doc
```
