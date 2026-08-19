# Ralph evidence — Analyze-35

## Commands
- `cargo test -p aira-peer --lib` → **11 passed**
- `cargo clippy -p aira-peer -p aira-cli -- -D warnings` → **ok**
- `cargo fmt -p aira-peer -- --check` → **ok**

## Changed
- `crates/aira-peer/src/noise.rs` (new) — XX + encrypt + `local.x25519` @ 0600
- `handshake.rs` — hello domain `aira:peer:hello:v1` + signed `x25519_pub_hex`
- `session.rs` — XX after hello + `ensure_noise_static_bind` + encrypted envelopes
- `lib.rs` — exports + bind/mode tests
- `Cargo.toml` / workspace — `snow`, `x25519-dalek`
- `docs/peer-link.md`, `docs/crypto.md`
- `Analyze-35/**`, `QUEUE.md`, `README.md`
