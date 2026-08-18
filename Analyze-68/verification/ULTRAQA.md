# ULTRAQA — Analyze-68

**Verdict:** PASS  
**Date:** 2026-08-18

| Scenario | Result |
|----------|--------|
| A finds C via B over UDP | exact stored `udp:nodes:` |
| Announce still works on same listen | OK |
| Untrusted FIND requester | no NODES / timeout Discv |
| Untrusted hint skipped | C not stored |
| Address book empty | OK |

```bash
cargo test -p aira-peer discv:: --lib
cargo check -p aira-cli
```
