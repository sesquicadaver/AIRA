# TODO_FIXME — Analyze-27

## Deferred
- [ ] Dual-key rotation grace window
- [ ] Node signing-secret rotate (`local.ed25519`)
- [ ] Per-CSU publisher identity distinct from node primary
- [ ] CRL audit / SQLite event log
- [ ] Process-keyring test isolation (serial_test) — trust tests use file-backed rings

## Done
- [x] `TrustStore::rotate` + supersedes metadata
- [x] CLI `identity trust rotate`
- [x] Protect local-test / SameIdentity / NotTrusted
- [x] Stabilize trust unit tests via `to_keyring()`
