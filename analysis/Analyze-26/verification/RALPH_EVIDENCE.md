# Ralph evidence — Analyze-26

## Changed files
- `crates/aira-object/src/crypto.rs` — `unrevoke`, `NotRevoked`, test
- `crates/aira-cli/src/main.rs` — `TrustCommands::Unrevoke`
- `docs/crypto.md`
- `Analyze-26/**`

## Tests
- `trust_crl_unrevoke_allows_explicit_readd` — PASS
- `cargo test -p aira-object --lib crypto::tests` — 7 PASS
- `cargo test --workspace` — PASS
- `cargo clippy -p aira-object -p aira-cli --all-targets -- -D warnings` — PASS
