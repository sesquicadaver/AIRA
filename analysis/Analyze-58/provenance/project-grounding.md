# Project grounding — Analyze-58 / QUEUE #23

**Date:** 2026-08-01

## Canon
| Source | Says |
|--------|------|
| QUEUE #23 | Durable relay hub registry on disk; reload after restart test; Out STUN / session crypto |
| A-44 Out | deferred **persistent disk session store** |
| A-44 Principles | Live hub + register + deliver; in-memory sessions |
| peer-link | durable relay (#23) planned |

## Physics
TCP/Noise live routes die on process exit. #23 cannot restore deliver without re-hold. Scope = **durable registry** (who registered / last_seen), not session resurrection.

## Proposed crystallize (default)
1. File: `peers/relay_hub.json` (schema `aira:peer:relay-hub:v1`)
2. On `register`: upsert `{identity_id, last_seen, online:true}` + save
3. On `unregister`: set `online:false`, refresh `last_seen`, save (keep history — like discovery journal)
4. `RelayHub::load(root)` / open-with-root: load durable list; **live routes empty**
5. Test: register → drop hub → load → durable contains id; live `deliver` fails until re-register
6. Out: STUN; rehydrate live sessions; multi-hop; queue undelivered envelopes

## Residual for user
Confirm disconnect keeps history (`online:false`) vs hard-delete row.
