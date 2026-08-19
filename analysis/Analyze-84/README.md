# Analyze-84 — modularize http.rs (QUEUE #49)

## Status
OPEN (PR in flight).

## Done when
`crates/aira-node/src/http.rs` is split into `http/{state,auth,util,handlers}.rs` + `mod.rs`. HTTP tests/behavior unchanged. `tls.rs` unchanged. No new routes.

## Out
Split of `tls.rs` (#50); new HTTP routes; authz semantics.
