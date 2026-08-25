# Analyze-164 — Split cli peer (QUEUE #129)

## Status
OPEN — mechanical split `aira-cli/commands/peer.rs` → `peer/{mod,book,dht,stun,discv,session}.rs`.

## Done when
Peer CLI tests green: `cargo test -p aira-cli`; clippy `-D warnings` on `aira-cli`.

## Out
New peer commands; protocol/behavior change.
