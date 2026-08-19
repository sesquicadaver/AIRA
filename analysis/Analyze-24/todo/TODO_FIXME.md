# TODO_FIXME — Analyze-24

## Deferred
- [ ] Key rotation / revocation lists
- [ ] Per-CSU publisher identity distinct from node primary
- [ ] YAML config / SQLite event log (older backlog)

## Done
- [x] `sync_trust_verifiers` prune absent peers
- [x] Protect local-test; keep signing-derived verifying keys
- [x] CLI `trust remove` calls sync
- [x] `ensure_trust_defaults` uses sync
- [x] Unit test asserts UnknownKey after remove+sync
