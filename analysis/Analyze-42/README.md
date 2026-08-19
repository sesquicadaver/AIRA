# Analyze-42 — Multi-tenant per-CSU keyring

**Scope:** Isolate CSU publisher signing secrets so CSU-A cannot sign as CSU-B’s publisher via the process `Keyring`. Plane/primary unchanged. No Manifesto/Meditation / peer.

**Status:** CLOSED — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Tenant signing material is keyed by `csu_id` (not a free-for-all process signing map)
2. CSU emit helpers always pass `tenant_csu`
3. Non-primary / non-local-test publishers require explicit `register_csu_tenant_signing`
4. Verifying pubkeys may still merge into the process ring (public)
5. Fail closed on cross-tenant or unregistered publisher

### Decision Drivers
1. QUEUE #9 / A-29 deferred isolation
2. Reuse A-29 `publisher_identity` + `make_*_as` call sites
3. Avoid full per-CSU verify Keyring split (heavier, low demo value)

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Tenant signing store + scoped `signature_for_tenant`** | Real isolation; small surface | Call-site updates |
| B. Full Keyring per CSU | Stronger | Heavy; breaks verify UX |
| C. Docs-only | None | Does not close QUEUE |

**Chosen: A.**

### ADR
- **Decision:** `register_csu_tenant_signing(csu, publisher, sk)` stores **signing** only in a process tenant map; registers **verifying** into process Keyring. `make_event_as` / `make_artifact_as` take `tenant_csu` and call `signature_for_tenant`. Unregistered tenants may sign only as `primary_signer` or `local-test`. Cross-tenant publisher mismatch → `TenantIsolation`.
- **Reject:** dumping tenant secrets into process signing map; silent global `signature_for` from CSU emits
- **Follow-up:** durable on-disk per-CSU secrets; retention of tenant rotate

### Acceptance
- [x] Distinct publisher via tenant registration signs + verifies
- [x] CSU-A cannot sign as CSU-B publisher (even if B verifying is in process ring)
- [x] Unregistered non-primary publisher → fail closed
- [x] Default stock CSUs (primary/local-test) still work
- [x] Tests + clippy; docs + QUEUE
