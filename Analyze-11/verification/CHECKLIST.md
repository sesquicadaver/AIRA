# Verification Checklist — Analyze-11

## Immutability
- [x] Manifesto etc / Meditation_About unchanged

## Epic 4
- [x] ArtifactDescriptor schema
- [x] CAS publish/resolve + hash mismatch
- [x] Mutation fails; supersession keeps old
- [x] EventDescriptor schema
- [x] Append-only + query by object/artifact ref
- [x] Subscriptions + idempotent event_id
- [x] Policy ALLOW|DENY|REQUIRE + PolicyEvaluated
- [x] InvariantChecker emits InvariantViolation

## Commands

```bash
cargo test -p aira-artifact -p aira-event -p aira-policy -p aira-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
bash Analyze-11/verification/deny-originals.sh
bash Analyze-11/verification/deny-foreign-workspace.sh
```

## Result

```text
cargo test -p aira-artifact → 4 passed
cargo test -p aira-event → 4 passed
cargo test -p aira-policy → 2 passed
cargo test -p aira-core → 6 passed
cargo clippy -D warnings → OK
deny-originals / deny-foreign → OK
Manifesto etc / Meditation_About → unchanged
```

**Verdict:** PASS
