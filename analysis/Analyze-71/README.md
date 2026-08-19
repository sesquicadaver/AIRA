# Analyze-71 — Tenant `.prev` prune (QUEUE #36)

## Status
CLOSED (implementation; QUEUE hash in close commit).

## Shipped
- `identity csu-tenant backups` + `… prune --keep/--older-than-days/--dry-run`
- Per-tenant retain (numeric unix stamps); never latest `.prev`
- QUEUE #37 appended (stdin secret)

## Out
stdin; HTTP; node `identity backups prune`; auto-prune; `--csu-id`
