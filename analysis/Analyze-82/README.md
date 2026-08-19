# Analyze-82 — modularize crypto.rs (QUEUE #47)

## Status
CLOSED (QUEUE #47 DONE @ b91a5b1 / PR #10).

## Done when
`crates/aira-object/src/crypto.rs` is split into `crypto/{error,keyring,trust_store,rotation}.rs` + `mod.rs`. Object/crypto tests green. `tenant.rs` unchanged.

## Out
Split of `tenant.rs` (#48); CLI; HTTP.
