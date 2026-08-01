# Analyze-62 — Durable per-CSU signing secrets

**QUEUE:** #27  
**Status:** CLOSED (pending tip hash after push)  
**Decision:** Option **A** — `identity/tenants/<hex(csu_id)>/{ed25519,meta.json}`; auto-load after trust sync

## Shipped
- `save_csu_tenant_signing` / `load_*` / `list_*`
- `sync_trust_verifiers` preserves tenant publishers
- `LocalSession::{open,submit_problem}` rehydrate after trust sync
- CLI `identity csu-tenant register|list|load`

## Out
Tenant rotate/revoke ceremony (#28)
