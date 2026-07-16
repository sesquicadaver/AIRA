# Verification Checklist — Analyze-16

## Immutability
- [x] Manifesto etc / Meditation_About unchanged

## Epic 9
- [x] Conformance Report schema + immutable artifact
- [x] C0 runner (5 cases)
- [x] C1 runner (4 cases)
- [x] Object / artifact immutability
- [x] Event causality
- [x] Policy gate
- [x] Failure-to-evidence
- [x] CLI `aira conformance run`

## Commands

```bash
cargo test -p aira-conformance
cargo clippy -p aira-conformance -p aira-cli --all-targets -- -D warnings
bash Analyze-16/verification/deny-originals.sh
bash Analyze-16/verification/deny-foreign-workspace.sh
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
```

## Result

```text
cargo test -p aira-conformance → 4 passed
CLI C0: total=5 passed=5
CLI C1: total=4 passed=4
clippy -D warnings → OK
deny-originals / deny-foreign → OK
```

**Verdict:** PASS
