# AIRA-RFC-0094 — Instance-scoped crypto

## 1. Summary

Phase I `#196`: embed/tests may bind a thread-local `Keyring` + primary via `bind_thread_crypto`. The process OnceLock remains the default when no bind is active. Sibling threads do not share signing identity.

## 5. Non-Goals

MSRV / supply-chain CI (`#197`); removing the process keyring from CLI/node; rewriting tenant map OnceLock; requiring every caller to pass a Keyring.

## 10. Contract

```text
bind_thread_crypto(ring, primary) → ThreadCryptoGuard
active_signature / verify_ed25519 / register_keyring → thread bind if set, else process OnceLock
drop(guard) → restore previous thread bind or process default
process keyring is not mutated by a thread bind
```

## 15. Tests

```text
cargo test -p aira-object --lib thread_crypto_scopes_do_not_leak
```
