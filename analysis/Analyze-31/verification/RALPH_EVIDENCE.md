# Ralph evidence — Analyze-31

## Commands
```bash
cargo test -p aira-object
# 24 passed (backup writes/fail_closed/preserve slot/dirtrap + rotate suite)

cargo clippy -p aira-object -p aira-cli -- -D warnings

# CLI: rotate (no .prev) then rotate --backup (.prev + meta)
```

## Semantics
- Stage `*.tmp` before overwrite; rename to `.prev` only after successful trust upsert
- Abort cleans tmp only; never rollback after trust on backup-commit issues
- Dir trap on final `.prev` is removed before rename

## Changed files
- `crates/aira-object/src/crypto.rs`, `lib.rs`
- `crates/aira-cli/src/main.rs`
- `docs/crypto.md`, `docs/local-node.md`
- `Analyze-31/**`
