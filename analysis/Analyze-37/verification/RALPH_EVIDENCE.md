# Ralph evidence — Analyze-37

## Commands
- `cargo test -p aira-object --lib` → **26 passed**
- `cargo test -p aira-peer --lib` → **15 passed**
- `cargo clippy -p aira-object -p aira-cli -p aira-peer -- -D warnings` → **ok**

## Changed
- `crates/aira-object/src/crypto.rs` — multi-verify Keyring; `rotate_node_signing_secret(..., grace_until)`; sync reloads node grace
- `crates/aira-cli/src/main.rs` — `identity rotate --until`
- `docs/crypto.md`, `QUEUE.md`, `README.md`, A-30/A-31 TODOs
- `Analyze-37/**`
