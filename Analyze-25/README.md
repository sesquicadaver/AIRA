# Analyze-25 — Minimal Trust CRL

**Scope:** Durable `revoked[]` in trust.json; block re-add; sync unload revoked peers.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Revoke is durable (unlike `trust remove`)
2. Never revoke `aira:identity:local-test`
3. Re-add of revoked id fails with explicit error
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `TrustStore.revoked` + `revoke()` + CLI `trust revoke`; upsert rejects revoked; sync treats revoked as untrusted (including no signing-derived keep for revoked ids)
- **Why:** Completes Analyze-23/24 trust stack without full rotation ceremony
- **Alternatives:** Full supersedes/dual-key window (deferred); delete-only remove (insufficient)
- **Follow-ups:** rotation ceremony; unrevoke admin; per-CSU publisher

### Acceptance
- revoke → sync → verify UnknownKey; re-upsert → RevokedKey
- list shows revoked; refuse revoke local-test
- workspace tests + clippy PASS
