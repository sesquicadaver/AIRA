# ULTRAQA — Analyze-70

**Verdict:** PASS  
**Date:** 2026-08-18

## Goal
Local federation join pins trust+membership; fail-closed without a valid self-signed descriptor; no accidental membership file.

## Scenario matrix
| ID | Scenario | Expected | Actual | Status |
|----|----------|----------|--------|--------|
| T1 | `cargo test -p aira-protocol federation` | 10 pass | 10 pass | PASS |
| T2 | `with_verifying_hex` detached | no local-test in ring | ok | PASS |
| U1 | `aira federation join --help` | flag `--descriptor` | ok | PASS |
| U2 | join without init | exit 1 | exit 1 | PASS |
| U3 | missing descriptor file | exit 1 | exit 1 | PASS |
| U4 | unsigned JSON | exit 1, no membership | exit 1, no file | PASS |
| U5 | malformed JSON | exit 1 | exit 1 | PASS |

```bash
cargo test -p aira-object with_verifying_hex
cargo test -p aira-protocol federation
cargo run -p aira-cli -- federation join --help
```
