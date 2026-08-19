# UltraQA — Analyze-59

| ID | Scenario | Result | Evidence |
|----|----------|--------|----------|
| S1 | Hung raw TCP then `peer dial`/`send` | PASS | dial_ms=148 (<3s); listener received ping; hung handshake eventually fail-closed |
| S2 | Corrupt inbound bytes then dial/send | PASS | `accept handshake error: frame too large`; subsequent `received peer.ping` |
| S3 | Two parallel `peer send` (alice+carol) | PASS | ≥2 `received` lines with both issuers |
| U1 | Unit: hung/broken/parallel/relay | PASS | `cargo test -p aira-peer --lib` 47 ok |
| L1 | clippy -D warnings peer+cli | PASS | clean |

**Verdict:** CLEAR
