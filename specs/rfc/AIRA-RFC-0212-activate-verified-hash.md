# AIRA-RFC-0212 — activate_verified hash continuity (RFC-D)

## 1. Summary

QUEUE `#327` (Phase V / Repair Pack 1): `activate_verified` must reject activation when source or post-copy bytes do not match `VerifiedPointer.content_hash`. Fail-closed: no `activated.latest.json`, no ModelInstalled activate Evidence on mismatch.

## 2. Problem Statement

Pre-`#327`, activate hashed the cache copy and wrote that hash into the activated pointer **without** comparing to the verify-time hash. A tampered verified file (or forged pointer hash) could still activate under a new content identity.

## 3. Motivation

`aira-repair.md` Pack 1; [`docs/phase-v-plan.md`](../../docs/phase-v-plan.md); parent consolidating RFC-0208 reserved until `#330`.

## 4. Scope

- `AcquisitionError::ActivateHashMismatch`
- Pre-copy source hash == `VerifiedPointer.content_hash`
- Post-copy dest hash == same expected hash (else delete dest + Err)
- Invalid/unparseable pointer hash → mismatch (fail-closed)
- Tests: tampered verified bytes; forged pointer hash; happy path unchanged

## 5. Non-Goals

```text
Capsule↔Output↔Result binding (#328) — DONE @ RFC-0213
E2E Pack 1 suite (#329)
Pack 2 multi-model GUI
GPU marketplace / LLM-in-Core
Weakening observe / activate admission gate (`check_activated`)
```

## 6. Compatibility

Successful verify→activate path unchanged when bytes match. CLI `aira models activate` surfaces the new error via existing map_err.

## 7. Security / Integrity

Verify-time content identity must survive into cache. Activate must not mint a fresh identity that bypasses verify.

## 8. Acceptance

```text
Tamper verified file after verify → ActivateHashMismatch; no activated.latest.json.
Forge VerifiedPointer.content_hash → ActivateHashMismatch.
Honest verify→activate still writes cache + Evidence + activated pointer.
```

## 9. References

- QUEUE `#327` · Analyze-364
- Depends on RFC-0013 (`activate_verified`), Phase D verify pointer
- Parent consolidating: RFC-0208 (file-free until `#330`)
