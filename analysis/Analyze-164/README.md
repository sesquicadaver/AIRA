# Analyze-164 — Split cli peer (QUEUE #129)

## Status
CLOSED @ eff14c5 / PR #92 (QUEUE #129 DONE).

## Done when
Mechanical split: `commands/peer/{mod,book,dht,stun,discv,session}.rs`; `cargo test -p aira-cli` + clippy green.

## Out
New peer commands; protocol/behavior change.
