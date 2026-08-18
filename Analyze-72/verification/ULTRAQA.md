# ULTRAQA — Analyze-72

**Verdict:** PASS  
**Date:** 2026-08-18

## Goal
register/rotate import 64-hex seed from file or stdin without argv; fail-closed on XOR/bad hex/oversize; never print seed.

## Scenario matrix
| ID | Scenario | Expected | Actual | Status |
|----|----------|----------|--------|--------|
| T1 | `cargo test -p aira-cli` | 9 pass | 9 pass | PASS |
| U1 | `--help` shows `--secret-hex-file` | prefer file | ok | PASS |
| U2 | both flags | clap exit ≠ 0 | exit 2 | PASS |
| U3 | register from file | pubkey matches seed | ok | PASS |
| U4 | rotate from stdin different seed | pubkey changes | ok | PASS |
| U5 | `0x` prefix file | exit 1, seed absent | ok | PASS |
| U6 | `--secret-hex-file -` on `/dev/tty` | TTY fail | no TTY in harness | SKIP (unit `is_tty=true`) |
| U7 | 4097-byte file | 4KiB err, seed absent | ok | PASS |

Cleanup: temp roots removed.
