# AIRA-RFC-0165 — Unique Desktop identity (RFC-D)

## 1. Summary

Phase Q `#276`: each new Desktop install creates an install-scoped persistent identity id (`aira:identity:desktop.<uuid>`). Display name remains `desktop`. Existing roots with on-disk identity files are not silently migrated. `TrustStore::upsert` refuses replacing an already-trusted id with a different public key (`KeyCollision`); use `rekey`/`rotate` for intentional changes. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`ensure_local_identity` always wrote `aira:identity:desktop` while generating a fresh key. Two clean installs shared the addressing id; TrustStore upsert silently overwrote peer keys for that id.

## 3. Motivation

Post-P audit §3 / `phase-q-plan` `#276`: multi-node Desktop requires distinct identity IDs and explicit collision handling.

## 4. Scope

- `new_desktop_identity_id` / `LEGACY_DESKTOP_IDENTITY_ID` / `read_local_identity_id`
- `ensure_bootstrap` / `ensure_local_identity` unique first-run path
- `TrustStore::upsert` KeyCollision on pubkey mismatch; `ensure_trust_defaults` uses `rekey` after rotate
- Two-root bootstrap tests; upsert collision unit test
- QUEUE → `#277`

## 5. Non-Goals

```text
Model light observe (#277)
Silent migration of legacy `aira:identity:desktop` installs
New trust system / second keyring
```

## 6. Compatibility / Security

Legacy installs keep their on-disk id. Invite import that collides on id≠key fails closed.

## 7. Rollout

QUEUE `#276` → Analyze-311 → PR; next `#277` Model light observe.
