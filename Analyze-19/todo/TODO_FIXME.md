# TODO_FIXME — Analyze-19

## Deferred (not this cycle)
- [ ] TLS / mTLS for local HTTP
- [ ] Authn/authz beyond loopback trust
- [ ] Persist discovery registry under `.aira/`
- [ ] Wire CSU registry handlers into OperationalPlane at HTTP submit time
- [ ] `conformance run --profile security|alpha` via HTTP
- [ ] Real Ed25519 verify (still TESTSIG in many helpers)

## Anti-stub
No intentional `todo!()` / empty handlers in `aira-node` HTTP routes shipped this cycle.
