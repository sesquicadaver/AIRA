# Cryptographic signatures (Alpha.2 + Identity Keyring + Plane signing + Trust Store)

## Local-test identity

Deterministic Ed25519 key for `aira:identity:local-test`:

- Seed (32 bytes, fixtures/tests only): `aira-mvp-local-test-ed25519-key!`
- Public key hex: `2754a265e1dd9eff273fb58b3162e474f7285d5a53d20ab0893e8523afbe7480`

API: `aira_object::{local_test_signature, verify_ed25519, Keyring, active_signature, primary_signer, TrustStore}`.

## Process keyring + primary signer

`verify_ed25519` resolves `signature.key_ref` via a process `Keyring` that always includes local-test.

**Primary signer** (Analyze-22): `aira_csu::support::{local_identity, local_signature, local_signature_over}` use `active_identity` / `active_signature`. When a node identity is registered, OperationalPlane + basic CSU emits carry that `key_ref`. The plane itself remains a C1 reference/demo ([operational-plane.md](operational-plane.md)), not a production event runtime.

**Per-CSU publisher** (Analyze-29 / Analyze-39 / Analyze-42 / Analyze-62): CSU business emits, `CSUFailed`, and CSU registry lifecycle events use `CsuManifest.publisher_identity` via `make_event_as` / `make_artifact_as` + `signature_for_tenant` (fail closed). Default `publisher_identity == identity_ref == primary` signs through the process keyring. A distinct publisher requires `register_csu_tenant_signing` / durable `save_csu_tenant_signing` — the **signing** secret stays in the tenant map (CSU-A cannot use CSU-B’s key); only the verifying pubkey merges into the process Keyring. Durable layout: `identity/tenants/<hex(csu_id)>/{ed25519,meta.json}` (mode `0600`). `LocalSession::open` / `submit_problem` call `load_all_csu_tenant_signing` **after** trust sync. Override helpers: `ContextBasicCsu::with_publisher` (and siblings). OperationalPlane ProblemStatement / plane lifecycle remain on primary.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant register \
  --csu-id aira:csu:example.worker --publisher aira:identity:worker-pub \
  --secret-hex-file ./worker.seed
# stdin (non-TTY): --secret-hex-file -
# demo-only (argv): --secret-hex <64-hex>
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant list
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant load
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant rotate \
  --csu-id aira:csu:example.worker --backup
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant backups
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant backups prune --keep 3
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant backups prune --older-than-days 31 --dry-run
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant revoke \
  --csu-id aira:csu:example.worker --reason compromised
```

**Tenant ceremony** (Analyze-63 / Analyze-72): `rotate` keeps the same `publisher_id`, writes a new `ed25519` (optional `ed25519.prev` + archive prior latest), audits `tenant_rotate`. `revoke` unloads the map, drops the publisher verifying key when unshared (never primary / local-test), deletes `identity/tenants/<hex>/`, audits `tenant_revoke`. Signing-side only — TrustStore CRL is not updated; historical verify may still succeed if the pubkey remains trusted. `register` refuses overwrite unless `--force` (prefer `rotate`). One `publisher_id` may bind to at most one CSU.

Import a known seed with `--secret-hex-file PATH` (file body: exactly 64 hex digits after trimming ends; `-` = stdin, refused on a TTY; a file named `-` is `./-`). `--secret-hex` remains demo-only (visible in argv). The two flags are mutually exclusive. Node `identity create` / `identity rotate` do not take a secret file.

**Tenant backup retention** (Analyze-71): GC archived `ed25519.prev.<stamp>` only (never latest `.prev` or live `ed25519`). Per-tenant `--keep` / `--older-than-days` (same intersection rule as node `identity backups prune`). `identity backups prune` does **not** touch tenant dirs.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant backups
# csu_id  stamp  old_pubkey|-  backed_up_at|-  path
cargo run -p aira-cli -- --root "$ROOT" identity csu-tenant backups prune --keep 3
```

