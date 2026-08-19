# Analyze-41 — Timestamped `.prev` history

**Scope:** On `--backup` rotate, archive the previous `local.ed25519.prev` (+ meta) under a UTC-compact timestamp instead of overwriting history. Canonical `.prev` remains the latest slot. CLI list. No Manifesto/Meditation.

**Status:** CLOSED — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Never silently destroy recoverable previous secrets when operator opted into `--backup`
2. Keep `identity/local.ed25519.prev` as **latest** (A-31 compat)
3. Secrets stay mode `0600`; meta has no secret bytes
4. Listable history for operators

### Decision Drivers
1. QUEUE #8 / A-31 deferred
2. Minimal change to staging/rename-after-success contract

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. Archive existing `.prev` → `.prev.<UTC>` before commit** | Compat; small diff | Many files in identity/ |
| B. Always write only into `prev/` dir | Cleaner layout | Breaks A-31 path |
| C. Single tar/jsonl of secrets | Compact | Worse ops / secret packing |

**Chosen: A.**

### ADR
- **Decision:** Before renaming tmp→`.prev`, if `.prev` exists, rename to `local.ed25519.prev.<YYYYMMDDTHHMMSSZ>` (+ matching `.meta.json`), preferring stamp from existing meta `backed_up_at`. Collision → suffix `-N`. Add `list_node_secret_backups` + CLI `identity backups`.
- **Reject:** deleting old slot; changing default rotate (still no backup without flag)
- **Follow-up:** retention/prune policy (out of scope)

### Acceptance
- [x] Two `--backup` rotates retain both secrets (latest + archived)
- [x] Canonical `.prev` is always the most recent backup
- [x] CLI lists history (stamp, pubkey, path)
- [x] Existing backup fail-closed / preserve-slot tests still green
- [x] docs + QUEUE
