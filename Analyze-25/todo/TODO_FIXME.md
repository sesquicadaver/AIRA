# TODO_FIXME — Analyze-25

## Deferred
- [ ] Key rotation ceremony (supersedes / dual-key window)
- [ ] Admin unrevoke path
- [ ] Per-CSU publisher identity distinct from node primary
- [ ] YAML config / SQLite event log (older backlog)

## Done
- [x] `RevokedEntry` + `TrustStore.revoked`
- [x] `revoke` / upsert → `RevokedKey`
- [x] sync unloads revoked (no signing-derived keep)
- [x] CLI `identity trust revoke`
- [x] Refuse revoke local-test (`ProtectedIdentity`)
