# Verification — Analyze-39

## Commands
```bash
cargo test -p aira-csu --lib
cargo clippy -p aira-csu --lib -- -D warnings
cargo test -p aira-flow --lib
```

## Results (2026-07-28)
- aira-csu: 9 passed
- clippy aira-csu: clean
- aira-flow: 8 passed

## Coverage
- Default publisher on CSUFailed (local-test)
- Distinct publisher on CSURegistered + CSUFailed
- Missing signing key: register-with-sink fails; dispatch emit_failed fail-closed (no forged event)
