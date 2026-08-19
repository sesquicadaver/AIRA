# Analyze-82 — modularize crypto.rs (QUEUE #47)

## Status
OPEN (implementation on branch `analyze-82-modularize-crypto`).

## Done when
`crates/aira-object/src/crypto.rs` is split into `crypto/{error,keyring,trust_store,rotation}.rs` + `mod.rs`. Object/crypto tests green. `tenant.rs` unchanged.

## Out
Split of `tenant.rs` (#48); CLI; HTTP.
