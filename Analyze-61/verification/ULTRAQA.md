# UltraQA — Analyze-61

| ID | Scenario | Result |
|----|----------|--------|
| S1 | list family columns after 3× rotate --backup | PASS |
| S2 | prune --keep 1 --dry-run | PASS (would_delete; files remain) |
| S3 | prune --keep 1 | PASS (older archive gone; latest kept) |
| S4 | prune without flags | PASS (nonzero exit) |
| U1 | unit prune_* ed25519+x25519 | PASS |
| L1 | clippy -D warnings object/peer/cli | PASS |

**Verdict:** CLEAR
