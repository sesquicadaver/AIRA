# AIRA-RFC-0143 — Presence expiry before AddressBook promote (RFC-D)

## 1. Summary

Phase N-fix `#251`: `promote_presence_to_address_book` and `discover_admit_promote` fail-closed when Presence is expired at `as_of`, including rows still returned by `query_identity`. RFC-0139 stays file-free until `#254`.

## 2. Problem Statement

`query_identity` intentionally returns the latest stored record (may be expired). Ab ovo / promote paths previously could upsert AddressBook from that stale row.

## 3. Motivation

Active discovery must not treat expired Presence as dialable. Ledger lookup ≠ active advertisement.

## 4. Scope

- `promote_presence_to_address_book(root, presence, as_of)` — reject via `is_presence_expired`
- `discover_admit_promote` — expiry check after `query_identity` (before DISCOVERED / promote)
- Tests: trusted+expired → `PeerError::Expired`; book empty; ab ovo path
- RFC-D this file; QUEUE → `#252`

## 5. Non-Goals

```text
netns NAT/firewall (#252)
next_candidate_port wrap (#253)
created_at < expires_at shape (#254) / RFC-0139
Changing query_identity to hide expired rows
```

## 6. Compatibility / Security

API additive `as_of` on promote. `DISCOVERED ≠ TRUSTED`. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#251` → Analyze-286 → PR; next `#252` netns NAT tests.
