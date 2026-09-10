# AIRA-RFC-0199 — Verification capsule-sourced (RFC-D)

## 1. Summary

Phase U `#314`: VerificationBasic sources `action` / `expression` from the **admitted capsule** (`CapsuleCompleted` `artifact_refs[1]`), not from executor output. Output fields, when present, must match the capsule. A substituted but internally consistent output (e.g. capsule `2+2`, output `1+1`/`2`) yields `VerificationFailed` with no VRA. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

`action_expression` preferred output `expression`, falling back to capsule only when absent. An executor could emit a different expression/result pair that recomputes cleanly and still receive VERIFIED while problem/context bindings came from the original capsule.

## 3. Motivation

`phase-u-plan` `#314` / post-T audit §2: independent verification must check what was commissioned, not what the executor claimed as input.

## 4. Scope

- `admitted_capsule` / `capsule_action_expression` / `output_matches_capsule`
- Fail-closed without readable capsule
- Tests: substituted expression; missing capsule
- QUEUE tip → `#315`
- RFC-D **0199**

## 5. Non-Goals

```text
Result-by-problem ArtifactStore authority (#315)
CLI identity (#316)
Changing generate-local non-VERIFIED policy
Merging with other U1 atoms
```

## 6. Compatibility / Security

Production CapsuleCompleted already carries `[output, capsule]`. Historical RFCs 0085/0101 “output OR capsule” are superseded for honesty by this atom for VerificationBasic.

## 7. Rollout

QUEUE `#314` → Analyze-350 → PR; next `#315` result-by-problem authority.
