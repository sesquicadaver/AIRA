# AIRA-RFC-0166 — Model light observe (RFC-D)

## 1. Summary

Phase Q `#277`: Desktop status refresh and mesh/model triple reload use **light** activate observation. Full `fs::read` + sha256 of model weights runs only on observe-ready cache miss (or metadata/pointer change). Generate-local **admission** (`check_activated`) always full-hashes weights. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`ActivatedPointerGate::observe` (used by ~2s status refresh and Start/Stop apply → mesh reload) hashed entire weight files every tick, blocking UI and burning I/O while admission already re-verifies before execution.

## 3. Motivation

Post-P audit / `phase-q-plan` `#277`: monitoring-grade ready must not equal admission-grade hashing on every poll.

## 4. Scope

- `VerifyMode::ObserveLight` vs `AdmitFull` in `activate_gate`
- Versioned `models/activated.observe-ready.json` bound to pointer fingerprint + cache len/mtime + content_hash
- Evidence artifact always verified on observe
- Tests: second observe skips full hash; admit always hashes; len/mtime invalidation
- QUEUE → `#278`

## 5. Non-Goals

```text
Model verify context / reopen keyring (#278)
Weakening admission or evidence checks
Root-scoped verification context changes
```

## 6. Compatibility / Security

Observe-ready is monitoring-only. Same-size same-mtime content swap can leave observe `ready` until admit; execution path remains fail-closed. Evidence JSON is still checked every observe.

## 7. Rollout

QUEUE `#277` → Analyze-312 → PR; next `#278` Model verify context.
