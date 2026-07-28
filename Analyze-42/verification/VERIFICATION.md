# Verification — Analyze-42

## Commands
```bash
cargo test -p aira-object -p aira-csu --lib
cargo test -p aira-csu-context-basic -p aira-csu-execution-basic -p aira-flow --lib
cargo clippy -p aira-object -p aira-csu -p aira-flow -p aira-cli -- -D warnings
```

## Results (2026-07-28)
- aira-object: 31 passed (tenant isolation + unregistered fail-closed)
- aira-csu: 9 passed
- context/execution/flow: green
- clippy: clean (after `too_many_arguments` allow on `make_event_as`)
