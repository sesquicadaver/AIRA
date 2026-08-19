# DI crystallize — Analyze-64 / QUEUE #29

**Chosen:** **A** — CSU-route authz by `publisher_identity`

- Map: `identity/http-tenant-auth.json` (Bearer → publisher_id)
- mTLS CN = publisher principal when present
- Admin: existing `--http-token` not in map → full access
- `POST /v1/csu/register` → 403 if publisher mismatch
- `GET /v1/csu` → filter for tenant; admin sees all
- `/health` unchanged; problems/events node-scoped this row
- Out: federation (#35), tenant `.prev` prune (#36)
