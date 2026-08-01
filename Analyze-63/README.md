# Analyze-63 — Tenant rotate / revoke ceremony

**QUEUE:** #28  
**Status:** CLOSED @ `1f560d8`  
**Decision:** Option **A** — same `publisher_id`, new key; revoke = unload + delete dir

## Shipped
- `rotate_csu_tenant_signing` / `revoke_csu_tenant_signing`
- `register --force`; secret-before-meta rename; one publisher ↔ one CSU
- `unregister_verifying` (never primary / local-test)
- Audit `tenant_rotate` / `tenant_revoke`
- CLI `identity csu-tenant rotate|revoke`

## Out
HTTP authz (#29); tenant `.prev` prune / stdin secret (#36)
