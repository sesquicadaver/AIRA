# DI crystallize — Analyze-70 / QUEUE #35

## In scope (A + A1 + B1 + C1 + F1)
1. CLI `aira federation join --descriptor <path>`.
2. Self-signed federation descriptor (Ed25519, domain `aira:federation:descriptor:v1`) verified against **pubkey inside the descriptor** (`identity_ref` == `signature.key_ref`).
3. On success: upsert `identity_ref` into TrustStore; write `.aira/federation/membership.json`.
4. Revoked `identity_ref` → fail-closed (no membership write).
5. One membership: same `federation_id` + key → idempotent; different `federation_id` → fail-closed.
6. Tests (verify / revoke / conflict / idempotent) + docs (`docs/peer-link.md`, layout note).

## Out
Join Request/Response (even files); new peer message; CRP / capability routing / import-export; settlement; Federation CSU; hello/Noise/trust-delta changes; HTTP federation API; `federation leave` / exit / federation-revoke.

## Decision boundaries (agent-owned)
Descriptor subset: `schema`, `federation_id`, `federation_type`, `identity_ref`, `public_key_hex`, `signature`. Canonical bytes: domain-concat (hello-style), not JSON. `signature.key_ref == identity_ref`. Same federation_id + different key → fail-closed in **join wrapper** (do not change `TrustStore::upsert`). Logic in `aira-protocol` (`federation`); CLI thin. Generic detached verify via `Keyring::with_verifying_hex` in `aira-object`. Wrapper also fail-closed: existing TrustStore pubkey mismatch for `identity_ref`; refuse `aira:identity:local-test`. Persist: `TrustStore::save` + `register_trust_store`, then membership. Membership schema `aira:federation:membership:v1` includes `joined_at`. Canonical bytes: `aira:federation:descriptor:v1|{schema}|{federation_id}|{federation_type}|{identity_ref}|{public_key_hex}`.
