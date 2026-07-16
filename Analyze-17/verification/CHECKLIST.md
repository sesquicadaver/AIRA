# Verification Checklist — Analyze-17

## Immutability
- [x] Manifesto etc / Meditation_About unchanged

## Epic 10
- [x] Protocol Envelope + Response schemas/fixtures
- [x] Invalid unsigned envelope rejected
- [x] AIRA-EP adapter (idempotent + unsupported version)
- [x] AIRA-AP adapter (publish/resolve/hash)
- [x] Identity Descriptor schema-valid
- [x] Discovery Capability (not Node)

## Commands

```bash
cargo test -p aira-protocol -p aira-schema
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p aira-cli -- schema validate --fixtures fixtures
bash Analyze-17/verification/deny-originals.sh
bash Analyze-17/verification/deny-foreign-workspace.sh
```

## Result

```text
cargo test -p aira-protocol → 6 passed
aira-schema fixtures → OK (incl. new protocol/identity)
clippy -D warnings → OK
deny-originals / deny-foreign → OK
```

**Verdict:** PASS
