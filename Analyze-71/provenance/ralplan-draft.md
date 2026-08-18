# Ralplan — Analyze-71 / QUEUE #36

## Principles
1. Never delete latest `ed25519.prev` / `ed25519.prev.meta.json` or live `ed25519` / `meta.json`.
2. Explicit CLI prune only — no auto-prune on rotate.
3. ≥1 of `--keep` / `--older-than-days` required; retain = intersection; per-tenant (B1).
4. Do not change `identity backups prune` or rotate/revoke/register (A1/C1).
5. Delete I/O fail-closed (abort prune); dry-run deletes nothing. Skip+continue only for undecodable dir / unparseable age / orphan meta.

## Decision drivers
- QUEUE done-when: prune CLI parity with node backups + tests.
- Tenant archives already exist (`archive_latest_tenant_prev`); missing is GC + list.
- Destructive GC of signing secrets → same skip/orphan rules as Analyze-61.

## Viable options
- **A (chosen):** `identity csu-tenant backups` + `… prune` in `tenant.rs`. Pros: operator surface matches tenant, node prune untouched, per-tenant rank. Cons: second prune command to learn.
- **B (rejected, A3):** fold into `identity backups prune` — mixes node and tenant GC.
- **C (rejected, A2):** require `--csu-id` — extra flag, not node parity.

## Stamp / age (must not copy node compact-UTC blindly)
Tenant rotate stamps with **unix seconds** (`ed25519.prev.<secs>`), not `YYYYMMDDTHHMMSSZ`.
- List/rank: numeric descending on stamp (lex sort is wrong for variable-width unix).
- Age: `backed_up_at` RFC3339 if valid; else parse stamp as unix seconds (`-N` suffix ignored for age).
- Unparseable + `--older-than-days` → skip delete + stderr; `--keep` alone still ranks.
- Orphan `ed25519.prev.<stamp>.meta.json` without secret: never delete.
- Secret without meta: prune candidate; age from filename only.

## List contract (`identity csu-tenant backups`)
Tab-separated, newest first per tenant, tenants sorted by `csu_id`:
```text
<csu_id>\t<stamp>\t<old_public_key_hex|- >\t<backed_up_at|- >\t<path>
```
`stamp` = `latest` or filename stamp. Always print summary `tenant_backups <tenants_root>`. If no slots: also print `(no csu tenant backups — use identity csu-tenant rotate --backup)`.

## CLI
- `identity csu-tenant backups` → list
- `identity csu-tenant backups prune [--keep N] [--older-than-days D] [--dry-run]`
- Neither keep nor days → exit 1
- Walk all tenant dirs; skip undecodable dir names (stderr); continue others
- `identity csu-tenant list` unchanged (tenants, not backup slots)

## API
- `CsuTenantBackupInfo { csu_id, stamp, secret_path, meta_path, old_public_key_hex, backed_up_at, is_latest }`
- `list_csu_tenant_secret_backups(root) -> Vec<…>`
- `prune_csu_tenant_secret_backups(root, keep, older_than_days, dry_run) -> NodeSecretPruneReport` (reuse report type) or identical tenant report
- Reuse `pub(crate) should_retain_archived` from `crypto.rs` (do not fork retain math)
- Do not change `rotate_csu_tenant_signing` / `archive_latest_tenant_prev`

## Pre-mortem (destructive)
1. Operator runs `--keep 0` expecting wipe of latest too → latest must survive; test keep=0.
2. Two tenants, keep=1: busy tenant must not starve the other (per-tenant rank).
3. `--older-than-days` + garbage stamp → skip, not delete; live secret intact.

## Tests
1. keep=1 two tenants with 2 archives each → each keeps newest archive + latest; older gone
2. keep=0 → all archives gone; latest `.prev` remains
3. older-than-days skips unparseable stamp; keep-only still deletes by rank
4. dry-run deletes nothing
5. no flags → error
6. orphan meta never deleted
7. latest + live `ed25519` never deleted
8. `identity backups prune` still does not touch tenant files (regression)
9. list columns + `latest` row present after `--backup`
10. numeric rank: stamps `9` and `10` — keep=1 retains `10` not `9`
11. file `ed25519.prev.tmp` is never listed or deleted

## Docs / QUEUE
- `docs/crypto.md` tenant prune section; `docs/local-node.md` if tenant CLI is documented
- QUEUE #36 scope without stdin; append **#37** stdin/`--secret-hex-file`
- Living Spec: honest — tenant archive GC, not node prune unification

## Architect (WATCH folded)
- Placement: `tenant.rs` + dedicated CLI; do **not** fold into `identity backups prune` (unix stamps ≠ compact UTC).
- Name filters = node list: skip `.tmp`, `.meta.json`, empty stamp, stamp containing `.`; never treat `ed25519.prev.meta.json` as orphan-meta. Prevents `--keep` from deleting `ed25519.prev.tmp` (stamp=`tmp`).
- Rank per-tenant: parse stamp as u64 (strip `-N`); `-N` is tiebreak only. Non-numeric stamps sort after all numeric (oldest) so `--keep` still assigns rank.
- Age: RFC3339 `backed_up_at`, else decimal unix seconds (strip `-N`). Do **not** call `compact_stamp_unix` / x25519 `…Z`.
- Delete I/O fail-closed for the whole prune (like node). Skip+continue only: undecodable dir, unparseable age, orphan meta.
- List `csu_id` from `decode_csu_dir_name`, not `meta.json`.
- CLI I/O like node: `skip path\treason` stderr; `deleted`/`would_delete`; always print `tenant_backups <tenants_root>` (empty list still).
- QUEUE: strip stdin from #36; append **#37**.

## Critic
**APPROVE** (00ac55e1) after Architect CLEAR (716b4441). Residual accepted: same-second archive; ops may only run node prune; `--keep 0` all tenants; no rollback of partial deletes.

## Implementation
1. `pub(crate) should_retain_archived` in `crypto.rs` (no retain-math fork).
2. `list_csu_tenant_secret_backups` + `prune_csu_tenant_secret_backups` in `tenant.rs`.
3. CLI nested `CsuTenantCommands::Backups`.
4. Tests + docs + Living Spec + QUEUE #37.
