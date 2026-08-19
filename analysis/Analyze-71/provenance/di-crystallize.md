# DI crystallize — Analyze-71 / QUEUE #36

## In scope (A + A1 + B1 + C1 + F1)
1. CLI `identity csu-tenant backups` (list) + `identity csu-tenant backups prune --keep/--older-than-days/--dry-run`.
2. Walk **all** `identity/tenants/<hex>/`. Policy **per-tenant** (like node per-family).
3. GC only archived `ed25519.prev.<stamp>` (+ paired `.meta.json`). Never latest `ed25519.prev` / `ed25519.prev.meta.json`, never live `ed25519` / `meta.json`.
4. Same retain rule as node: ≥1 flag required; retain = intersection of supplied policies; newest archived rank 0.
5. Logic in `aira-object` `tenant.rs`; CLI thin. `identity backups prune` unchanged.
6. QUEUE #37: stdin/`--secret-hex-file` (append at end).

## Out
stdin/`--secret-hex-file`; HTTP authz; change `identity backups prune`; auto-prune on rotate; `--csu-id`; rotate/revoke/register semantics; TrustStore/CRL; delete latest `.prev` or live secret.

## Decision boundaries (agent-owned)
Stamp age: meta `backed_up_at` RFC3339 else unix-seconds filename (tenant `unix_stamp()`, not compact UTC). Rank by numeric stamp, not lex. Orphan meta never deleted. Unparseable age + `--older-than-days` → skip. Malformed tenant dir → skip+stderr, continue. Reuse `should_retain_archived` via `pub(crate)`.
