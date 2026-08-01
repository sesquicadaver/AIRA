# ULTRAQA — Analyze-58

**Verdict:** PASS (after rework)

## Baseline
| Check | Result |
|-------|--------|
| `cargo test -p aira-peer` | 43 tests (incl. TTL none retain) |
| clippy aira-peer + aira-cli | ok |

## Scenarios
| ID | Result |
|----|--------|
| S1 `--relay-ttl-days` without `--relay` | fail-closed PASS |
| S3 corrupt schema before bind | no `listening`, exit≠0 PASS |
| Unit restart/offline/TTL/online protected | PASS |

## Rework
Registry Mutex; durable-before-live; CLI validate before bind.
