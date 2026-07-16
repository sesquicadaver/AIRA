# TODO_FIXME — Analyze-23

## Deferred
- [ ] Key rotation / revocation lists
- [ ] Unload verifying key from process keyring on `trust remove` (same-process stale verify)
- [ ] Per-CSU publisher identity distinct from node primary
- [ ] YAML config / SQLite event log (older backlog)

## Done
- [x] TrustStore + trust.json persistence
- [x] register_trust_store / ensure_trust_defaults
- [x] LocalSession loads trust before plane
- [x] CLI `identity trust list|add|remove`
- [x] Refuse remove of local-test
