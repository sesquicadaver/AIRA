# Analyze-61 — Retention/prune `.prev.<stamp>`

**QUEUE:** #26  
**Status:** CLOSED (`eeaed25`)  
**Decision:** Option **C** — `--keep` and/or `--older-than-days`; ed25519 + x25519; never latest

## Shipped
- `prune_node_secret_backups` / `prune_noise_static_backups`
- `identity backups` list with `family` column; `identity backups prune`
- Tests + `docs/crypto.md`

## Out
per-CSU secrets (#27); auto-prune on rotate
