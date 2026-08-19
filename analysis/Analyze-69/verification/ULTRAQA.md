# ULTRAQA — Analyze-69

**Verdict:** PASS  
**Date:** 2026-08-18

## Goal and success criteria
- Non-loopback HTTP bind refuses without `--allow-public-bind` (exit ≠ 0, no listen)
- Loopback `/health` still works
- Env cannot bypass the flag
- Stop: matrix green or bounded kill of any accidental bind

## Scenario matrix
| ID | User/attacker model | Scenario | Command/harness | Expected signal | Actual result | Status | Evidence | Cleanup |
|----|---------------------|----------|-----------------|-----------------|---------------|--------|----------|---------|
| S1 | careless operator | `--listen 0.0.0.0` no flag | `aira-node --http --listen 0.0.0.0:19991` | exit 1, fail-closed | exit 1 | PASS | stderr `--allow-public-bind` | n/a |
| S2 | LAN bind | `192.168.0.1` no flag | same | exit 1 | exit 1 | PASS | not loopback | n/a |
| S3 | IPv6 unspecified | `[::]` no flag | same | exit 1 | exit 1 | PASS | `[::]:19991` | n/a |
| S4 | local operator | loopback `/health` | timeout + curl | HTTP 200 | 200 | PASS | `127.0.0.1:18987` | timeout killed |
| S5 | env bypass | `AIRA_ALLOW_PUBLIC_BIND=1` | env + `0.0.0.0` | still refuse | refuse | PASS | no env on flag | n/a |
| S6 | malformed | `--listen not-an-addr` | parse fail | exit 1 | exit 1 | PASS | invalid socket | n/a |
| S7 | health without mTLS | `--health-listen 0.0.0.0` | requires mTLS first | exit 1 | exit 1 | PASS | `--tls-client-ca` | n/a |
| S8 | flag without HTTP | `--allow-public-bind` only | require `--http` | exit 1 | exit 1 | PASS | require --http | n/a |
| S9 | injection-like addr | `0.0.0.0:8787;rm -rf /tmp` | parse fail | exit 1 | exit 1 | PASS | invalid syntax | n/a |
| S10 | mapped loopback | `[::ffff:127.0.0.1]` | fail-closed (not std loopback) | exit 1 | exit 1 | PASS | not loopback | n/a |
| S11 | opt-in public | `--allow-public-bind --listen 0.0.0.0` | warning + bind | warning + listen | PASS | killed after evidence | port 19991 cleared |
| S12 | help | `--help` | flag + examples | present | PASS | `--allow-public-bind` | n/a |
| S13 | flag + bad addr | `--allow-public-bind --listen :::` | parse fail | exit 1 | PASS | invalid syntax | n/a |

```bash
cargo test -p aira-node
# probes against target/debug/aira-node (temp --root)
```
