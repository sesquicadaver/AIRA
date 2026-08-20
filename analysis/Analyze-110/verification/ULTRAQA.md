# UltraQA — Analyze-110

| ID | Check | Result |
|----|-------|--------|
| U1 | `cargo test -p aira-schema desktop_settings` | PASS |
| U2 | `schema validate --fixtures` (44 passed) | PASS |
| U3 | invalid missing `instance_id` fails | PASS |
| U4 | valid fixture `network_profile=P0` | PASS |
| U5 | clippy `-D warnings` aira-schema | PASS |

**Verdict:** PASS (schema-only; runtime ultraqa skipped)
