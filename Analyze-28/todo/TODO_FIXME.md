# TODO_FIXME — Analyze-28

## Deferred
- [ ] Per-CSU publisher identity distinct from node primary
- [ ] Node signing-secret rotate (`local.ed25519`)
- [ ] CRL audit / SQLite event log
- [ ] YAML config (older backlog)

## Done
- [x] `grace_until` on RevokedEntry
- [x] `rotate(..., grace_until)` + `to_keyring_at`
- [x] sync keeps active grace keys
- [x] CLI `trust rotate --until`
- [x] upsert(old) still RevokedKey during grace
