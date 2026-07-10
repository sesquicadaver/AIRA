# Verification Checklist — Analyze-12

## Immutability
- [x] Manifesto etc / Meditation_About unchanged

## Epic 5
- [x] CsuManifest schema + unsigned reject
- [x] Registry ABI/signature + list
- [x] Lifecycle transitions + events
- [x] Csu trait + outputs
- [x] Active-only dispatch + CSUFailed
- [x] Isolation baseline
- [x] `aira csu list|register`

## Commands

```bash
cargo test -p aira-csu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
bash Analyze-12/verification/deny-originals.sh
bash Analyze-12/verification/deny-foreign-workspace.sh
cargo run -p aira-cli -- csu register --manifest fixtures/valid/csu/manifest.json --registry /tmp/aira-csu-reg.json
cargo run -p aira-cli -- csu list --registry /tmp/aira-csu-reg.json
```

## Result

```text
cargo test -p aira-csu → 7 passed
cargo clippy -D warnings → OK
deny-originals / deny-foreign → OK
CLI csu register/list → OK
Manifesto etc / Meditation_About → unchanged
```

**Verdict:** PASS
