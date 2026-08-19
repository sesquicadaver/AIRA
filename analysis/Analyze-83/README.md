# Analyze-83 — modularize tenant.rs (QUEUE #48)

## Status
OPEN (PR in flight).

## Done when
`crates/aira-object/src/tenant.rs` is split into `tenant/{paths,map,persist,ceremony,prune}.rs` + `mod.rs`. Tenant tests green. `crypto/` unchanged. HTTP authz semantics unchanged.

## Out
Rewrite of `crypto.rs`; HTTP authz; new tenant behavior.
