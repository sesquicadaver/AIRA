# Ralph evidence — Analyze-27

## Changed files
- `crates/aira-object/src/crypto.rs` — rotate, supersedes fields, errors, trust tests
- `crates/aira-cli/src/main.rs` — `TrustCommands::Rotate`
- `docs/crypto.md`
- `Analyze-27/**`

## Tests
- `trust_rotate_revokes_old_trusts_new` — PASS
- `cargo test -p aira-object --lib crypto::tests` — 8 PASS
- `cargo test --workspace` — PASS
- `cargo clippy -p aira-object -p aira-cli --all-targets -- -D warnings` — PASS
