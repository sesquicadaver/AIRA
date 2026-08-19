# TODO_FIXME — Analyze-40

## Deferred
- [ ] SQLite ceremony audit table (optional; JSONL is durable now)
- [x] Timestamped `.prev` history (QUEUE #8 → Analyze-41)

## Done
- [x] `TrustAuditLog` / `trust-audit.jsonl`
- [x] CLI revoke/unrevoke/rotate + `identity trust audit`
- [x] `apply_trust_delta` audit (`source=peer-delta`)
- [x] `rotate_node_signing_secret` → `node_rotate`
- [x] docs/crypto.md + QUEUE #7 DONE
