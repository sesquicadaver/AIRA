# ULTRAQA — Analyze-66

**Verdict:** PASS  
**Date:** 2026-08-17

## Hostile checks
| Scenario | Result |
|----------|--------|
| Mock Binding → XOR-MAPPED | OK |
| Persist `stun_reflexive.json` | OK |
| `--from-stun` + `--addr` | Fail-closed |
| Missing stun file | Err |
| No default public STUN | CLI requires server |
| dial TCP/book | Unchanged (regression suite) |

```bash
cargo test -p aira-peer stun:: --lib
cargo check -p aira-cli
```
