# TODO_FIXME — Analyze-19

## Deferred (not this cycle)
- [x] TLS for local HTTP — **DONE** in Analyze-45 (`--tls-cert`/`--tls-key`/`--tls-self-signed`)
- [ ] Authn/authz beyond loopback trust / mTLS
- [x] Persist discovery registry under `.aira/` — **DONE** in Analyze-45 (`discovery/registry.json`)
- [ ] Wire CSU registry handlers into OperationalPlane at HTTP submit time
- [ ] `conformance run --profile security|alpha` via HTTP
- [ ] Real Ed25519 verify (still TESTSIG in many helpers)

## Anti-stub
No intentional `todo!()` / empty handlers in `aira-node` HTTP routes shipped this cycle.
