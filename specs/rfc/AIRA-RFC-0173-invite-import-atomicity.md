# AIRA-RFC-0173 — Invite import atomicity (RFC-D)

## 1. Summary

Phase Q `#284`: PeerInvite import runs schema + Prime Port / AIRA-bind checks **before** any TrustStore or AddressBook durable write. Address-book save failure after trust write rolls trust back to the pre-import snapshot. Camera QR scan surfaces decode/import errors instead of swallowing them. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`import_invite` previously wrote TrustStore, then validated/upserted AddressBook. A non-prime `addr` (e.g. `:9797`) returned `Err` after durable trust mutation — silent partial commit.

## 3. Motivation

`phase-q-plan` `#284` / acceptance: invite failure must not leave silent TrustStore mutation; all checks (incl. port invariant) before writes; commit/rollback or explicit partial.

## 4. Scope

- `validate_peer_invite` + pre-write `validate_aira_bind`
- In-memory upserts → saves; trust rollback if book save fails
- Camera: distinguish “no QR” vs decode/import failure
- QUEUE → `#285`

## 5. Non-Goals

```text
Consolidating RFC-0164 (#285)
Changing Prime Port range / transport semantics
Multi-writer concurrent invite import locking
```

## 6. Compatibility / Security

Valid prime-port invites behave as before. Invalid-addr invites fail closed with unchanged trust.json. Trust-only invites (no addr) unchanged.

## 7. Rollout

QUEUE `#284` → Analyze-319 → PR; next `#285` RFC-0164 consolidating close.
