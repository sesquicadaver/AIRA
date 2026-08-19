# Ralplan — Analyze-63 / QUEUE #28 (rev1 after Architect ITERATE)

## Principles
1. Ceremony ≠ raw `save` (dedicated rotate/revoke APIs).
2. Same `publisher_id` across rotate (event history stable).
3. Fail closed; public audit only; no TrustStore CRL coupling.
4. One publisher_id → at most one CSU in the process tenant map.

## Decision
Option A + Architect B1–B10 (B4 = defer prune to QUEUE #36).

## Scope
1. `unregister_verifying(publisher) -> bool` — remove process verifying keys; **never** drop `primary_signer` / `LOCAL_TEST_KEY_REF` (return false).
2. `save_csu_tenant_signing(..., force: bool)` — refuse existing dir unless force; rename **secret then meta**; refuse duplicate publisher across other CSUs.
3. `rotate_csu_tenant_signing(root, csu, new_sk, backup)`:
   - require existing tenant; keep meta.publisher_id; refuse shared publisher (invariant via map)
   - backup: `ed25519.prev` + meta; archive prior latest to `.prev.<stamp>`
   - commit secret→meta; register; audit **after** commit
4. `revoke_csu_tenant_signing(root, csu, reason: &str)` — reason non-empty required; unregister; drop verifying iff no other tenant shares publisher; delete dir; audit
5. CLI: `rotate [--backup] [--secret-hex]`, `revoke --reason`, `register --force`
6. Audit: `TenantRotate` / `TenantRevoke`, source=`csu-tenant`
7. Docs: revoke = signing-side only (historical verify may remain if pubkey still in trust)
8. QUEUE append #36: tenant `.prev.<stamp>` prune (out of this row)
9. Living Spec + TODO_FIXME

## Out
HTTP authz, grace, CRL, publisher rename, tenant prune CLI, stdin secret file.

## Tests (must)
rotate happy / missing / backup archive / audit after commit; revoke + shared publisher keep verifying; revoke never drops primary; register refuse overwrite + force; register refuse duplicate publisher; save secret-first regression.

## Critic gate
APPROVE required before code.
