# Ralph evidence — Analyze-25

## Changed files
- `crates/aira-object/src/crypto.rs` — RevokedEntry, revoke, RevokedKey/ProtectedIdentity, sync CRL
- `crates/aira-object/src/lib.rs` — exports
- `crates/aira-cli/src/main.rs` — trust revoke + list CRL
- `docs/crypto.md`
- `Analyze-25/**`

## Tests
- `crypto::tests::trust_crl_revoke_blocks_readd_and_verify` — PASS
- `cargo test --workspace` — PASS
- `cargo clippy -p aira-object -p aira-cli --all-targets -- -D warnings` — PASS
