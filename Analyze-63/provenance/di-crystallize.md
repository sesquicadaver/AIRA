# DI crystallize — Analyze-63 / QUEUE #28

**Chosen:** **A** (same `publisher_id`, new key; revoke = unload + delete dir)

- `rotate_csu_tenant_signing` + CLI `identity csu-tenant rotate [--backup]`
- `revoke_csu_tenant_signing` + CLI `identity csu-tenant revoke`
- Audit actions `tenant_rotate` / `tenant_revoke` (source `csu-tenant`)
- `register --force` for overwrite; default refuse
- Rename commit: secret before meta
- Out: TrustStore CRL, grace, publisher rename, HTTP authz (#29)

**Operator rule:** hereafter recommend only the best option (no peer worse alternatives).
