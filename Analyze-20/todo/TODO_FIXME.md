# TODO_FIXME — Analyze-20

## Deferred
- [ ] Verify signatures against `aira identity create` key files
- [ ] Multi-key trust store / key rotation
- [ ] Re-sign remaining schema-only fixtures that still contain TESTSIG (non-admission paths)
- [ ] Remove `aira-object` example `print_local_sig` if unused long-term

## Done this cycle
- [x] Deterministic local-test Ed25519
- [x] Admission-path verify (artifact/event/CSU/protocol/invariant)
- [x] Reject TESTSIG on admission
