# Cryptographic signatures (Alpha.2 + Identity Keyring + Plane signing)

## Local-test identity

Deterministic Ed25519 key for `aira:identity:local-test`:

- Seed (32 bytes, fixtures/tests only): `aira-mvp-local-test-ed25519-key!`
- Public key hex: `2754a265e1dd9eff273fb58b3162e474f7285d5a53d20ab0893e8523afbe7480`

API: `aira_object::{local_test_signature, verify_ed25519, Keyring, active_signature, primary_signer}`.

## Process keyring + primary signer

`verify_ed25519` resolves `signature.key_ref` via a process `Keyring` that always includes local-test.

**Primary signer** (Analyze-22): `aira_csu::support::{local_identity, local_signature, local_signature_over}` use `active_identity` / `active_signature`. When a node identity is registered, OperationalPlane + basic CSU emits carry that `key_ref`.

On `LocalSession::open` / `submit_problem` / `aira identity create`:

1. Load `.aira/identity/` into the keyring
2. Set primary signer to the node `identity_id`

```bash
cargo run -p aira-cli -- --root "$ROOT" identity create --name local
cargo run -p aira-cli -- --root "$ROOT" problem submit --text "Calculate 2 + 2"
# events/artifacts producer_identity + signature.key_ref == aira:identity:local
```

## Canonical signed messages

| Object | Message bytes |
|--------|----------------|
| Artifact | `content_hash.as_str()` |
| Event | `payload_hash.as_str()` **or** domain message |
| CSU manifest | `csu_id.as_str()` |
| Problem object | `content_hash.as_str()` |
| Protocol envelope | `payload_hash.as_str()` **or** domain message |
| Identity descriptor (create) | `identity_id` bytes |

Empty and `TESTSIG` are rejected on admission.

## Out of scope (later)

Multi-key trust store / rotation; TLS; per-CSU publisher identity overrides.
