# Analyze-22 — Plane Node Signing

**Scope:** OperationalPlane / pipeline emits sign with node identity when registered

## Ralplan (APPROVED — consensus)

### Principles
1. Default signer remains `aira:identity:local-test` when no node identity
2. When node identity is loaded, it becomes **primary** for `local_identity` / `local_signature*`
3. `LocalSession` must register identity **before** constructing/rebuilding the plane
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** Process `primary_signer` + redirect `aira_csu::support::local_*` to `active_signature` / `active_identity`
- **Why:** Single choke-point covers plane, runtime, make_event/make_artifact without rewriting every CSU
- **Alternatives:** Pass signer through every CSU constructor (deferred)
- **Follow-ups:** Multi-identity trust; per-CSU publisher identity

### Acceptance
- With node identity: submit → events/artifacts have `key_ref` / `producer_identity` = node id
- Without node identity: behavior unchanged (local-test)
- `cargo test --workspace` + clippy PASS; originals untouched
