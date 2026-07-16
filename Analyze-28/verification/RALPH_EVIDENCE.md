# Ralph evidence — Analyze-28

## Changed files
- `Cargo.toml` / `Cargo.lock` / `aira-object/Cargo.toml` — `time` dep
- `crates/aira-object/src/crypto.rs` — grace_until, to_keyring_at, sync, test
- `crates/aira-object/src/lib.rs` — exports
- `crates/aira-cli/src/main.rs` — `--until`
- `docs/crypto.md`
- `Analyze-28/**`

## Tests
- `trust_rotate_grace_allows_old_until` — PASS
- `cargo test -p aira-object --lib crypto::tests` — 9 PASS
- `cargo test --workspace` — PASS
- clippy `-D warnings` — PASS
