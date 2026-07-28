# VERIFICATION — Analyze-46

## Commands

```bash
cargo test -p aira-conformance --lib
cargo test -p aira-node http_conformance
cargo clippy -p aira-conformance -p aira-cli -p aira-node -- -D warnings
```

## Results (2026-07-28)

- `aira-conformance`: **7 passed** (incl. `c2_suite_passes_and_emits_report`)
- `http_conformance_c2`: **ok**
- clippy `-D warnings`: **ok**

## Acceptance map

| Item | Evidence |
|------|----------|
| 5 M13 exit cases | `run_c2` |
| CLI/HTTP C2 | wired + HTTP test |
| docs/QUEUE | conformance.md, QUEUE #12 DONE |
