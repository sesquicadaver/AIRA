# Ralph evidence — Analyze-30

## Commands
```bash
cargo test -p aira-object
# 20 passed (incl. cutover, missing identity, CRL rollback)

cargo clippy -p aira-object -p aira-cli -- -D warnings
# Finished

# CLI smoke (temp root):
# identity create → sign → rotate → old verify FAIL → new verify OK
# trust list shows same id with new pubkey
```

## Changed files
- `crates/aira-object/src/crypto.rs` — `rotate_node_signing_secret` (+ restore on trust failure)
- `crates/aira-object/src/lib.rs` — re-export
- `crates/aira-cli/src/main.rs` — `IdentityCommands::Rotate`
- `docs/crypto.md`, `docs/local-node.md`
- `Analyze-30/**`
