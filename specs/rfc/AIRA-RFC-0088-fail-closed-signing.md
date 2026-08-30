# AIRA-RFC-0088 — Fail-closed `active_signature`

## 1. Summary

Phase I `#190`: `active_signature` signs only the process primary identity. It does not fall back to `local_test_signature`. Demo/test use the default primary `aira:identity:local-test`. `LocalSession` propagates identity/trust/tenant load errors.

## 5. Non-Goals

Atomic session persist (`#191`); instance-scoped crypto (`#196`); removing local-test from the process keyring (fixtures still need it).

## 10. Contract

```text
active_signature(msg) → ring.sign(primary, msg)
primary without signing key → CryptoError::NoSigningKey (not local-test key_ref)
default / reset_primary_signer → local-test is explicit demo/test primary
LocalSession::open / submit_problem → register_node_identity / ensure_trust_defaults / load_all_csu_tenant_signing errors surface
```

## 15. Tests

```text
cargo test -p aira-object --lib active_signature_does_not_fallback_to_local_test
cargo test -p aira-flow --lib local_session_rejects_corrupt_identity
```
