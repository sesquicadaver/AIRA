# Analyze-31 — Opt-in Node Secret Durable Backup

**Scope:** Before rotate overwrite, optional write of previous `local.ed25519` to `local.ed25519.prev` (0600).

**Status:** ralplan → ralph → code-review **REQUEST CHANGES** (Cycle 3: backup-commit rollback after trust)

## Ralplan (APPROVED — consensus)

### Principles
1. Default rotate stays Analyze-30 (no durable old secret)
2. `--backup` is operator opt-in; fail closed if backup cannot be written
3. Backup is local FS only — not CRL, not dual-key, not peer notify
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `rotate_node_signing_secret(..., backup: bool)` writes `identity/local.ed25519.prev` (+ small meta JSON) before overwrite
- **Why:** recovery surface after cutover without expanding Keyring
- **Reject:** always-on backup (more secrets on disk by default); dual-key same-ref

### Acceptance
- `--backup`: prev file exists with old secret; perms 0600; rotate succeeds
- Without flag: no `.prev` created
- Backup write failure → no secret overwrite
- On abort after backup staging: only tmp cleaned; existing `.prev` slot preserved; no orphan durable secret from failed attempt
- Staging uses `*.tmp` then rename only after successful rotate
- Trust rollback path still works
- tests + clippy PASS; docs updated
