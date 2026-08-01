# UltraQA — Analyze-64

| ID | Scenario | Result |
|----|----------|--------|
| U1 | `cargo test -p aira-node` (46) | PASS |
| L1 | `clippy -p aira-node --all-targets -D warnings` | PASS |
| S1 | tenant register own publisher | PASS (unit/HTTP) |
| S2 | cross-publisher register → 403 | PASS |
| S3 | tenant list filtered / admin all | PASS |
| S4 | map without token boot fail | PASS |

**Verdict:** CLEAR
