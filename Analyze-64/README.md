# Analyze-64 — Multi-tenant HTTP authz

**QUEUE:** #29  
**Status:** CLOSED @ `0aaa314`  
**Decision:** Option **A** — Bearer map → `publisher_id`; CSU register/list authz

## Shipped
- `identity/http-tenant-auth.json` (0600) + `--http-tenant-auth`
- `Principal::{Admin,Tenant,Unscoped}`; map wins over admin on same secret
- `POST /v1/csu/register` → 403 cross-publisher; `GET /v1/csu` filtered
- Boot fail if map present without `--http-token`

## Out
mTLS CN→principal seam; federation (#35); problems/events tenancy; #36 prune
