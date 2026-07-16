# Analyze-26 — Admin Trust Unrevoke

**Scope:** Clear CRL entry without auto re-trust; explicit `trust add` required after unrevoke.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Unrevoke only removes from `revoked[]` — never restores pubkey/entries automatically
2. Re-trust requires explicit `trust add` / `upsert`
3. Unrevoke of non-revoked id → `NotRevoked` error (not silent)
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `TrustStore::unrevoke` + CLI `trust unrevoke`; no auto-restore from `RevokedEntry.public_key_hex`
- **Why:** Completes Analyze-25 CRL with admin escape hatch without silent re-trust risk
- **Alternatives:** Auto re-add from CRL pubkey (rejected); unrevoke=trust (ambiguous)
- **Follow-ups:** rotation ceremony; audit log; per-CSU publisher

### Acceptance
- revoke → upsert fails → unrevoke → upsert OK → sync → verify OK
- repeat unrevoke → NotRevoked
- docs + workspace tests + clippy PASS
