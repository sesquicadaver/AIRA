# AIRA-RFC-0145 — `next_candidate_port` modular ring wrap (RFC-D)

## 1. Summary

Phase N-fix `#253`: `next_candidate_port` / `next_candidate_port_from_index` use modular arithmetic on `P_AIRA_COUNT`. Fix incorrect `usize::wrapping_sub` + `rem_euclid` when `current` index is before `preferred` after wrap. RFC-0139 stays file-free until `#254`.

## 2. Problem Statement

Walk offset used `cur.wrapping_sub(start).rem_euclid(P_AIRA_COUNT)`, which is wrong for unsigned when `cur < start` (post-wrap). End→begin worked; mid-ring wrap after crossing `P_AIRA_LAST` did not.

## 3. Motivation

Collision walks must stay on the prime ring without skipping or inventing ports.

## 4. Scope

- `(cur + P_AIRA_COUNT - start) % P_AIRA_COUNT + 1` distance
- `from_index`: `(preferred % N + offset % N) % N`
- Unit tests: last→first; current-before-preferred after wrap
- RFC-D this file; QUEUE → `#254`

## 5. Non-Goals

```text
Presence created_at < expires_at / RFC-0139 (#254)
Changing P_AIRA generation
```

## 6. Compatibility / Security

Behavioral fix for wrap path only. No ledger in `aira-core`.

## 7. Rollout

QUEUE `#253` → Analyze-288 → PR; next `#254` Presence temporal + RFC-0139 close.
