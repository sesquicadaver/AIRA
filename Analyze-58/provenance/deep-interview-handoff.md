# Deep-interview handoff — Analyze-58

**Date:** 2026-08-01  
**Status:** COMPLETE  
**Ambiguity:** ≤ 0.12

## Interview-complete rationale
Grounded on A-44 Out + QUEUE #23. User chose keep-history on disconnect (**A**) plus **optional TTL** (31 days when enabled) for prune.

## Crystallized

| Item | Decision |
|------|----------|
| Semantics | Durable registry ≠ live sessions |
| File | `peers/relay_hub.json`, schema `aira:peer:relay-hub:v1` |
| Register | upsert `{identity_id, last_seen, online:true}` + save |
| Unregister | `online:false`, refresh `last_seen`, save |
| Reload | load durable list; live routes empty until re-hold |
| TTL | **Optional**. When set (CLI/API), prune entries with `last_seen` older than N days. Recommended value **31**. When unset — no prune (history kept). |
| Done when | reload-after-restart test (+ TTL prune test when enabled) |
| Out | STUN; session crypto / rehydrate live sessions; undelivered queue; multi-hop |

## Acceptance
1. Register → drop hub → load → durable contains id; live deliver fails until re-register  
2. Unregister → durable shows online:false  
3. With TTL=31, entry older than 31d pruned on load/save path  
4. Without TTL — old entries retained  
