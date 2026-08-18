# Ralplan — Analyze-70 / QUEUE #35

## Principles
1. Join is a local operator ceremony, not a new network protocol.
2. Trust bootstrap is self-signed + explicit CLI invoke (A1), then CRL still wins.
3. One membership until a later leave row (C1).
4. Do not touch peer hello / Noise / trust-delta / `TrustStore::upsert`.

## Decision drivers
- QUEUE done-when: мінімальний join+trust + Living Spec.
- Book II §14 descriptor only (not 14.3/14.4).
- Existing TrustStore upsert/CRL; federation types stay out of Core.

## Viable options
- **A (chosen):** `aira-protocol` federation module + CLI join; `aira-object` only `Keyring::with_verifying_hex`. Pros: testable without TCP; reuses TrustStore; respects Core boundary. Cons: not a multi-node handshake.
- **B (rejected):** Join Request/Response files — Out (B1).
- **C (rejected):** peer message — Out + anti-merge authn+federation.

## Architect (WATCH folded)
- Placement: `aira-protocol/src/federation.rs`, not federation types in `aira-object`.
- Do not change `TrustStore::upsert`; different-key fail-closed lives in join wrapper.
- Detached verify against embedded pubkey (`Keyring::with_verifying_hex` empty ring, one key, not `with_local_test`).
- Honest Living Spec: local pin / TOFU; other federation members stay Untrusted until separately trusted.
- Order: verify → CRL + singleton/key checks → `save`+`register_trust_store` → membership.

## Critic (ITERATE folded)
Canonical bytes (signature excluded):
`aira:federation:descriptor:v1|{schema}|{federation_id}|{federation_type}|{identity_ref}|{public_key_hex}`
`schema` == domain string; `identity_ref == signature.key_ref` (AiraRef); `federation_id` is Book II string (`aira:federation:…`).

`membership.json`: `schema` (`aira:federation:membership:v1`), `federation_id`, `federation_type`, `identity_ref`, `public_key_hex`, `joined_at` (RFC3339). Create dir on join; `init_node` unchanged.

Persist: after checks, `TrustStore::save` + `register_trust_store` (like `trust add`), then membership. No new `TrustAuditAction`.

Wrapper fail-closed (upsert unchanged): other `federation_id`; same `federation_id` + different key; existing TrustStore entry for `identity_ref` with different pubkey; `identity_ref == LOCAL_TEST_KEY_REF`. Same id+key success is idempotent (`joined_at` frozen). Crash after save before membership: retry same id+key completes membership.

## Implementation
1. `Keyring::with_verifying_hex` in `aira-object`.
2. `join_federation` in `aira-protocol`.
3. CLI `aira federation join --descriptor`.
4. Tests + docs + Living Spec.

## Tests
- happy join → trust entry + membership
- bad signature / unsigned / `key_ref != identity_ref` / invalid hex → fail; membership absent; `trust.json` unchanged
- revoked identity → fail
- second federation_id → fail, first membership intact
- same id+key → idempotent
- same id, different key (membership or TrustStore) → fail
- refuse `aira:identity:local-test`
- `cargo test -p aira-object` (`with_verifying_hex`); `cargo test -p aira-protocol` (federation)

## Done when
Join+trust durable; Living Spec; no CRP/settlement/peer wire.
