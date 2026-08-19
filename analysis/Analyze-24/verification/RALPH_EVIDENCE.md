# Ralph evidence — Analyze-24

## Changed files
- `crates/aira-object/src/crypto.rs` — `sync_trust_verifiers`; ensure_defaults uses sync; test asserts UnknownKey
- `crates/aira-object/src/lib.rs` — export
- `crates/aira-cli/src/main.rs` — trust remove → sync
- `docs/crypto.md`
- `Analyze-24/**`

## Tests
- `cargo test -p aira-object --lib crypto::tests` — PASS
- `cargo test --workspace` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- originals deny gate — OK
