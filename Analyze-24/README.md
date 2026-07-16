# Analyze-24 — Trust Keyring Unload / Sync

**Scope:** After `trust remove`, process keyring must stop verifying that key_ref (no same-process stale trust).

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. `trust.json` is source of truth for peer verifying keys in-process
2. Never unload `aira:identity:local-test`
3. Keys with signing material keep verifying material derived from signing
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `sync_trust_verifiers(root)` prunes process verifying keys not present in trust.json, then re-registers trust entries
- **Why:** Additive `register_keyring` alone cannot revoke; CLI remove must sync
- **Alternatives:** Only `unregister_verifying(one)` (weaker if multiple removals / file edits); full process ring reset (too destructive)
- **Follow-ups:** rotation ceremonies, CRL, per-CSU publisher identity

### Acceptance
- Unit test: add peer → verify OK → remove + sync → verify UnknownKey
- CLI `trust remove` calls sync; local-test refuse unchanged
- workspace tests + clippy PASS
