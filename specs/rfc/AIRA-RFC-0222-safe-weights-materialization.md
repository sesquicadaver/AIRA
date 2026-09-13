# AIRA-RFC-0222 — Safe weights materialization (RFC-D)

## 1. Summary

QUEUE `#338` (Phase W / Pack 1 residual honesty): model weights materialization (quarantine / verify / activate) **MUST** use no-follow opens, a bounded stream buffer (no full-file 2×W `read`), and a post-copy hash of the actual dest bytes. Symlink or hash mismatch is fail-closed; partial/symlink paths must not leave a ready model.

## 2. Problem Statement

Acquisition paths used `fs::read` + `fs::copy` + `fs::read` (full buffers) and followed symlinks. A dest symlink under `models/…` could write outside the scoped tree before `ensure_under_models`. Verify copied to verified without re-hashing dest (TOCTOU). Audit A7 / S8 / S18.

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` A7; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `materialize_weights_nofollow` / `content_hash_nofollow` (`O_NOFOLLOW` on Unix; symlink reject; 64KiB buffer; post-copy dest hash)
- Wire quarantine fetch, verify promote, activate cache copy
- `AcquisitionError::{SymlinkRejected,MaterializeHashMismatch}`
- Tests: dest/source symlink escape; expected mismatch removes dest; honest path

## 5. Non-Goals

```text
Backend verified binding (#339)
Reuse candidate check (#340)
GPU marketplace / Pack 2 GUI
Changing ContentHash::sha256_path API globally
```

## 6. Compatibility

Honest regular-file materialize paths unchanged functionally. Symlink materialize paths that previously followed now fail closed.

## 7. Acceptance

```text
Dest symlink → SymlinkRejected; outside target unchanged; no ready pointer
Source symlink on activate → SymlinkRejected; no activated pointer
Post-copy / expected hash mismatch → dest removed; fail-closed
Honest stream path → quarantine/verify/activate OK
No full-file double buffer on materialize path
Tip → first OPEN #339
```

## 8. References

- QUEUE `#338` · Analyze-375
- Parent: RFC-0215; prior RFC-0221; Phase L / Q materialize honesty
