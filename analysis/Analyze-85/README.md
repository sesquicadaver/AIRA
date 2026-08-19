# Analyze-85 — modularize tls.rs (QUEUE #50)

## Status
OPEN (PR in flight).

## Done when
`crates/aira-node/src/tls.rs` is split into `tls/{paths,pem,verifier,serve}.rs` + `mod.rs`. TLS/mTLS/health-bind tests green. `http/` unchanged. No new TLS modes.

## Out
HTTP route split (#49 already); new TLS modes; health-bind policy change.
