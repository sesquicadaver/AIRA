# Ralplan — Analyze-62 (QUEUE #27) — revision 1

## Principles
1. Secrets on disk under `identity/tenants/<encoded-csu-id>/`; signing only in tenant map
2. Mode `0600` on `ed25519`; meta = publisher_id + public_key_hex + csu_id
3. `save` / `load` / `load_all` + existing isolation
4. Rehydrate **after** trust sync (not before): `register_node_identity` → `ensure_trust_defaults` → `load_all_csu_tenant_signing`
5. Same rehydrate on `LocalSession::open` **and** `submit_problem`
6. `sync_trust_verifiers` preserves verifying keys for publishers currently in the tenant map
7. `load(csu)` missing → Err; `load_all` missing tenants dir → Ok(0)
8. Bijective filesystem encoding for csu_id (percent-encode / decode)
9. Out: #28 ceremony

## Layout
```
identity/tenants/<percent-encoded-csu-id>/
  ed25519      # hex secret, 0600
  meta.json    # { "csu_id", "publisher_id", "public_key_hex" }
```

## API / CLI
- `save_csu_tenant_signing(root, csu, publisher, sk)`
- `load_csu_tenant_signing(root, csu)`
- `load_all_csu_tenant_signing(root) -> usize`
- `list_csu_tenant_signing(root) -> Vec<…>`
- CLI: `identity csu-tenant register|list|load`

## Test-spec
1. save → reset_csu_tenants → load → sign works
2. load_all multi-tenant isolation
3. corrupt/meta mismatch → Err
4. 0600 on unix
5. open + submit_problem rehydrate after trust sync (verifier still present)
6. load_all empty → 0
7. sync preserves in-memory tenant publishers

## Out
ceremony; TrustStore redesign; dumping secrets into process signing Keyring