Recommended starting points: `--keep 3` or `--older-than-days 31`. At least one flag is required. Stamps are unix seconds (numeric rank, not lex). Unparseable ages under `--older-than-days` are skipped; orphan `.meta.json` without a secret is never deleted.

On `LocalSession::open` / `submit_problem` / `aira identity create`:

1. Load `.aira/identity/` into the keyring
2. Set primary signer to the node `identity_id`
3. Ensure `.aira/identity/trust.json` defaults (node pub when present; **never** `local-test` — SEC-1) and register verifying keys

**SEC-1 migration (QUEUE #134):** Older nodes may have `aira:identity:local-test` in `trust.json`. `ensure_trust_defaults` / session open strips that entry on save. `identity trust add` refuses local-test; peer handshake and discv reject it even if a legacy file still lists it. Process keyring keeps local-test for fixtures/signing only — not runtime peer trust.

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
# local-test must not appear in trust.json (SEC-1); legacy entries are stripped on session open
```

`register_trust_store` merges entries into the process keyring so `verify_ed25519` / `aira identity verify` succeed for trusted peers without their signing keys on disk.

**Unload / sync** (Analyze-24 / Analyze-62 / Analyze-63): `sync_trust_verifiers` prunes process verifying keys absent from `trust.json` (never unloads `local-test` from the **process keyring** for fixture signing; signing identities keep derived verifying keys unless revoked; **in-memory CSU tenant publishers are preserved** until tenant unload / `csu-tenant revoke` — trust CRL alone does not drop them). `identity trust remove` and `ensure_trust_defaults` call sync so unload takes effect in-process immediately.

**SEC-1:** `aira:identity:local-test` must **not** be in `trust.json` entries. Upsert, peer handshake, and discv admission reject it; `ensure_trust_defaults` migrates legacy installs.

**SEC-3:** Object/Event/Artifact descriptor verify requires `signature.key_ref == producer_identity` before canonical Ed25519 check; stores reject cross-identity descriptors.

**SEC-4:** Event log and Event Protocol adapter reject equivocation: duplicate `event_id` with a different canonical descriptor hash returns `EQUIVOCATION` / `EventError::Equivocation`; same hash remains idempotent (`ACCEPTED`, no second append).

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

**Ceremony audit** (Analyze-40 / Analyze-63): append-only `.aira/identity/trust-audit.jsonl` records `revoke` / `unrevoke` / `rotate` / `rekey` / `node_rotate` / `tenant_rotate` / `tenant_revoke` (public metadata only — never secrets). Written by CLI trust commands, `apply_trust_delta`, `rotate_node_signing_secret`, and `csu-tenant rotate|revoke`.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity trust revoke \
  --key-ref aira:identity:peer-alice --reason compromised
cargo run -p aira-cli -- --root "$ROOT" identity trust audit
# recorded_at  action  subject  new_id  reason  source
cargo run -p aira-cli -- --root "$ROOT" identity trust audit --last 5
```

**Rotate** (Analyze-27/28): atomic peer replacement — revoke `old` with `superseded_by`, trust `new` with `supersedes`. Without `--until`, old signatures fail immediately after sync. With `--until <RFC3339 UTC>`, dual-key grace keeps old pubkey verifiable until that instant (`RevokedEntry.grace_until`); upsert of old remains blocked.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity trust rotate \
  --old-key-ref aira:identity:peer-alice \
  --new-key-ref aira:identity:peer-alice-v2 \
  --pubkey-hex <64-hex> --reason "rollover" \
  --until 2026-07-17T00:00:00Z
```

## Node signing-secret rotate (Analyze-30/31/37)

Rewrites `.aira/identity/local.ed25519` and updates `local.identity.json` **without** changing `identity_id`. Trust store upserts the new pubkey for the same id (no CRL).

**Immediate cutover** (default): signatures made with the previous secret fail under the same `key_ref`.

**Dual-key grace** (Analyze-37): `identity rotate --until <RFC3339 UTC>` keeps the previous verifying key for the same `key_ref` until that instant (`previous_public_key` + `previous_grace_until` on the identity descriptor). `Keyring` may hold multiple verifying keys per ref; signing remains the current secret only.

**Peer notify** (Analyze-38): `identity rotate --notify-peers` announces the **upcoming** pubkey to the address book **before** cutover (`trust-delta` op `rekey`), so hello still verifies; receivers with `--apply-trust` upsert the issuer pubkey.

**Remote same-id dual-key** (Analyze-50): `rekey` with `--until` / `grace_until` stores `previous_public_key_hex` on the peer `TrustEntry` so both old and new Ed25519 keys verify under the same `identity_id` until cutoff. Prefer `--notify-peers --until <RFC3339>` so receivers keep the old key through the notify→rotate window.

**Noise static rotate** (Analyze-49): the same `identity rotate` always regenerates `identity/local.x25519`. With `--backup`, the prior secret is written to `identity/local.x25519.prev` (prior `.prev` archived as `local.x25519.prev.<stamp>`).

Peers learn the new Noise static from the next hello (`x25519_pub_hex` Ed25519-signed). Dedicated peer-notify for x25519 was evaluated in Analyze-54 and closed as **wont-need** (QUEUE #19).

```bash
cargo run -p aira-cli -- --root "$ROOT" identity rotate
# rotated aira:identity:local
# old_public_key …
# public_key …

# Opt-in durable previous secret (Analyze-31):
cargo run -p aira-cli -- --root "$ROOT" identity rotate --backup
# … backup …/identity/local.ed25519.prev

# Dual-key grace (Analyze-37):
cargo run -p aira-cli -- --root "$ROOT" identity rotate --until 2099-01-01T00:00:00Z
# … grace_until …

# Notify address-book peers before cutover (Analyze-38 + A-50 grace):
cargo run -p aira-cli -- --root "$ROOT" identity rotate --notify-peers --until 2099-01-01T00:00:00Z
```

Default rotate still leaves no durable old secret. With `--backup`, the previous secret is staged under `*.tmp` (mode `0600`) before overwrite and renamed to `identity/local.ed25519.prev` (+ `local.ed25519.prev.meta.json`) only after a successful rotate. Staging failure or mid-rotate abort removes tmp only (existing `.prev` / history slots are preserved).

**Timestamped history** (Analyze-41): each new `--backup` archives the prior latest slot to `local.ed25519.prev.<YYYYMMDDTHHMMSSZ>` (+ matching `.meta.json`) so earlier secrets remain recoverable. Canonical `.prev` is always the most recent backup.

**Retention / prune** (Analyze-61): GC archived stamp slots only (never the canonical latest `.prev`). Same CLI also prunes `local.x25519.prev.<stamp>`.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity backups
# ed25519|x25519  stamp  old_pubkey|-  backed_up_at|-  path
cargo run -p aira-cli -- --root "$ROOT" identity backups prune --keep 3
cargo run -p aira-cli -- --root "$ROOT" identity backups prune --older-than-days 31
cargo run -p aira-cli -- --root "$ROOT" identity backups prune --keep 3 --older-than-days 31 --dry-run
```

Recommended starting points: `--keep 3` or `--older-than-days 31`. At least one flag is required. Retain = intersection of supplied policies per key family. Unparseable ages under `--older-than-days` are skipped (not deleted); orphan `.meta.json` without a secret is never deleted.

```bash
cargo run -p aira-cli -- --root "$ROOT" identity rotate --backup
cargo run -p aira-cli -- --root "$ROOT" identity rotate --backup
cargo run -p aira-cli -- --root "$ROOT" identity backups
# latest / <stamp>  old_public_key_hex  backed_up_at  path
```

## Canonical signed messages

| Object | Message bytes |
|--------|----------------|
| Canonical descriptor helper (Analyze-74 / #39) | `descriptor_signing_hash(strip signature).as_str()` — Event #40, Artifact #41, Object #42, CSU #43 |
| Artifact | canonical descriptor JSON without `signature` (Analyze-76 / #41); payload still must match `content_hash` |
| Event | canonical descriptor JSON without `signature` (Analyze-75 / #40); no event-path `LOCAL_TEST_DOMAIN_MSG` fallback |
| CSU manifest | canonical manifest JSON without `signature` (Analyze-78 / #43); signer = `identity_ref` |
| Problem object | canonical descriptor JSON without `signature` (Analyze-77 / #42) |
| Protocol envelope | `payload_hash.as_str()` only (Analyze-79 / #44); no `LOCAL_TEST_DOMAIN_MSG` verify fallback |
| Identity descriptor (create / rotate) | `identity_id` bytes (Analyze-79 / #44); no domain-message verify |

Empty and `TESTSIG` are rejected on admission.

## Out of scope (later)

Канон: [`QUEUE.md`](../QUEUE.md). Phase C: [`docs/phase-c-plan.md`](phase-c-plan.md).

| Було Out | Рядок |
|----------|-------|
| CN→TrustStore mapping | #20 **DONE** (Analyze-55) |
| Separate health listener without client cert | #21 **DONE** (Analyze-56 `--health-listen`) |
| Optional (anonymous) client TLS auth | deferred (A-51 TODO; не в QUEUE) |
| On-disk per-CSU secret files | #27 **DONE** (Analyze-62) |
| Tenant ceremony | #28 **DONE** (Analyze-63) |
| Multi-tenant HTTP authz | #29 **DONE** (Analyze-64) |
| YAML config parity | #30 **DONE** (Analyze-65) |
| STUN / ICE-lite dial path | #31 **DONE** (Analyze-66 Binding reflexive) |
| Retention/prune `.prev.<stamp>` | #26 **DONE** (Analyze-61) |
| Tenant `.prev.<stamp>` prune | #36 **DONE** (Analyze-71) |
| Tenant `--secret-hex-file` / stdin | #37 **DONE** (Analyze-72) |
| SQLite ceremony audit table | після #26 (не окремий рядок поки JSONL достатньо; додати в кінець за потреби) |
| UDP discv5-style announce | #32 **DONE** (Analyze-67) |
| Iterative FIND_NODE | #33 **DONE** (Analyze-68) |
| Public HTTP bind opt-in | #34 **DONE** (Analyze-69 `--allow-public-bind`) |
| Federation join prototype | #35 **DONE** (Analyze-70 local pin) |

Shipped: local HTTP TLS (A-45); HTTP Bearer (A-48); DHT-lite (A-47); coordinated `local.x25519` rotate (A-49); remote same-id TrustStore dual-key / `TrustStore::rekey` (A-50); mTLS require client cert via `--tls-client-ca` (A-51); CN→TrustStore (A-55); self-sovereign trust-delta (A-52); plain `--health-listen` when mTLS (A-56); DHT→address_book `--apply-book` (A-57); durable relay registry (A-58); concurrent peer accept (`accept_tcp` + spawned handshake, A-59); systemd examples (A-60); `.prev.<stamp>` prune CLI (A-61); durable per-CSU secrets (A-62); tenant rotate/revoke ceremony (A-63); multi-tenant HTTP CSU authz Bearer map (A-64); YAML∨JSON config read parity (A-65); STUN Binding reflexive + `dht announce --from-stun` (A-66); UDP discv announce → local DHT store (A-67); iterative UDP FIND_NODE (A-68); public HTTP bind opt-in `--allow-public-bind` (A-69); local federation join pin (A-70); tenant `ed25519.prev.<stamp>` prune (A-71); tenant `--secret-hex-file` (A-72).

See also: [peer-link.md](peer-link.md) (hello v1 + Noise XX + trust-delta + rekey notify + relay/DHT).
