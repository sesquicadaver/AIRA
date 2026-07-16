# Cryptographic signatures (Alpha.2 + Identity Keyring)

## Local-test identity

Deterministic Ed25519 key for `aira:identity:local-test`:

- Seed (32 bytes, fixtures/tests only): `aira-mvp-local-test-ed25519-key!`
- Public key hex: `2754a265e1dd9eff273fb58b3162e474f7285d5a53d20ab0893e8523afbe7480`

API: `aira_object::{local_test_signature, verify_ed25519, Keyring, LOCAL_TEST_DOMAIN_MSG}`.

## Process keyring (Analyze-21)

`verify_ed25519` resolves `signature.key_ref` via a process `Keyring` that always includes local-test.

On `LocalSession::open` / `aira identity create|sign|verify`, keys from `.aira/identity/` are registered:

- `identity/local.identity.json` — `identity_id` + `public_key.key_hex`
- `identity/local.ed25519` — 32-byte secret as hex (mode 0600 best-effort)

```bash
cargo run -p aira-cli -- --root "$ROOT" identity create --name local
cargo run -p aira-cli -- --root "$ROOT" identity sign --text "hello"
cargo run -p aira-cli -- --root "$ROOT" identity verify --text "hello" --signature <hex>
```

## Canonical signed messages

| Object | Message bytes |
|--------|----------------|
| Artifact | `content_hash.as_str()` |
| Event | `payload_hash.as_str()` **or** `LOCAL_TEST_DOMAIN_MSG` |
| CSU manifest | `csu_id.as_str()` |
| Protocol envelope | `payload_hash.as_str()` **or** domain message |
| Identity descriptor (create) | `identity_id` bytes |

`signature_value` is lowercase hex of the 64-byte Ed25519 signature. Empty and `TESTSIG` are rejected on admission.

## Out of scope (later)

Multi-key trust store / rotation; plane-wide signing of all CSU emits with node key; TLS.
