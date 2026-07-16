# Analyze-23 — Multi-Identity Trust Store

**Scope:** Persist trusted verifying public keys; load into process keyring

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Trust store is verifying-only (never stores peer secrets)
2. Path: `.aira/identity/trust.json`
3. local-test + node identity auto-present; peers added via CLI
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** JSON trust file + `register_trust_store` merges verifying keys into Keyring
- **Why:** Extends Alpha.2/21 verify path without Core change
- **Alternatives:** SQLite trust table (deferred); embed in config.json (wrong separation)
- **Follow-ups:** key rotation ceremony, revocation lists

### Acceptance
- `trust add` persists pubkey; verify succeeds for that key_ref
- `trust list` / `remove` work; LocalSession loads trust on open
- Unknown key_ref still fails; workspace tests + clippy PASS

See [INDEX.md](INDEX.md), [LIVING_SPEC_MATRIX.md](LIVING_SPEC_MATRIX.md), [verification/](verification/).
