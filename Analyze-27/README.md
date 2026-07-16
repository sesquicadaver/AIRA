# Analyze-27 — Minimal Trust Rotate

**Scope:** Atomic `trust rotate`: revoke old + trust new with `supersedes` / `superseded_by` metadata. No dual-key verify window.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Rotate is atomic on TrustStore (old CRL + new entry)
2. Old signatures stop verifying immediately after sync (no grace window)
3. Never rotate involving `local-test`
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `TrustStore::rotate` + CLI; metadata fields only; verify path unchanged
- **Why:** Peer key replacement without manual revoke+add race; builds on CRL
- **Alternatives:** Dual-key window (deferred); only docs recipe (no atomicity)
- **Follow-ups:** dual-key grace; node secret rotate; per-CSU publisher

### Acceptance
- rotate old→new: old UnknownKey, new OK; old.superseded_by=new; new.supersedes=old
- protected local-test; same-ref error; old must be trusted
- workspace tests + clippy PASS
