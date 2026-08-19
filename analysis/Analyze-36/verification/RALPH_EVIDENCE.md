# Ralph evidence — Analyze-36

## Commands
- `cargo test -p aira-peer --lib` → **15 passed**
- `cargo clippy -p aira-peer -p aira-cli -- -D warnings` → **ok**
- `cargo fmt -p aira-peer -p aira-cli` → **ok**

## Changed
- `crates/aira-peer/src/trust_delta.rs` (new)
- `crates/aira-peer/src/lib.rs` — exports + tests
- `crates/aira-cli/src/main.rs` — `trust-send`, `--apply-trust`
- `docs/peer-link.md`, `docs/crypto.md`
- `Analyze-36/**`, `QUEUE.md`, `README.md`, related TODOs
