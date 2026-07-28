# Ralph evidence — Analyze-38

## Commands
- `cargo test -p aira-peer --lib` → **17 passed**
- `cargo clippy -p aira-peer -p aira-cli -- -D warnings` → **ok**

## Changed
- `crates/aira-peer/src/trust_delta.rs` — `Rekey` op + apply upsert
- `crates/aira-peer/src/notify.rs` — notify-before-rotate helpers
- `crates/aira-cli` — `--notify-peers`, `peer notify-rekey --pubkey-hex`
- docs + QUEUE + Analyze-38/**
