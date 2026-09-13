# AIRA-RFC-0220 — Activate evidence authority (RFC-D)

## 1. Summary

QUEUE `#336` (Phase W / Pack 1 residual honesty): `VerifiedPointer` is **locator-only**. Activation authority is the primary signed `model-verify-evidence` at `evidence_artifact_id`. Model/hash/`verified=true` must come from that evidence; pointer fields that disagree reject before any cache write.

## 2. Problem Statement

`activate_verified` trusted unsigned `VerifiedPointer.content_hash` / `model_ref` without resolving or verifying `evidence_artifact_id`. Changing weights **and** the pointer hash passed RFC-0212 checks and published fresh activate evidence (audit D5 / A5).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` A5/D5; parent RFC-0215 reserved until `#342`.

## 4. Scope

- Resolve + cryptographically verify `model-verify-evidence` before copy
- Locator integrity: pointer `model_ref` / `content_hash` / `verified_path` must match evidence
- Source/post-copy hash vs evidence `observed_hash` (RFC-0212 continuity retained)
- `AcquisitionError::ActivateEvidenceAuthority` on evidence/locator failure
- Tamper matrix tests (weights; hash; weights+hash; model_ref; evidence ref; signature)

## 5. Non-Goals

```text
Production trust / no implicit local-test (#337)
Safe weights materialization (#338)
Backend verified binding (#339)
Gate rewrite of ActivatedPointerGate
```

## 6. Compatibility

Honest verify→activate paths unchanged. Hand-mutated pointers / forged evidence fail closed.

## 7. Acceptance

```text
weights-only / hash-only / weights+hash / model_ref / missing evidence / bad signature → no activated.latest.json
Honest path → activate OK
Tip → first OPEN #337
```

## 8. References

- QUEUE `#336` · Analyze-373
- Parent: RFC-0215; prior RFC-0219; continuity RFC-0212; Phase L RFC-0112 (gate)
