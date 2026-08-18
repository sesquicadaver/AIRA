# DI crystallize — Analyze-69 / QUEUE #34

## In scope (A + A1 + B1 + F1)
1. `--allow-public-bind` required for non-loopback `--listen` **and** `--health-listen`.
2. Without flag → fail-closed (no bind, no warning-and-continue).
3. Loopback unchanged (default `127.0.0.1:8787`).
4. TLS/Bearer remain independent opt-in; public + plain HTTP → **warning**, not fail.
5. Tests: refuse `0.0.0.0` without flag; allow with flag; `--health-listen` same policy.
6. Docs: `docs/local-node.md` + systemd note that examples stay loopback.

## Out
Federation; peer TCP/UDP listen; requiring TLS/Bearer for public bind; public-by-default; port changes.

## Decision boundaries (agent-owned)
Flag name `--allow-public-bind` (no env default); helper `assert_bind_allowed`; IPv6 loopback (`::1`) counts as loopback.
