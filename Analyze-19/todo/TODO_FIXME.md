# TODO_FIXME — Analyze-19

## Deferred (not this cycle)
- [x] TLS for local HTTP — **DONE** in Analyze-45 (`--tls-cert`/`--tls-key`/`--tls-self-signed`)
- [x] HTTP Bearer authn — **DONE** in Analyze-48 (`--http-token` / `AIRA_HTTP_TOKEN`; `/health` exempt)
- [x] mTLS client-cert authn — **DONE** in Analyze-51 (`--tls-client-ca`; require mode)
- [x] Persist discovery registry under `.aira/` — **DONE** in Analyze-45 (`discovery/registry.json`)
- [ ] Wire CSU registry handlers into OperationalPlane at HTTP submit time
- [ ] `conformance run --profile security|alpha` via HTTP
- [ ] Real Ed25519 verify (still TESTSIG in many helpers)

## Anti-stub
No intentional `todo!()` / empty handlers in `aira-node` HTTP routes shipped this cycle.
