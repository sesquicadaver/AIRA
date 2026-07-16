# Cryptographic signatures (Alpha.2)

## Local-test identity

Deterministic Ed25519 key for `aira:identity:local-test`:

- Seed (32 bytes, fixtures/tests only): `aira-mvp-local-test-ed25519-key!`
- Public key hex: `2754a265e1dd9eff273fb58b3162e474f7285d5a53d20ab0893e8523afbe7480`

API: `aira_object::{local_test_signature, verify_ed25519, LOCAL_TEST_DOMAIN_MSG}`.

## Canonical signed messages

| Object | Message bytes |
|--------|----------------|
| Artifact | `content_hash.as_str()` |
| Event | `payload_hash.as_str()` **or** `LOCAL_TEST_DOMAIN_MSG` (emitters that reuse a domain signer) |
| CSU manifest | `csu_id.as_str()` |
| Protocol envelope | `payload_hash.as_str()` **or** domain message |

`signature_value` is lowercase hex of the 64-byte Ed25519 signature. Empty and `TESTSIG` are rejected on admission.

## Out of scope (later)

Verify against keys from `aira identity create`; multi-key trust store; TLS.
