# Ralph evidence — Analyze-29

## Changed files
- `crates/aira-object` — `signature_for`
- `crates/aira-csu/src/support.rs` — make_*_as, apply_publisher, test
- `csu/{context,reduction,execution,verification,evidence,artifact}-basic` — publisher emits + with_publisher
- `docs/crypto.md`
- `Analyze-29/**`

## Tests
- `support::tests::publisher_override_signs_distinct_from_primary` — PASS
- `cargo test --workspace` — PASS
- clippy `-D warnings` — PASS
