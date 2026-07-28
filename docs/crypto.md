# Cryptographic signatures (Alpha.2 + Identity Keyring + Plane signing + Trust Store)

## Local-test identity

Deterministic Ed25519 key for `aira:identity:local-test`:

- Seed (32 bytes, fixtures/tests only): `aira-mvp-local-test-ed25519-key!`
- Public key hex: `2754a265e1dd9eff273fb58b3162e474f7285d5a53d20ab0893e8523afbe7480`

API: `aira_object::{local_test_signature, verify_ed25519, Keyring, active_signature, primary_signer, TrustStore}`.

## Process keyring + primary signer

`verify_ed25519` resolves `signature.key_ref` via a process `Keyring` that always includes local-test.

**Primary signer** (Analyze-22): `aira_csu::support::{local_identity, local_signature, local_signature_over}` use `active_identity` / `active_signature`. When a node identity is registered, OperationalPlane + basic CSU emits carry that `key_ref`.

**Per-CSU publisher** (Analyze-29): CSU emits use `CsuManifest.publisher_identity` via `make_event_as` / `make_artifact_as` + `signature_for` (fail closed if no signing key). Default `publisher_identity == identity_ref == primary`. Override with `ContextBasicCsu::with_publisher` (and siblings). Plane ProblemStatement / lifecycle remain on primary.

On `LocalSession::open` / `submit_problem` / `aira identity create`:

1. Load `.aira/identity/` into the keyring
2. Set primary signer to the node `identity_id`
3. Ensure `.aira/identity/trust.json` defaults (local-test + node pub) and register verifying keys

```bash
cargo run -p aira-cli -- --root "$ROOT" identity create --name local
cargo run -p aira-cli -- --root "$ROOT" problem submit --text "Calculate 2 + 2"
# events/artifacts producer_identity + signature.key_ref == aira:identity:local
```

## Trust store (Analyze-23)

Path: `.aira/identity/trust.json` — verifying public keys only (never peer secrets).

```bash
cargo run -p aira-cli -- --root "$ROOT" identity trust list
cargo run -p aira-cli -- --root "$ROOT" identity trust add \
  --key-ref aira:identity:peer-alice --pubkey-hex <64-hex>
cargo run -p aira-cli -- --root "$ROOT" identity trust remove \
  --key-ref aira:identity:peer-alice
# refuse remove of aira:identity:local-test
```

`register_trust_store` merges entries into the process keyring so `verify_ed25519` / `aira identity verify` succeed for trusted peers without their signing keys on disk.

**Unload / sync** (Analyze-24): `sync_trust_verifiers` prunes process verifying keys absent from `trust.json` (never unloads `local-test`; signing identities keep derived verifying keys unless revoked). `identity trust remove` and `ensure_trust_defaults` call sync so unload takes effect in-process immediately.

**CRL** (Analyze-25): `trust.json` field `revoked[]` is a durable deny list. `identity trust revoke --key-ref … [--reason …]` moves an id out of `entries` onto the CRL; `trust add` / `upsert` of a revoked id fails with `RevokedKey`. `remove` is still non-durable (re-add allowed). `local-test` cannot be revoked.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity trust revoke \
  --key-ref aira:identity:peer-alice --reason compromised
cargo run -p aira-cli -- --root "$ROOT" identity trust list
# shows REVOKED lines; re-add of peer-alice fails
```

**Unrevoke** (Analyze-26): `identity trust unrevoke --key-ref …` clears the CRL entry only. It does **not** restore `entries` or process verifying keys (no silent re-trust from stored CRL pubkey). Operator must run `trust add` again.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity trust unrevoke \
  --key-ref aira:identity:peer-alice
cargo run -p aira-cli -- --root "$ROOT" identity trust add \
  --key-ref aira:identity:peer-alice --pubkey-hex <64-hex>
```

| Action | Durable deny? | Re-add without unrevoke? | Auto-trust after? |
|--------|---------------|--------------------------|-------------------|
| `remove` | no | yes | n/a |
| `revoke` | yes (CRL) | no | n/a |
| `unrevoke` | clears CRL | then yes via `add` | **no** — need `add` |
| `rotate` | yes (old→CRL) | old needs `unrevoke` | **yes** for new; old only during `--until` grace |

**Rotate** (Analyze-27/28): atomic peer replacement — revoke `old` with `superseded_by`, trust `new` with `supersedes`. Without `--until`, old signatures fail immediately after sync. With `--until <RFC3339 UTC>`, dual-key grace keeps old pubkey verifiable until that instant (`RevokedEntry.grace_until`); upsert of old remains blocked.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity trust rotate \
  --old-key-ref aira:identity:peer-alice \
  --new-key-ref aira:identity:peer-alice-v2 \
  --pubkey-hex <64-hex> --reason "rollover" \
  --until 2026-07-17T00:00:00Z
```

## Node signing-secret rotate (Analyze-30)

Rewrites `.aira/identity/local.ed25519` and updates `local.identity.json` **without** changing `identity_id`. Trust store upserts the new pubkey for the same id (no CRL). Immediate cutover: signatures made with the previous secret fail under the same `key_ref`.

This is **not** peer `identity trust rotate` (which replaces one trusted id with another).

```bash
cargo run -p aira-cli -- --root "$ROOT" identity rotate
# rotated aira:identity:local
# old_public_key …
# public_key …

# Opt-in durable previous secret (Analyze-31):
cargo run -p aira-cli -- --root "$ROOT" identity rotate --backup
# … backup …/identity/local.ed25519.prev
```

Default rotate still leaves no durable old secret. With `--backup`, the previous secret is staged under `*.tmp` (mode `0600`) before overwrite and renamed to `identity/local.ed25519.prev` (+ `local.ed25519.prev.meta.json`) only after a successful rotate. Staging failure or mid-rotate abort removes tmp only (existing `.prev` slot is preserved). A single `.prev` slot is overwritten on each successful `--backup` rotate.

## Canonical signed messages

| Object | Message bytes |
|--------|----------------|
| Artifact | `content_hash.as_str()` |
| Event | `payload_hash.as_str()` **or** domain message |
| CSU manifest | `csu_id.as_str()` |
| Problem object | `content_hash.as_str()` |
| Protocol envelope | `payload_hash.as_str()` **or** domain message |
| Identity descriptor (create / rotate) | `identity_id` bytes |

Empty and `TESTSIG` are rejected on admission.

## Out of scope (later)

Dual-key grace for the same node `key_ref`; TLS; multi-tenant per-CSU keyring; CRL audit log; auto peer notify of rotated pubkey; coordinated rotate of `local.x25519` with Ed25519.

See also: [peer-link.md](peer-link.md) (hello v1 + Noise XX).
