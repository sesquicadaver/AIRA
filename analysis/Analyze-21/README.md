# Analyze-21 — Identity Keyring

**Scope:** Verify/sign using keys from `aira identity create` (post Alpha.2)

## Ralplan (APPROVED — consensus)

### Principles
1. Keep deterministic `aira:identity:local-test` always available
2. Node identity from `.aira/identity/` registers into a process keyring used by `verify_ed25519`
3. CLI can sign/verify with the node key; LocalSession installs keyring on open
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `Keyring` in `aira-object` + process registry; `verify_ed25519` resolves `key_ref`
- **Why:** Minimal change to admission call sites; local node is single-process
- **Alternatives:** Thread Keyring through every store (deferred); global only local-test (status quo)
- **Follow-ups:** Multi-identity trust store, plane signing with node key for all CSU emits

### Acceptance
- Load identity JSON + `local.ed25519` → verify signatures with that `key_ref`
- Unknown `key_ref` still fails; local-test still works
- `aira identity sign|verify` works against node root
- `cargo test --workspace` + clippy PASS; originals untouched
