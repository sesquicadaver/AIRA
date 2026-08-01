# ULTRAQA — Analyze-57

**Goal:** `--apply-book` promotes DHT exact/announce into address book; fail-closed without flag/`--dht`.  
**Verdict:** **PASS**

## Baseline
| Check | Result |
|-------|--------|
| `cargo test -p aira-peer` | 39 passed |
| clippy aira-peer + aira-cli `-D warnings` | ok |

## Scenario matrix
| ID | Intent | Result |
|----|--------|--------|
| S1 | listen `--apply-book` without `--dht` | FAIL closed PASS |
| S2 | find exact `--apply-book` preserves via | PASS |
| S3 | closest-only / no exact → skipped, book empty | PASS |
| I1 | announce+promote+dial (+ via keep) | unit PASS |

## Stop
PASS cycle 1.
