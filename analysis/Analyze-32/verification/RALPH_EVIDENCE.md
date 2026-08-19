# Ralph evidence — Analyze-32

## Commands
```bash
cargo test -p aira-peer
# 7 passed

cargo clippy -p aira-peer -- -D warnings
# Finished
```

## Review fixes
- Strict `payload_hash` verify (no LOCAL_TEST domain fallback on wire)
- 10s timeouts on dial/accept/handshake/frames
- `listen` loopback-only + `listen_explicit`
- Stronger untrusted test; truncated frame + non-loopback tests

## Changed files
- `crates/aira-peer/**` (new)
- `Cargo.toml` workspace member
- `docs/peer-link.md`, `docs/crypto.md`, `docs/local-node.md`
- `Analyze-32/**`
