# Ralplan — Analyze-61 (QUEUE #26) — revision 2 (consensus candidate)

## Principles
1. Never delete canonical latest `local.ed25519.prev` / `local.ed25519.prev.meta.json` / `local.x25519.prev`
2. Explicit CLI prune only — no auto-prune on rotate
3. ≥1 of `--keep` / `--older-than-days` required
4. Retain archived slot iff every supplied policy holds (per family)
5. Per-family independent prune; same CLI flags
6. Fail-closed I/O; dry-run deletes nothing
7. Out: per-CSU (#27)

## List output contract (`identity backups`)
Tab-separated columns (stable):
```text
<family>\t<stamp>\t<old_public_key_hex|- >\t<backed_up_at|- >\t<path>
```
- `family` = `ed25519` | `x25519`
- Existing ed25519 rows gain a leading `ed25519` column (documented breaking display change; fields after family keep prior meaning)
- x25519: `old_public_key_hex` and `backed_up_at` are `-` unless we later add meta (not this slice)
- Final summary line unchanged style: `backups <identity_dir>`

## Age / malformed rules
- **ed25519 archived:** age = parse `backed_up_at` from paired meta if valid RFC3339; else parse filename stamp `YYYYMMDDTHHMMSSZ` (optional `-N` suffix ignored for age → use base before `-N` if needed, or treat whole stamp lexicographically for rank only). If **neither** yields a valid age and `--older-than-days` is set → **skip delete** for that slot, print `skip <path> (unparseable age)` to stderr; do not fail whole command unless I/O error on a delete that was attempted.
- Orphan `.meta.json` without secret file: **never delete** in this slice (report `skip orphan-meta …` on prune when scanning); list may omit orphans.
- Secret without meta: still a prune candidate; age from filename stamp only; if unparseable under `--older-than-days` → skip delete as above. `--keep` rank still applies (no age needed).
- **x25519:** age from filename only as `<unix-secs>Z` (optional `-N` suffix); this matches Analyze-49 `archive_x25519_prev_if_present`. Unparseable + `--older-than-days` → skip delete + stderr.

## CLI
- `identity backups` → list (contract above)
- `identity backups prune [--keep N] [--older-than-days D] [--dry-run]`
- Neither keep nor days → exit error

## API
- `prune_node_secret_backups` / `PruneReport { deleted, skipped, dry_run }`
- `list_noise_static_backups` + `prune_noise_static_backups`

## Test-spec
1–7 as r1 plus:
8. list lines start with `ed25519` / `x25519`
9. orphan meta never deleted
10. unparseable age + `--older-than-days` → skip, not delete; `--keep` alone still prunes by rank
11. mixed-family: one family skip does not block the other

## Out
per-CSU; auto prune on rotate; SQLite
