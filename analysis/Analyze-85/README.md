# Analyze-85 — modularize tls.rs (QUEUE #50)

## Status
CLOSED (QUEUE #50 DONE @ 061c535 / PR #13).

## Done when
`crates/aira-node/src/tls.rs` is split into `tls/{paths,pem,verifier,serve}.rs` + `mod.rs`. TLS/mTLS/health-bind tests green. `http/` unchanged. No new TLS modes.

## Out
HTTP route split (#49 already); new TLS modes; health-bind policy change.
