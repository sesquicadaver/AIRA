# Analyze-40 — CRL / ceremony audit log

**Scope:** Durable append-only log of trust ceremony ops (`revoke` / `unrevoke` / `rotate` / `rekey` / `node_rotate`). No secrets. No Manifesto/Meditation.

**Status:** CLOSED — APPROVE / CLEAR

## Ralplan (APPROVED — consensus)

### Principles
1. Append-only durable record under node identity dir
2. Never log signing secrets (pubkey hex OK)
3. All mutation paths that change CRL / peer trust / node secret must record
4. CLI must list the log
5. No rusqlite in `aira-object` this slice (SQLite event-log remains deferred)

### Decision Drivers
1. QUEUE #7 / A-26…30 deferred
2. Existing JSON files under `.aira/identity/`
3. Peer apply + CLI + node rotate share TrustStore / rotate helpers

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. JSONL `trust-audit.jsonl` in aira-object** | No new dep; matches identity layout; testable | Not SQL-queryable |
| B. SQLite table in aira-core | Queryable | Couples ceremony to object DB; heavier |
| C. Only CLI-side logging | Small | Misses peer-delta / library callers |

**Chosen: A.** SQLite ceremony table → later if needed.

### ADR
- **Decision:** `TrustAuditLog` / `TrustAuditEntry` at `.aira/identity/trust-audit.jsonl`. Append after successful persist from CLI trust revoke/unrevoke/rotate, `apply_trust_delta` (incl. rekey), and `rotate_node_signing_secret`. CLI `identity trust audit`.
- **Reject:** silent best-effort drop of audit errors without surfacing; logging secrets
- **Follow-up:** SQLite audit table; timestamped `.prev` history (QUEUE #8)

### Acceptance
- [x] revoke/unrevoke/rotate write audit lines
- [x] peer-delta apply writes audit with `source=peer-delta`
- [x] node rotate writes `node_rotate`
- [x] CLI `identity trust audit` lists entries
- [x] No secrets in log
- [x] Tests + clippy; docs + QUEUE
