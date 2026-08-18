# ULTRAQA — Analyze-67

**Verdict:** PASS  
**Date:** 2026-08-18

## Hostile checks
| Scenario | Result |
|----------|--------|
| Trusted UDP roundtrip → `dht.json` source=udp | OK |
| Address book unchanged | OK |
| Untrusted issuer | Untrusted, no store |
| Revoked issuer | Revoked, no store |
| identity_id ≠ key_ref | IdentityMismatch |
| Non-loopback bind without `--explicit` | Err |
| TCP dial path | Untouched |

```bash
cargo test -p aira-peer discv:: --lib
cargo check -p aira-cli
```
