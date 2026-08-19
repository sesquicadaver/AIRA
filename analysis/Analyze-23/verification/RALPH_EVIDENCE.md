# Ralph evidence — Analyze-23

## Changed files
- `crates/aira-object/src/crypto.rs` — TrustStore/TrustEntry, register/ensure
- `crates/aira-object/src/lib.rs` — re-exports
- `crates/aira-flow/src/local.rs` — trust_json path; ensure on open/submit
- `crates/aira-cli/src/main.rs` — `identity trust list|add|remove`
- `docs/crypto.md`, `docs/local-node.md`
- `Analyze-23/**`

## Tests
- `cargo test --workspace` — PASS
- `crypto::tests::trust_store_peer_verify_without_signing_key` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `bash Analyze-23/verification/deny-originals.sh` — OK

## Smoke (venv/local cargo only)
```bash
ROOT=/tmp/aira-trust-23b
# trust add peer → list shows peer + local-test
# trust remove local-test → refuse (exit 1)
# trust remove peer → list local-test only
```
Evidence: CLI smoke PASS 2026-07-16.
